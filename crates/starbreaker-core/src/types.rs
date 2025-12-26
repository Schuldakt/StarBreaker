//! Common types used across StarBreaker
//!
//! This module provides shared type definitions used by multiple crates.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Unique identifier for game entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Create a new entity ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the raw ID value
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}

impl From<u64> for EntityId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

/// GUID (Globally Unique Identifier) used in game data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl Guid {
    /// Create a GUID from raw bytes
    pub fn from_bytes(bytes: &[u8; 16]) -> Self {
        Self {
            data1: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            data2: u16::from_le_bytes([bytes[4], bytes[5]]),
            data3: u16::from_le_bytes([bytes[6], bytes[7]]),
            data4: [
                bytes[8], bytes[9], bytes[10], bytes[11],
                bytes[12], bytes[13], bytes[14], bytes[15],
            ],
        }
    }

    /// Convert to standard GUID string format
    pub fn to_string_standard(&self) -> String {
        format!(
            "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            self.data1, self.data2, self.data3,
            self.data4[0], self.data4[1],
            self.data4[2], self.data4[3], self.data4[4],
            self.data4[5], self.data4[6], self.data4[7]
        )
    }
}

impl std::fmt::Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string_standard())
    }
}

/// 3D vector (position, normal, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const FORWARD: Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            Self::ZERO
        }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::ZERO
    }
}

/// 4D vector (quaternion, color with alpha, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, z: 1.0, w: 1.0 };
    pub const IDENTITY: Self = Self { x: 0.0, y: 0.0, z: 0.0, w: 1.0 };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

impl Default for Vec4 {
    fn default() -> Self {
        Self::ZERO
    }
}

/// 2D vector (UV coordinates, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::ZERO
    }
}

/// 4x4 transformation matrix
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mat4x4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4x4 {
    pub const IDENTITY: Self = Self {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// Create a new matrix from a flat array
    pub fn from_flat(data: &[f32; 16]) -> Self {
        Self {
            m: [
                [data[0], data[1], data[2], data[3]],
                [data[4], data[5], data[6], data[7]],
                [data[8], data[9], data[10], data[11]],
                [data[12], data[13], data[14], data[15]],
            ],
        }
    }

    /// Get translation component
    pub fn translation(&self) -> Vec3 {
        Vec3::new(self.m[3][0], self.m[3][1], self.m[3][2])
    }
}

impl Default for Mat4x4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BoundingBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BoundingBox {
    pub const ZERO: Self = Self {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
    };

    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    pub fn size(&self) -> Vec3 {
        Vec3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    pub fn expand(&mut self, point: Vec3) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self::ZERO
    }
}

/// Color in RGBA format (0-255 per channel)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const RED: Self = Self { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };

    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Convert to normalized float values (0.0-1.0)
    pub fn to_float(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Asset reference (path or ID)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetRef {
    /// Reference by file path
    Path(PathBuf),
    /// Reference by entity ID
    Id(EntityId),
    /// Reference by name
    Name(String),
}

impl AssetRef {
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    pub fn id(id: u64) -> Self {
        Self::Id(EntityId::new(id))
    }

    pub fn name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec3_operations() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        assert!((v1.dot(&v2) - 32.0).abs() < 0.001);
        
        let cross = v1.cross(&v2);
        assert!((cross.x - (-3.0)).abs() < 0.001);
        assert!((cross.y - 6.0).abs() < 0.001);
        assert!((cross.z - (-3.0)).abs() < 0.001);
    }

    #[test]
    fn test_guid_format() {
        let guid = Guid {
            data1: 0x12345678,
            data2: 0x1234,
            data3: 0x5678,
            data4: [0x9A, 0xBC, 0xDE, 0xF0, 0x12, 0x34, 0x56, 0x78],
        };
        
        assert_eq!(
            guid.to_string_standard(),
            "12345678-1234-5678-9ABC-DEF012345678"
        );
    }

    #[test]
    fn test_bounding_box_expand() {
        let mut bbox = BoundingBox::new(Vec3::ZERO, Vec3::ZERO);
        bbox.expand(Vec3::new(1.0, 2.0, 3.0));
        bbox.expand(Vec3::new(-1.0, -2.0, -3.0));

        assert_eq!(bbox.min, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(bbox.max, Vec3::new(1.0, 2.0, 3.0));
    }
}