// starbreaker-parsers/src/dcb/records.rs
//! Record types and values for DataCore Binary format

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::{StructDef, PropertyDef, StringTable};
use crate::traits::ParseResult;

/// A single data record from the DCB file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Record ID within the file
    pub id: u32,
    /// Struct type ID
    pub struct_id: u32,
    /// Record name (if any)
    pub name: String,
    /// Unique GUID
    pub guid: u64,
    /// Property values
    pub values: HashMap<String, RecordValue>,
}

impl Record {
    /// Get a value by property name
    pub fn get(&self, name: &str) -> Option<&RecordValue> {
        self.values.get(name)
    }
    
    /// Get a string value
    pub fn get_string(&self, name: &str) -> Option<&str> {
        match self.values.get(name)? {
            RecordValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
    
    /// Get an integer value (converts various int types)
    pub fn get_int(&self, name: &str) -> Option<i64> {
        match self.values.get(name)? {
            RecordValue::Int32(v) => Some(*v as i64),
            RecordValue::Int64(v) => Some(*v),
            RecordValue::UInt32(v) => Some(*v as i64),
            RecordValue::UInt64(v) => Some(*v as i64),
            RecordValue::Enum(v) => Some(*v as i64),
            _ => None,
        }
    }
    
    /// Get a float value
    pub fn get_float(&self, name: &str) -> Option<f64> {
        match self.values.get(name)? {
            RecordValue::Float(v) => Some(*v as f64),
            RecordValue::Double(v) => Some(*v),
            RecordValue::Int32(v) => Some(*v as f64),
            RecordValue::UInt32(v) => Some(*v as f64),
            _ => None,
        }
    }
    
    /// Get a boolean value
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.values.get(name)? {
            RecordValue::Boolean(v) => Some(*v),
            RecordValue::Int32(v) => Some(*v != 0),
            RecordValue::UInt32(v) => Some(*v != 0),
            _ => None,
        }
    }
    
    /// Get a reference value
    pub fn get_reference(&self, name: &str) -> Option<&RecordRef> {
        match self.values.get(name)? {
            RecordValue::Reference(r) => Some(r),
            _ => None,
        }
    }
    
    /// Get a Vec3 value
    pub fn get_vec3(&self, name: &str) -> Option<[f32; 3]> {
        match self.values.get(name)? {
            RecordValue::Vec3(v) => Some(*v),
            _ => None,
        }
    }
    
    /// Check if this record has a specific property
    pub fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }
    
    /// Get all property names
    pub fn property_names(&self) -> impl Iterator<Item = &str> {
        self.values.keys().map(|s| s.as_str())
    }
    
    /// Convert to a JSON value for serialization
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "struct_id": self.struct_id,
            "name": self.name,
            "guid": format!("{:016X}", self.guid),
            "values": self.values_to_json()
        })
    }
    
    /// Convert values to JSON
    fn values_to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        
        for (name, value) in &self.values {
            map.insert(name.clone(), value.to_json());
        }
        
        serde_json::Value::Object(map)
    }
}

/// Value types for record properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordValue {
    /// Boolean value
    Boolean(bool),
    /// 32-bit signed integer
    Int32(i32),
    /// 64-bit signed integer
    Int64(i64),
    /// 32-bit unsigned integer
    UInt32(u32),
    /// 64-bit unsigned integer
    UInt64(u64),
    /// 32-bit float
    Float(f32),
    /// 64-bit float
    Double(f64),
    /// String value
    String(String),
    /// GUID value
    Guid([u8; 16]),
    /// Reference to another record
    Reference(RecordRef),
    /// 3D vector
    Vec3([f32; 3]),
    /// 4D vector/quaternion
    Vec4([f32; 4]),
    /// Enumeration value
    Enum(u32),
    /// Array of values
    Array(Vec<RecordValue>),
    /// Locale string with key
    LocaleString { key: String, value: String },
    /// Unknown/unparsed data
    Unknown(u32),
}

impl RecordValue {
    /// Convert to JSON value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            RecordValue::Boolean(v) => serde_json::Value::Bool(*v),
            RecordValue::Int32(v) => serde_json::Value::Number((*v).into()),
            RecordValue::Int64(v) => serde_json::json!(*v),
            RecordValue::UInt32(v) => serde_json::Value::Number((*v).into()),
            RecordValue::UInt64(v) => serde_json::json!(*v),
            RecordValue::Float(v) => serde_json::json!(*v),
            RecordValue::Double(v) => serde_json::json!(*v),
            RecordValue::String(v) => serde_json::Value::String(v.clone()),
            RecordValue::Guid(v) => {
                let hex: String = v.iter().map(|b| format!("{:02X}", b)).collect();
                serde_json::Value::String(hex)
            }
            RecordValue::Reference(r) => serde_json::json!({
                "record_id": r.record_id,
                "struct_id": r.struct_id
            }),
            RecordValue::Vec3(v) => serde_json::json!({
                "x": v[0],
                "y": v[1],
                "z": v[2]
            }),
            RecordValue::Vec4(v) => serde_json::json!({
                "x": v[0],
                "y": v[1],
                "z": v[2],
                "w": v[3]
            }),
            RecordValue::Enum(v) => serde_json::Value::Number((*v).into()),
            RecordValue::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect())
            }
            RecordValue::LocaleString { key, value } => serde_json::json!({
                "key": key,
                "value": value
            }),
            RecordValue::Unknown(type_id) => serde_json::json!({
                "unknown_type": type_id
            }),
        }
    }
    
    /// Get as string, with conversion
    pub fn as_string(&self) -> Option<String> {
        match self {
            RecordValue::String(s) => Some(s.clone()),
            RecordValue::Boolean(b) => Some(b.to_string()),
            RecordValue::Int32(v) => Some(v.to_string()),
            RecordValue::Int64(v) => Some(v.to_string()),
            RecordValue::UInt32(v) => Some(v.to_string()),
            RecordValue::UInt64(v) => Some(v.to_string()),
            RecordValue::Float(v) => Some(v.to_string()),
            RecordValue::Double(v) => Some(v.to_string()),
            RecordValue::Enum(v) => Some(v.to_string()),
            RecordValue::LocaleString { value, .. } => Some(value.clone()),
            _ => None,
        }
    }
    
    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            RecordValue::Boolean(_) => "bool",
            RecordValue::Int32(_) => "i32",
            RecordValue::Int64(_) => "i64",
            RecordValue::UInt32(_) => "u32",
            RecordValue::UInt64(_) => "u64",
            RecordValue::Float(_) => "f32",
            RecordValue::Double(_) => "f64",
            RecordValue::String(_) => "string",
            RecordValue::Guid(_) => "guid",
            RecordValue::Reference(_) => "reference",
            RecordValue::Vec3(_) => "vec3",
            RecordValue::Vec4(_) => "vec4",
            RecordValue::Enum(_) => "enum",
            RecordValue::Array(_) => "array",
            RecordValue::LocaleString { .. } => "locale_string",
            RecordValue::Unknown(_) => "unknown",
        }
    }
}

/// Reference to another record
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RecordRef {
    /// Target record ID
    pub record_id: u32,
    /// Target struct type ID
    pub struct_id: u32,
}

impl RecordRef {
    /// Check if this is a null/empty reference
    pub fn is_null(&self) -> bool {
        self.record_id == 0xFFFFFFFF || self.record_id == 0
    }
}

/// Lightweight record info for searching/filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordInfo {
    /// Record ID
    pub id: u32,
    /// Struct type name
    pub struct_name: String,
    /// Record name
    pub name: String,
    /// GUID
    pub guid: u64,
}

impl From<&Record> for RecordInfo {
    fn from(record: &Record) -> Self {
        Self {
            id: record.id,
            struct_name: String::new(), // Needs to be filled from struct lookup
            name: record.name.clone(),
            guid: record.guid,
        }
    }
}

/// Lazy-loaded record that defers parsing values until accessed
#[derive(Debug, Clone)]
pub struct LazyRecord {
    /// Record metadata (always loaded)
    pub id: u32,
    pub struct_id: u32,
    pub name: String,
    pub guid: u64,
    
    /// File position where record property data starts
    pub(crate) file_offset: u64,
    
    /// Cached values (loaded on first access)
    values: Arc<RwLock<Option<HashMap<String, RecordValue>>>>,
}

impl LazyRecord {
    /// Create a new lazy record
    pub fn new(
        id: u32,
        struct_id: u32,
        name: String,
        guid: u64,
        file_offset: u64,
    ) -> Self {
        Self {
            id,
            struct_id,
            name,
            guid,
            file_offset,
            values: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Check if values are loaded
    pub fn is_loaded(&self) -> bool {
        self.values.read().is_some()
    }
    
    /// Load values from file if not already loaded
    /// This should be called by the DataCore when values are needed
    pub(crate) fn ensure_loaded<F>(
        &self,
        loader: F,
    ) -> ParseResult<()>
    where
        F: FnOnce(u64) -> ParseResult<HashMap<String, RecordValue>>,
    {
        let mut values = self.values.write();
        if values.is_none() {
            *values = Some(loader(self.file_offset)?);
        }
        Ok(())
    }
    
    /// Get a value by property name (loads values if needed)
    pub fn get<F>(&self, name: &str, loader: F) -> ParseResult<Option<RecordValue>>
    where
        F: FnOnce(u64) -> ParseResult<HashMap<String, RecordValue>>,
    {
        self.ensure_loaded(loader)?;
        Ok(self.values.read().as_ref().and_then(|v| v.get(name).cloned()))
    }
    
    /// Get all values (loads if needed)
    pub fn values<F>(&self, loader: F) -> ParseResult<HashMap<String, RecordValue>>
    where
        F: FnOnce(u64) -> ParseResult<HashMap<String, RecordValue>>,
    {
        self.ensure_loaded(loader)?;
        Ok(self.values.read().as_ref().unwrap().clone())
    }
    
    /// Convert to a fully loaded Record
    pub fn to_record<F>(&self, loader: F) -> ParseResult<Record>
    where
        F: FnOnce(u64) -> ParseResult<HashMap<String, RecordValue>>,
    {
        let values = self.values(loader)?;
        Ok(Record {
            id: self.id,
            struct_id: self.struct_id,
            name: self.name.clone(),
            guid: self.guid,
            values,
        })
    }
    
    /// Unload values to free memory
    pub fn unload(&self) {
        *self.values.write() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn make_test_record() -> Record {
        let mut values = HashMap::new();
        values.insert("name".to_string(), RecordValue::String("Test Ship".to_string()));
        values.insert("mass".to_string(), RecordValue::Float(50000.0));
        values.insert("enabled".to_string(), RecordValue::Boolean(true));
        values.insert("health".to_string(), RecordValue::Int32(1000));
        values.insert("position".to_string(), RecordValue::Vec3([1.0, 2.0, 3.0]));
        
        Record {
            id: 1,
            struct_id: 10,
            name: "TestRecord".to_string(),
            guid: 0x123456789ABCDEF0,
            values,
        }
    }
    
    #[test]
    fn test_get_string() {
        let record = make_test_record();
        assert_eq!(record.get_string("name"), Some("Test Ship"));
        assert_eq!(record.get_string("mass"), None);
    }
    
    #[test]
    fn test_get_float() {
        let record = make_test_record();
        assert_eq!(record.get_float("mass"), Some(50000.0));
        assert_eq!(record.get_float("health"), Some(1000.0)); // Converts int
    }
    
    #[test]
    fn test_get_bool() {
        let record = make_test_record();
        assert_eq!(record.get_bool("enabled"), Some(true));
        assert_eq!(record.get_bool("health"), Some(true)); // Non-zero int
    }
    
    #[test]
    fn test_get_vec3() {
        let record = make_test_record();
        assert_eq!(record.get_vec3("position"), Some([1.0, 2.0, 3.0]));
    }
    
    #[test]
    fn test_to_json() {
        let record = make_test_record();
        let json = record.to_json();
        
        assert_eq!(json["name"], "TestRecord");
        assert_eq!(json["values"]["name"], "Test Ship");
    }
    
    #[test]
    fn test_record_ref_is_null() {
        let null_ref = RecordRef { record_id: 0xFFFFFFFF, struct_id: 0 };
        assert!(null_ref.is_null());
        
        let valid_ref = RecordRef { record_id: 123, struct_id: 456 };
        assert!(!valid_ref.is_null());
    }
    
    #[test]
    fn test_lazy_record_creation() {
        let lazy = LazyRecord::new(
            1,
            10,
            "Test".to_string(),
            0x123,
            100,
        );
        
        assert_eq!(lazy.id, 1);
        assert_eq!(lazy.struct_id, 10);
        assert_eq!(lazy.name, "Test");
        assert_eq!(lazy.guid, 0x123);
        assert!(!lazy.is_loaded());
    }
    
    #[test]
    fn test_lazy_record_unload() {
        let lazy = LazyRecord::new(1, 10, "Test".to_string(), 0x123, 100);
        
        // Simulate loading
        let mut test_values = HashMap::new();
        test_values.insert("test".to_string(), RecordValue::Int32(42));
        *lazy.values.write() = Some(test_values);
        
        assert!(lazy.is_loaded());
        
        // Unload
        lazy.unload();
        assert!(!lazy.is_loaded());
    }
}