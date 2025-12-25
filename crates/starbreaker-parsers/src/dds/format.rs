//! DDS texture format detection

use super::header::{DdsHeader, DX10Header, PixelFormat, pf_flags};

/// DXGI format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DxgiFormat {
    Unknown = 0,
    BC1Unorm = 71,
    BC1UnormSrgb = 72,
    BC2Unorm = 74,
    BC2UnormSrgb = 75,
    BC3Unorm = 77,
    BC3UnormSrgb = 78,
    BC4Unorm = 80,
    BC4Snorm = 81,
    BC5Unorm = 83,
    BC5Snorm = 84,
    BC6HUf16 = 95,
    BC6HSf16 = 96,
    BC7Unorm = 98,
    BC7UnormSrgb = 99,
    R8G8B8A8Unorm = 28,
    R8G8B8A8UnormSrgb = 29,
    B8G8R8A8Unorm = 87,
    B8G8R8A8UnormSrgb = 91,
}

impl DxgiFormat {
    /// Create from u32 value
    pub fn from_u32(value: u32) -> Self {
        match value {
            71 => DxgiFormat::BC1Unorm,
            72 => DxgiFormat::BC1UnormSrgb,
            74 => DxgiFormat::BC2Unorm,
            75 => DxgiFormat::BC2UnormSrgb,
            77 => DxgiFormat::BC3Unorm,
            78 => DxgiFormat::BC3UnormSrgb,
            80 => DxgiFormat::BC4Unorm,
            81 => DxgiFormat::BC4Snorm,
            83 => DxgiFormat::BC5Unorm,
            84 => DxgiFormat::BC5Snorm,
            95 => DxgiFormat::BC6HUf16,
            96 => DxgiFormat::BC6HSf16,
            98 => DxgiFormat::BC7Unorm,
            99 => DxgiFormat::BC7UnormSrgb,
            28 => DxgiFormat::R8G8B8A8Unorm,
            29 => DxgiFormat::R8G8B8A8UnormSrgb,
            87 => DxgiFormat::B8G8R8A8Unorm,
            91 => DxgiFormat::B8G8R8A8UnormSrgb,
            _ => DxgiFormat::Unknown,
        }
    }
}

/// Detected texture format
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextureFormat {
    /// Block Compressed 1 (DXT1)
    BC1,
    /// Block Compressed 2 (DXT3)
    BC2,
    /// Block Compressed 3 (DXT5)
    BC3,
    /// Block Compressed 4
    BC4,
    /// Block Compressed 5
    BC5,
    /// Block Compressed 6 (HDR)
    BC6H,
    /// Block Compressed 7
    BC7,
    /// Uncompressed RGBA8
    RGBA8,
    /// Uncompressed BGRA8
    BGRA8,
    /// Unknown format
    Unknown,
}

impl TextureFormat {
    /// Detect format from DDS header
    pub fn from_header(header: &DdsHeader, dx10: Option<&DX10Header>) -> Self {
        // Check DX10 header first
        if let Some(dx10_hdr) = dx10 {
            return Self::from_dxgi_format(dx10_hdr.dxgi_format);
        }

        // Check FourCC
        Self::from_fourcc(&header.pixel_format)
    }

    /// Detect from DXGI format
    fn from_dxgi_format(format: u32) -> Self {
        match DxgiFormat::from_u32(format) {
            DxgiFormat::BC1Unorm | DxgiFormat::BC1UnormSrgb => TextureFormat::BC1,
            DxgiFormat::BC2Unorm | DxgiFormat::BC2UnormSrgb => TextureFormat::BC2,
            DxgiFormat::BC3Unorm | DxgiFormat::BC3UnormSrgb => TextureFormat::BC3,
            DxgiFormat::BC4Unorm | DxgiFormat::BC4Snorm => TextureFormat::BC4,
            DxgiFormat::BC5Unorm | DxgiFormat::BC5Snorm => TextureFormat::BC5,
            DxgiFormat::BC6HUf16 | DxgiFormat::BC6HSf16 => TextureFormat::BC6H,
            DxgiFormat::BC7Unorm | DxgiFormat::BC7UnormSrgb => TextureFormat::BC7,
            DxgiFormat::R8G8B8A8Unorm | DxgiFormat::R8G8B8A8UnormSrgb => TextureFormat::RGBA8,
            DxgiFormat::B8G8R8A8Unorm | DxgiFormat::B8G8R8A8UnormSrgb => TextureFormat::BGRA8,
            _ => TextureFormat::Unknown,
        }
    }

    /// Detect from pixel format FourCC
    fn from_fourcc(pf: &PixelFormat) -> Self {
        if pf.flags & pf_flags::FOURCC != 0 {
            match &pf.fourcc {
                b"DXT1" => TextureFormat::BC1,
                b"DXT2" | b"DXT3" => TextureFormat::BC2,
                b"DXT4" | b"DXT5" => TextureFormat::BC3,
                b"ATI1" | b"BC4U" => TextureFormat::BC4,
                b"ATI2" | b"BC5U" => TextureFormat::BC5,
                _ => TextureFormat::Unknown,
            }
        } else if pf.flags & pf_flags::RGB != 0 {
            if pf.rgb_bit_count == 32 {
                if pf.r_bit_mask == 0x000000FF {
                    TextureFormat::RGBA8
                } else {
                    TextureFormat::BGRA8
                }
            } else {
                TextureFormat::Unknown
            }
        } else {
            TextureFormat::Unknown
        }
    }

    /// Get block size for compressed formats
    pub fn block_size(&self) -> Option<usize> {
        match self {
            TextureFormat::BC1 | TextureFormat::BC4 => Some(8),
            TextureFormat::BC2 | TextureFormat::BC3 | TextureFormat::BC5 
            | TextureFormat::BC6H | TextureFormat::BC7 => Some(16),
            _ => None,
        }
    }

    /// Check if format is block-compressed
    pub fn is_compressed(&self) -> bool {
        matches!(self, 
            TextureFormat::BC1 | TextureFormat::BC2 | TextureFormat::BC3 |
            TextureFormat::BC4 | TextureFormat::BC5 | TextureFormat::BC6H |
            TextureFormat::BC7
        )
    }
}
