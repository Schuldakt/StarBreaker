//! Texture conversion and decompression utilities
//!
//! Converts DDS textures to common formats (PNG, TGA) with BC decompression support.

mod decompressor;
mod converter;

pub use converter::{TextureConverter, TextureConvertOptions, ImageFormat};
pub use decompressor::decompress_bc;

use thiserror::Error;

/// Texture conversion errors
#[derive(Error, Debug)]
pub enum TextureError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    
    #[error("Invalid dimensions: {width}x{height}")]
    InvalidDimensions { width: u32, height: u32 },
    
    #[error("Invalid mipmap level: {level} (max: {max})")]
    InvalidMipLevel { level: u32, max: u32 },
}

pub type TextureResult<T> = Result<T, TextureError>;
