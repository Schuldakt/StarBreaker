//! DDS header structures

use std::io::{Read, Seek};
use crate::traits::{ParseResult, ParseError};

/// DDS header flags
pub mod flags {
    pub const CAPS: u32 = 0x1;
    pub const HEIGHT: u32 = 0x2;
    pub const WIDTH: u32 = 0x4;
    pub const PITCH: u32 = 0x8;
    pub const PIXEL_FORMAT: u32 = 0x1000;
    pub const MIPMAP_COUNT: u32 = 0x20000;
    pub const LINEAR_SIZE: u32 = 0x80000;
    pub const DEPTH: u32 = 0x800000;
}

/// Caps flags
pub mod caps {
    pub const COMPLEX: u32 = 0x8;
    pub const TEXTURE: u32 = 0x1000;
    pub const MIPMAP: u32 = 0x400000;
}

/// Caps2 flags
pub mod caps2 {
    pub const CUBEMAP: u32 = 0x200;
    pub const CUBEMAP_POSITIVEX: u32 = 0x400;
    pub const CUBEMAP_NEGATIVEX: u32 = 0x800;
    pub const CUBEMAP_POSITIVEY: u32 = 0x1000;
    pub const CUBEMAP_NEGATIVEY: u32 = 0x2000;
    pub const CUBEMAP_POSITIVEZ: u32 = 0x4000;
    pub const CUBEMAP_NEGATIVEZ: u32 = 0x8000;
    pub const VOLUME: u32 = 0x200000;
}

/// DDS pixel format flags
pub mod pf_flags {
    pub const ALPHAPIXELS: u32 = 0x1;
    pub const ALPHA: u32 = 0x2;
    pub const FOURCC: u32 = 0x4;
    pub const RGB: u32 = 0x40;
    pub const YUV: u32 = 0x200;
    pub const LUMINANCE: u32 = 0x20000;
}

/// DDS header (124 bytes)
#[derive(Debug, Clone)]
pub struct DdsHeader {
    pub size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mipmap_count: u32,
    pub reserved1: [u32; 11],
    pub pixel_format: PixelFormat,
    pub caps: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,
    pub reserved2: u32,
}

impl DdsHeader {
    /// Parse DDS header from reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> ParseResult<Self> {
        let mut header_data = [0u8; 124];
        reader.read_exact(&mut header_data)?;

        let size = u32::from_le_bytes([header_data[0], header_data[1], header_data[2], header_data[3]]);
        
        if size != 124 {
            return Err(ParseError::InvalidStructure(
                format!("Invalid DDS header size: expected 124, got {}", size)
            ));
        }

        let flags = u32::from_le_bytes([header_data[4], header_data[5], header_data[6], header_data[7]]);
        let height = u32::from_le_bytes([header_data[8], header_data[9], header_data[10], header_data[11]]);
        let width = u32::from_le_bytes([header_data[12], header_data[13], header_data[14], header_data[15]]);
        let pitch_or_linear_size = u32::from_le_bytes([header_data[16], header_data[17], header_data[18], header_data[19]]);
        let depth = u32::from_le_bytes([header_data[20], header_data[21], header_data[22], header_data[23]]);
        let mipmap_count = u32::from_le_bytes([header_data[24], header_data[25], header_data[26], header_data[27]]);

        // Reserved1 (11 u32s)
        let mut reserved1 = [0u32; 11];
        for i in 0..11 {
            let offset = 28 + i * 4;
            reserved1[i] = u32::from_le_bytes([
                header_data[offset],
                header_data[offset + 1],
                header_data[offset + 2],
                header_data[offset + 3],
            ]);
        }

        // Pixel format (32 bytes starting at offset 72)
        let pixel_format = PixelFormat::parse(&header_data[72..104])?;

        let caps = u32::from_le_bytes([header_data[104], header_data[105], header_data[106], header_data[107]]);
        let caps2 = u32::from_le_bytes([header_data[108], header_data[109], header_data[110], header_data[111]]);
        let caps3 = u32::from_le_bytes([header_data[112], header_data[113], header_data[114], header_data[115]]);
        let caps4 = u32::from_le_bytes([header_data[116], header_data[117], header_data[118], header_data[119]]);
        let reserved2 = u32::from_le_bytes([header_data[120], header_data[121], header_data[122], header_data[123]]);

        Ok(DdsHeader {
            size,
            flags,
            height,
            width,
            pitch_or_linear_size,
            depth,
            mipmap_count,
            reserved1,
            pixel_format,
            caps,
            caps2,
            caps3,
            caps4,
            reserved2,
        })
    }

    /// Check if this DDS has a DX10 extended header
    pub fn has_dx10_header(&self) -> bool {
        self.pixel_format.fourcc == *b"DX10"
    }

    /// Check if this is a cubemap
    pub fn is_cubemap(&self) -> bool {
        self.caps2 & caps2::CUBEMAP != 0
    }

    /// Check if this has mipmaps
    pub fn has_mipmaps(&self) -> bool {
        self.caps & caps::MIPMAP != 0 && self.mipmap_count > 1
    }
}

/// DDS pixel format (32 bytes)
#[derive(Debug, Clone)]
pub struct PixelFormat {
    pub size: u32,
    pub flags: u32,
    pub fourcc: [u8; 4],
    pub rgb_bit_count: u32,
    pub r_bit_mask: u32,
    pub g_bit_mask: u32,
    pub b_bit_mask: u32,
    pub a_bit_mask: u32,
}

impl PixelFormat {
    /// Parse pixel format from 32-byte slice
    pub fn parse(data: &[u8]) -> ParseResult<Self> {
        if data.len() < 32 {
            return Err(ParseError::InvalidStructure(
                "Pixel format data too short".to_string()
            ));
        }

        let size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let flags = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let fourcc = [data[8], data[9], data[10], data[11]];
        let rgb_bit_count = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let r_bit_mask = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let g_bit_mask = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let b_bit_mask = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let a_bit_mask = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

        Ok(PixelFormat {
            size,
            flags,
            fourcc,
            rgb_bit_count,
            r_bit_mask,
            g_bit_mask,
            b_bit_mask,
            a_bit_mask,
        })
    }

    /// Get FourCC as string
    pub fn fourcc_string(&self) -> String {
        String::from_utf8_lossy(&self.fourcc).to_string()
    }
}

/// DX10 extended header
#[derive(Debug, Clone)]
pub struct DX10Header {
    pub dxgi_format: u32,
    pub resource_dimension: u32,
    pub misc_flag: u32,
    pub array_size: u32,
    pub misc_flags2: u32,
}

impl DX10Header {
    /// Parse DX10 header from reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> ParseResult<Self> {
        let mut header_data = [0u8; 20];
        reader.read_exact(&mut header_data)?;

        let dxgi_format = u32::from_le_bytes([header_data[0], header_data[1], header_data[2], header_data[3]]);
        let resource_dimension = u32::from_le_bytes([header_data[4], header_data[5], header_data[6], header_data[7]]);
        let misc_flag = u32::from_le_bytes([header_data[8], header_data[9], header_data[10], header_data[11]]);
        let array_size = u32::from_le_bytes([header_data[12], header_data[13], header_data[14], header_data[15]]);
        let misc_flags2 = u32::from_le_bytes([header_data[16], header_data[17], header_data[18], header_data[19]]);

        Ok(DX10Header {
            dxgi_format,
            resource_dimension,
            misc_flag,
            array_size,
            misc_flags2,
        })
    }
}
