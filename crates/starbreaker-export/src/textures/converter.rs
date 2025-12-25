//! Texture format converter
//!
//! Converts DDS textures to PNG, TGA, and other formats.

use crate::textures::{TextureError, TextureResult, decompressor};
use starbreaker_parsers::dds::DdsTexture;
use image::{RgbaImage, ImageFormat as ImgFormat, DynamicImage};
use std::path::Path;

/// Output image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG format (lossless, good compression)
    Png,
    /// TGA format (lossless, simple)
    Tga,
    /// BMP format (lossless, no compression)
    Bmp,
    /// JPEG format (lossy, smaller size)
    Jpeg { quality: u8 },
}

impl ImageFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Tga => "tga",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Jpeg { .. } => "jpg",
        }
    }

    /// Convert to image crate's format
    fn to_img_format(&self) -> ImgFormat {
        match self {
            ImageFormat::Png => ImgFormat::Png,
            ImageFormat::Tga => ImgFormat::Tga,
            ImageFormat::Bmp => ImgFormat::Bmp,
            ImageFormat::Jpeg { .. } => ImgFormat::Jpeg,
        }
    }
}

/// Texture conversion options
#[derive(Debug, Clone)]
pub struct TextureConvertOptions {
    /// Output format
    pub format: ImageFormat,
    
    /// Include mipmaps (export multiple files)
    pub include_mipmaps: bool,
    
    /// Flip Y axis (useful for normal maps)
    pub flip_y: bool,
    
    /// Maximum mipmap level to export (0 = only main texture)
    pub max_mip_level: Option<u32>,
    
    /// Handle normal maps (convert from DX to OpenGL format)
    pub convert_normal_map: bool,
}

impl Default for TextureConvertOptions {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            include_mipmaps: false,
            flip_y: false,
            max_mip_level: None,
            convert_normal_map: false,
        }
    }
}

/// Texture converter
pub struct TextureConverter {
    options: TextureConvertOptions,
}

impl TextureConverter {
    /// Create new converter with default options
    pub fn new() -> Self {
        Self {
            options: TextureConvertOptions::default(),
        }
    }

    /// Create converter with custom options
    pub fn with_options(options: TextureConvertOptions) -> Self {
        Self { options }
    }

    /// Convert DDS texture to output format
    /// 
    /// # Arguments
    /// 
    /// * `texture` - DDS texture to convert
    /// * `output_path` - Output file path (extension will be added)
    /// 
    /// # Returns
    /// 
    /// Number of files written (1 for main texture, more if mipmaps included)
    pub fn convert(&self, texture: &DdsTexture, output_path: impl AsRef<Path>) -> TextureResult<usize> {
        let output_path = output_path.as_ref();
        let mut files_written = 0;

        // Determine how many mip levels to export
        let max_level = if self.options.include_mipmaps {
            let max = texture.mipmap_count().saturating_sub(1);
            self.options.max_mip_level.map(|limit| limit.min(max)).unwrap_or(max)
        } else {
            0
        };

        // Export each mip level
        for level in 0..=max_level {
            let mip_data = texture.get_mipmap(level).ok_or(TextureError::InvalidMipLevel {
                level,
                max: texture.mipmap_count().saturating_sub(1),
            })?;

            // Calculate dimensions for this mip level
            let width = (texture.width() >> level).max(1);
            let height = (texture.height() >> level).max(1);

            // Decompress texture data to RGBA8
            let rgba_data = decompressor::decompress_bc(
                &texture.format,
                mip_data,
                width,
                height,
            )?;

            // Create image
            let mut img = RgbaImage::from_raw(width, height, rgba_data)
                .ok_or(TextureError::DecompressionFailed(
                    "Failed to create image from decompressed data".to_string()
                ))?;

            // Apply transformations
            if self.options.flip_y {
                image::imageops::flip_vertical_in_place(&mut img);
            }

            if self.options.convert_normal_map {
                self.convert_normal_map_format(&mut img);
            }

            // Determine output filename
            let output_file = if level == 0 {
                output_path.with_extension(self.options.format.extension())
            } else {
                let stem = output_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("texture");
                let parent = output_path.parent().unwrap_or(Path::new("."));
                parent.join(format!("{}_mip{}.{}", stem, level, self.options.format.extension()))
            };

            // Write image
            self.write_image(&img, &output_file)?;
            files_written += 1;
        }

        Ok(files_written)
    }

    /// Convert multiple textures in batch
    /// 
    /// Returns (successful_count, total_bytes_written)
    pub fn convert_batch(
        &self,
        textures: &[(DdsTexture, impl AsRef<Path>)],
    ) -> (usize, u64) {
        let mut success_count = 0;
        let mut total_bytes = 0u64;

        for (texture, output_path) in textures {
            match self.convert(texture, output_path) {
                Ok(_) => {
                    success_count += 1;
                    // Estimate file size (very rough)
                    let pixels = texture.width() * texture.height();
                    total_bytes += pixels as u64 * 4; // RGBA8
                }
                Err(_) => {
                    // Continue on error
                }
            }
        }

        (success_count, total_bytes)
    }

    /// Write image to file
    fn write_image(&self, img: &RgbaImage, output_path: &Path) -> TextureResult<()> {
        let dynamic_img = DynamicImage::ImageRgba8(img.clone());
        dynamic_img.save_with_format(output_path, self.options.format.to_img_format())?;
        Ok(())
    }

    /// Convert normal map from DirectX format (Y+) to OpenGL format (Y-)
    /// 
    /// In DirectX, green channel points up (+Y), in OpenGL it points down (-Y)
    fn convert_normal_map_format(&self, img: &mut RgbaImage) {
        for pixel in img.pixels_mut() {
            // Invert green channel
            pixel[1] = 255 - pixel[1];
        }
    }

    /// Extract specific mipmap level as standalone image
    pub fn extract_mipmap(
        &self,
        texture: &DdsTexture,
        level: u32,
        output_path: impl AsRef<Path>,
    ) -> TextureResult<()> {
        let mip_data = texture.get_mipmap(level).ok_or(TextureError::InvalidMipLevel {
            level,
            max: texture.mipmap_count().saturating_sub(1),
        })?;

        let width = (texture.width() >> level).max(1);
        let height = (texture.height() >> level).max(1);

        let rgba_data = decompressor::decompress_bc(
            &texture.format,
            mip_data,
            width,
            height,
        )?;

        let img = RgbaImage::from_raw(width, height, rgba_data)
            .ok_or(TextureError::DecompressionFailed(
                "Failed to create image from mipmap".to_string()
            ))?;

        self.write_image(&img, output_path.as_ref())?;

        Ok(())
    }

    /// Get texture information without converting
    pub fn get_info(texture: &DdsTexture) -> TextureInfo {
        TextureInfo {
            width: texture.width(),
            height: texture.height(),
            mipmap_count: texture.mipmap_count(),
            format: format!("{:?}", texture.format),
            is_cubemap: texture.is_cubemap(),
            data_size: texture.data_size(),
        }
    }
}

impl Default for TextureConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// Texture information
#[derive(Debug, Clone)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub mipmap_count: u32,
    pub format: String,
    pub is_cubemap: bool,
    pub data_size: usize,
}
