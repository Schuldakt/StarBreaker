// stardust-parsers/src/dcb/structs.rs
//! Structure and property definitions for DataCore Binary format

use serde::{Deserialize, Serialize};

/// Structure definition in DCB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDef {
    /// Unique ID
    pub id: u32,
    /// Structure name
    pub name: String,
    /// Parent structure ID (for inheritance)
    pub parent_id: Option<u32>,
    /// First property index
    pub property_start: u32,
    /// Number of properties
    pub property_count: u32,
    /// Size in bytes when serialized
    pub size: u32,
    /// Flags
    pub flags: u32,
}

impl StructDef {
    /// Check if this struct inherits from another
    pub fn inherits_from(&self, parent_id: u32) -> bool {
        self.parent_id == Some(parent_id)
    }
    
    /// Get property indices for this struct
    pub fn property_indices(&self) -> std::ops::Range<usize> {
        let start = self.property_start as usize;
        let end = start + self.property_count as usize;
        start..end
    }
}

/// Property definition in DCB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyDef {
    /// Unique ID
    pub id: u32,
    /// Property name
    pub name: String,
    /// Data type
    pub data_type: DataType,
    /// Reference to struct type (for complex types)
    pub struct_id: Option<u32>,
    /// Conversion/modifier flags
    pub conversion: u32,
}

impl PropertyDef {
    /// Check if this property is a reference type
    pub fn is_reference(&self) -> bool {
        matches!(self.data_type, DataType::Reference)
    }
    
    /// Check if this property is an array
    pub fn is_array(&self) -> bool {
        matches!(self.data_type, DataType::Array(_))
    }
}

/// Data types supported by DCB
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    /// Boolean value (1 byte)
    Boolean,
    /// Signed 8-bit integer
    Int8,
    /// Signed 16-bit integer
    Int16,
    /// Signed 32-bit integer
    Int32,
    /// Signed 64-bit integer
    Int64,
    /// Unsigned 8-bit integer
    UInt8,
    /// Unsigned 16-bit integer
    UInt16,
    /// Unsigned 32-bit integer
    UInt32,
    /// Unsigned 64-bit integer
    UInt64,
    /// 32-bit float
    Float,
    /// 64-bit float
    Double,
    /// String (offset into string table)
    String,
    /// GUID (16 bytes)
    Guid,
    /// Locale string (with localization key)
    LocaleString,
    /// Reference to another record
    Reference,
    /// 3D vector (3 floats)
    Vec3,
    /// 4D vector/quaternion (4 floats)
    Vec4,
    /// Enumeration value
    Enum,
    /// Array of elements
    Array(Box<DataType>),
    /// Unknown type
    Unknown(u32),
}

impl DataType {
    /// Convert from raw u32 type ID
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => DataType::Boolean,
            1 => DataType::Int8,
            2 => DataType::Int16,
            3 | 4 => DataType::Int32,
            5 => DataType::Int64,
            6 => DataType::UInt8,
            7 => DataType::UInt16,
            8 => DataType::UInt32,
            9 => DataType::UInt64,
            10 => DataType::Float,
            11 => DataType::Double,
            12 => DataType::String,
            13 => DataType::Guid,
            14 => DataType::LocaleString,
            15 => DataType::Reference,
            16 => DataType::Vec3,
            17 => DataType::Vec4,
            18 => DataType::Enum,
            // Array types have high bit set
            v if v & 0x80000000 != 0 => {
                let inner_type = v & 0x7FFFFFFF;
                DataType::Array(Box::new(DataType::from_u32(inner_type)))
            }
            other => DataType::Unknown(other),
        }
    }
    
    /// Get the size in bytes for this type
    pub fn size(&self) -> Option<usize> {
        Some(match self {
            DataType::Boolean | DataType::Int8 | DataType::UInt8 => 1,
            DataType::Int16 | DataType::UInt16 => 2,
            DataType::Int32 | DataType::UInt32 | DataType::Float | 
            DataType::String | DataType::Enum => 4,
            DataType::Int64 | DataType::UInt64 | DataType::Double |
            DataType::Reference => 8,
            DataType::Vec3 => 12,
            DataType::Vec4 => 16,
            DataType::Guid => 16,
            DataType::LocaleString => 8, // offset + hash
            DataType::Array(_) => return None, // Variable size
            DataType::Unknown(_) => return None,
        })
    }
    
    /// Check if this is a numeric type
    pub fn is_numeric(&self) -> bool {
        matches!(self, 
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
            DataType::Float | DataType::Double
        )
    }
    
    /// Check if this is an integer type
    pub fn is_integer(&self) -> bool {
        matches!(self,
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64
        )
    }
    
    /// Check if this is a floating-point type
    pub fn is_float(&self) -> bool {
        matches!(self, DataType::Float | DataType::Double)
    }
    
    /// Get a human-readable type name
    pub fn type_name(&self) -> &'static str {
        match self {
            DataType::Boolean => "bool",
            DataType::Int8 => "i8",
            DataType::Int16 => "i16",
            DataType::Int32 => "i32",
            DataType::Int64 => "i64",
            DataType::UInt8 => "u8",
            DataType::UInt16 => "u16",
            DataType::UInt32 => "u32",
            DataType::UInt64 => "u64",
            DataType::Float => "f32",
            DataType::Double => "f64",
            DataType::String => "string",
            DataType::Guid => "guid",
            DataType::LocaleString => "locale_string",
            DataType::Reference => "reference",
            DataType::Vec3 => "vec3",
            DataType::Vec4 => "vec4",
            DataType::Enum => "enum",
            DataType::Array(_) => "array",
            DataType::Unknown(_) => "unknown",
        }
    }
}

/// Flag definitions for struct flags
pub mod struct_flags {
    pub const ABSTRACT: u32 = 0x01;
    pub const SERIALIZABLE: u32 = 0x02;
    pub const COMPONENT: u32 = 0x04;
    pub const ENTITY: u32 = 0x08;
}

/// Conversion types for properties
pub mod conversion {
    pub const NONE: u32 = 0;
    pub const DISTANCE: u32 = 1;
    pub const SPEED: u32 = 2;
    pub const MASS: u32 = 3;
    pub const TIME: u32 = 4;
    pub const ANGLE: u32 = 5;
    pub const TEMPERATURE: u32 = 6;
    pub const POWER: u32 = 7;
    pub const FORCE: u32 = 8;
    pub const CURRENCY: u32 = 9;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_type_size() {
        assert_eq!(DataType::Boolean.size(), Some(1));
        assert_eq!(DataType::Int32.size(), Some(4));
        assert_eq!(DataType::Float.size(), Some(4));
        assert_eq!(DataType::Vec3.size(), Some(12));
        assert_eq!(DataType::Array(Box::new(DataType::Int32)).size(), None);
    }
    
    #[test]
    fn test_data_type_from_u32() {
        assert_eq!(DataType::from_u32(0), DataType::Boolean);
        assert_eq!(DataType::from_u32(10), DataType::Float);
        
        // Array types
        let array_type = DataType::from_u32(0x80000004);
        match array_type {
            DataType::Array(inner) => assert_eq!(*inner, DataType::Int32),
            _ => panic!("Expected Array type"),
        }
    }
    
    #[test]
    fn test_struct_def_property_indices() {
        let s = StructDef {
            id: 0,
            name: "Test".to_string(),
            parent_id: None,
            property_start: 5,
            property_count: 3,
            size: 24,
            flags: 0,
        };
        
        assert_eq!(s.property_indices(), 5..8);
    }
}