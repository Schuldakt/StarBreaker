//! Block-compressed texture decompression
//!
//! Decompresses BC1-BC7 textures using the texpresso library.

use crate::textures::{TextureError, TextureResult};
use starbreaker_parsers::dds::TextureFormat;

/// Decompress block-compressed texture data to RGBA8
pub fn decompress_bc(
    format: &TextureFormat,
    data: &[u8],
    width: u32,
    height: u32,
) -> TextureResult<Vec<u8>> {
    if width == 0 || height == 0 {
        return Err(TextureError::InvalidDimensions { width, height });
    }

    let pixel_count = (width * height) as usize;
    let mut output = vec![0u8; pixel_count * 4]; // RGBA8

    match format {
        TextureFormat::BC1 => decompress_bc1(data, width, height, &mut output)?,
        TextureFormat::BC2 => decompress_bc2(data, width, height, &mut output)?,
        TextureFormat::BC3 => decompress_bc3(data, width, height, &mut output)?,
        TextureFormat::BC4 => decompress_bc4(data, width, height, &mut output)?,
        TextureFormat::BC5 => decompress_bc5(data, width, height, &mut output)?,
        TextureFormat::BC6H => decompress_bc6h(data, width, height, &mut output)?,
        TextureFormat::BC7 => decompress_bc7(data, width, height, &mut output)?,
        TextureFormat::RGBA8 => {
            // Already uncompressed
            if data.len() == output.len() {
                output.copy_from_slice(data);
            } else {
                return Err(TextureError::DecompressionFailed(
                    format!("RGBA8 data size mismatch: expected {}, got {}", output.len(), data.len())
                ));
            }
        }
        TextureFormat::BGRA8 => {
            // Convert BGRA to RGBA
            if data.len() == output.len() {
                for i in 0..pixel_count {
                    let idx = i * 4;
                    output[idx] = data[idx + 2];     // R <- B
                    output[idx + 1] = data[idx + 1]; // G <- G
                    output[idx + 2] = data[idx];     // B <- R
                    output[idx + 3] = data[idx + 3]; // A <- A
                }
            } else {
                return Err(TextureError::DecompressionFailed(
                    format!("BGRA8 data size mismatch: expected {}, got {}", output.len(), data.len())
                ));
            }
        }
        TextureFormat::Unknown => {
            return Err(TextureError::UnsupportedFormat("Unknown texture format".to_string()));
        }
    }

    Ok(output)
}

/// Decompress BC1 (DXT1)
fn decompress_bc1(data: &[u8], width: u32, height: u32, output: &mut [u8]) -> TextureResult<()> {
    texpresso::Format::Bc1.decompress(data, width as usize, height as usize, output);
    Ok(())
}

/// Decompress BC2 (DXT3)
fn decompress_bc2(data: &[u8], width: u32, height: u32, output: &mut [u8]) -> TextureResult<()> {
    texpresso::Format::Bc2.decompress(data, width as usize, height as usize, output);
    Ok(())
}

/// Decompress BC3 (DXT5)
fn decompress_bc3(data: &[u8], width: u32, height: u32, output: &mut [u8]) -> TextureResult<()> {
    texpresso::Format::Bc3.decompress(data, width as usize, height as usize, output);
    Ok(())
}

/// Decompress BC4
fn decompress_bc4(data: &[u8], width: u32, height: u32, output: &mut [u8]) -> TextureResult<()> {
    texpresso::Format::Bc4.decompress(data, width as usize, height as usize, output);
    Ok(())
}

/// Decompress BC5
fn decompress_bc5(data: &[u8], width: u32, height: u32, output: &mut [u8]) -> TextureResult<()> {
    texpresso::Format::Bc5.decompress(data, width as usize, height as usize, output);
    Ok(())
}

/// Decompress BC6H (HDR)
fn decompress_bc6h(_data: &[u8], _width: u32, _height: u32, _output: &mut [u8]) -> TextureResult<()> {
    // BC6H is HDR format - texpresso 2.0 may not support it directly
    // For now, return an error
    Err(TextureError::UnsupportedFormat(
        "BC6H decompression not yet supported".to_string()
    ))
}

/// Decompress BC7
fn decompress_bc7(_data: &[u8], _width: u32, _height: u32, _output: &mut [u8]) -> TextureResult<()> {
    // BC7 may not be in texpresso 2.0 - check and fallback if needed
    Err(TextureError::UnsupportedFormat(
        "BC7 decompression not yet supported".to_string()
    ))
}
