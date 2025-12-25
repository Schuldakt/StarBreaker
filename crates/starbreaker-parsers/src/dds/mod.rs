//! DDS (DirectDraw Surface) texture format parser
//!
//! Parses DDS texture files used by Star Citizen, including split files (.dds.1, .dds.2, etc.)
//!
//! # Split Files
//! Star Citizen splits large DDS textures across multiple files to work around file size limits.
//! The combiner automatically detects and merges these split files.

mod header;
mod format;
mod combiner;

pub use header::{DdsHeader, DX10Header, PixelFormat};
pub use format::{DxgiFormat, TextureFormat};
pub use combiner::DdsCombiner;

use std::io::{Read, Seek};
use crate::traits::{Parser, ParseResult, ParseError, ParseOptions, ProgressCallback};

/// DDS file magic number "DDS "
const DDS_MAGIC: u32 = 0x20534444;

/// Parsed DDS texture
#[derive(Debug)]
pub struct DdsTexture {
    /// DDS header
    pub header: DdsHeader,
    /// DX10 extended header (if present)
    pub dx10_header: Option<DX10Header>,
    /// Raw texture data
    pub data: Vec<u8>,
    /// Detected texture format
    pub format: TextureFormat,
    /// Whether this was combined from split files
    pub was_split: bool,
}

impl DdsTexture {
    /// Get width in pixels
    pub fn width(&self) -> u32 {
        self.header.width
    }

    /// Get height in pixels
    pub fn height(&self) -> u32 {
        self.header.height
    }

    /// Get mipmap count
    pub fn mipmap_count(&self) -> u32 {
        self.header.mipmap_count
    }

    /// Get total data size
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    /// Check if texture has mipmaps
    pub fn has_mipmaps(&self) -> bool {
        self.header.mipmap_count > 1
    }

    /// Check if texture is a cubemap
    pub fn is_cubemap(&self) -> bool {
        self.header.is_cubemap()
    }

    /// Get data for a specific mipmap level
    /// Returns None if the level doesn't exist
    pub fn get_mipmap(&self, level: u32) -> Option<&[u8]> {
        if level >= self.mipmap_count() {
            return None;
        }

        let mut offset = 0;
        let mut width = self.width();
        let mut height = self.height();

        // Calculate offset to the requested mip level
        for _ in 0..level {
            let mip_size = self.calculate_mip_size(width, height);
            offset += mip_size;
            
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }

        let mip_size = self.calculate_mip_size(width, height);
        
        if offset + mip_size <= self.data.len() {
            Some(&self.data[offset..offset + mip_size])
        } else {
            None
        }
    }

    /// Calculate the size of a mip level in bytes
    fn calculate_mip_size(&self, width: u32, height: u32) -> usize {
        match &self.format {
            TextureFormat::BC1 => {
                // BC1: 8 bytes per 4x4 block
                let block_width = (width + 3) / 4;
                let block_height = (height + 3) / 4;
                (block_width * block_height * 8) as usize
            }
            TextureFormat::BC2 | TextureFormat::BC3 => {
                // BC2/BC3: 16 bytes per 4x4 block
                let block_width = (width + 3) / 4;
                let block_height = (height + 3) / 4;
                (block_width * block_height * 16) as usize
            }
            TextureFormat::BC4 => {
                // BC4: 8 bytes per 4x4 block (single channel)
                let block_width = (width + 3) / 4;
                let block_height = (height + 3) / 4;
                (block_width * block_height * 8) as usize
            }
            TextureFormat::BC5 => {
                // BC5: 16 bytes per 4x4 block (two channels)
                let block_width = (width + 3) / 4;
                let block_height = (height + 3) / 4;
                (block_width * block_height * 16) as usize
            }
            TextureFormat::BC6H | TextureFormat::BC7 => {
                // BC6H/BC7: 16 bytes per 4x4 block
                let block_width = (width + 3) / 4;
                let block_height = (height + 3) / 4;
                (block_width * block_height * 16) as usize
            }
            TextureFormat::RGBA8 | TextureFormat::BGRA8 => {
                // Uncompressed RGBA/BGRA: 4 bytes per pixel
                (width * height * 4) as usize
            }
            TextureFormat::Unknown => {
                // Fallback: assume RGBA
                (width * height * 4) as usize
            }
        }
    }

    /// Extract all mipmap levels
    /// Returns a vector of (level, width, height, data) tuples
    pub fn extract_mipmaps(&self) -> Vec<(u32, u32, u32, Vec<u8>)> {
        let mut mipmaps = Vec::new();
        let mut width = self.width();
        let mut height = self.height();

        for level in 0..self.mipmap_count() {
            if let Some(mip_data) = self.get_mipmap(level) {
                mipmaps.push((level, width, height, mip_data.to_vec()));
            }
            
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }

        mipmaps
    }

    /// Get the dimensions of a specific mipmap level
    pub fn get_mipmap_dimensions(&self, level: u32) -> Option<(u32, u32)> {
        if level >= self.mipmap_count() {
            return None;
        }

        let mut width = self.width();
        let mut height = self.height();

        for _ in 0..level {
            width = (width / 2).max(1);
            height = (height / 2).max(1);
        }

        Some((width, height))
    }
}

/// DDS Parser
pub struct DdsParser;

impl DdsParser {
    /// Create a new DDS parser
    pub fn new() -> Self {
        Self
    }
}

impl Default for DdsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for DdsParser {
    type Output = DdsTexture;

    fn extensions(&self) -> &[&str] {
        &["dds"]
    }

    fn magic_bytes(&self) -> Option<&[u8]> {
        None  // DDS only has one magic, but return None for simplicity
    }

    fn name(&self) -> &str {
        "DDS Texture Parser"
    }

    fn parse_with_options<R: Read + Seek>(
        &self,
        mut reader: R,
        _options: &ParseOptions,
        _progress: Option<ProgressCallback>,
    ) -> ParseResult<Self::Output> {
        // Read magic
        let mut magic_buf = [0u8; 4];
        reader.read_exact(&mut magic_buf)?;
        let magic = u32::from_le_bytes(magic_buf);

        if magic != DDS_MAGIC {
            return Err(ParseError::InvalidMagic {
                expected: DDS_MAGIC.to_le_bytes().to_vec(),
                found: magic_buf.to_vec(),
            });
        }

        // Parse header
        let header = DdsHeader::parse(&mut reader)?;

        // Parse DX10 header if present
        let dx10_header = if header.has_dx10_header() {
            Some(DX10Header::parse(&mut reader)?)
        } else {
            None
        };

        // Detect format
        let format = TextureFormat::from_header(&header, dx10_header.as_ref());

        // Read texture data
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        Ok(DdsTexture {
            header,
            dx10_header,
            data,
            format,
            was_split: false,
        })
    }
}
