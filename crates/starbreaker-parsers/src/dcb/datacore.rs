//! DataCore container and header structures

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::Arc;
use parking_lot::Mutex;
use super::{StringTable, StructDef, PropertyDef, Record, LazyRecord, RecordValue, DataType};
use crate::traits::{ParseResult, ParseError};

/// DataCore file header
#[derive(Debug, Clone)]
pub struct DataCoreHeader {
    pub version: u32,
    pub struct_count: u32,
    pub property_count: u32,
    pub record_count: u32,
    pub string_offset: u64,
    pub struct_offset: u64,
    pub property_offset: u64,
    pub record_offset: u64,
}

/// Parsed DataCore database
#[derive(Debug)]
pub struct DataCore {
    pub header: DataCoreHeader,
    pub strings: StringTable,
    pub structs: Vec<StructDef>,
    pub properties: Vec<PropertyDef>,
    pub records: Vec<Record>,
    pub struct_index: HashMap<String, usize>,
    pub record_index: HashMap<u64, usize>,
}

impl DataCore {
    /// Get a record by GUID
    pub fn get_record(&self, guid: u64) -> Option<&Record> {
        self.record_index.get(&guid).map(|&idx| &self.records[idx])
    }
    
    /// Get a record by name
    pub fn get_record_by_name(&self, name: &str) -> Option<&Record> {
        self.records.iter().find(|r| r.name == name)
    }
    
    /// Get a struct definition by name
    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.struct_index.get(name).map(|&idx| &self.structs[idx])
    }
    
    /// Find records by struct type
    pub fn find_by_struct(&self, struct_name: &str) -> Vec<&Record> {
        if let Some(&struct_idx) = self.struct_index.get(struct_name) {
            self.records.iter()
                .filter(|r| r.struct_id == struct_idx as u32)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get total record count
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
    
    /// Get all struct names
    pub fn struct_names(&self) -> Vec<&str> {
        self.structs.iter().map(|s| s.name.as_str()).collect()
    }
}

/// Lazy-loading DataCore that loads records on-demand
#[derive(Debug)]
pub struct LazyDataCore {
    pub header: DataCoreHeader,
    pub strings: Arc<StringTable>,
    pub structs: Arc<Vec<StructDef>>,
    pub properties: Arc<Vec<PropertyDef>>,
    
    /// Lazy records with metadata only
    pub records: Vec<LazyRecord>,
    
    /// Indices for quick lookup
    pub struct_index: HashMap<String, usize>,
    pub record_index: HashMap<u64, usize>,
    
    /// File path for lazy loading (if loaded from file)
    file_path: Option<PathBuf>,
    
    /// Shared file handle for lazy loading
    file_handle: Arc<Mutex<Option<std::fs::File>>>,
}

impl LazyDataCore {
    /// Create a new lazy datacore
    pub fn new(
        header: DataCoreHeader,
        strings: StringTable,
        structs: Vec<StructDef>,
        properties: Vec<PropertyDef>,
        records: Vec<LazyRecord>,
        struct_index: HashMap<String, usize>,
        record_index: HashMap<u64, usize>,
        file_path: Option<PathBuf>,
    ) -> Self {
        Self {
            header,
            strings: Arc::new(strings),
            structs: Arc::new(structs),
            properties: Arc::new(properties),
            records,
            struct_index,
            record_index,
            file_path: file_path.clone(),
            file_handle: Arc::new(Mutex::new(
                file_path.and_then(|p| std::fs::File::open(p).ok())
            )),
        }
    }
    
    /// Get a lazy record by GUID
    pub fn get_record(&self, guid: u64) -> Option<&LazyRecord> {
        self.record_index.get(&guid).map(|&idx| &self.records[idx])
    }
    
    /// Get a lazy record by name
    pub fn get_record_by_name(&self, name: &str) -> Option<&LazyRecord> {
        self.records.iter().find(|r| r.name == name)
    }
    
    /// Get a struct definition by name
    pub fn get_struct(&self, name: &str) -> Option<&StructDef> {
        self.struct_index.get(name).map(|&idx| &self.structs[idx])
    }
    
    /// Load a specific record's values
    pub fn load_record(&self, record: &LazyRecord) -> ParseResult<HashMap<String, RecordValue>> {
        let loader = |offset: u64| self.load_record_values(offset, record.struct_id);
        record.values(loader)
    }
    
    /// Load record values from file
    fn load_record_values(
        &self,
        offset: u64,
        struct_id: u32,
    ) -> ParseResult<HashMap<String, RecordValue>> {
        let mut file = self.file_handle.lock();
        let file_ref = file.as_mut().ok_or_else(|| {
            ParseError::InvalidStructure("No file handle available for lazy loading".to_string())
        })?;
        
        // Seek to record data
        file_ref.seek(SeekFrom::Start(offset))?;
        
        // Get struct definition
        let struct_def = self.structs.get(struct_id as usize).ok_or_else(|| {
            ParseError::InvalidStructure(format!("Invalid struct ID: {}", struct_id))
        })?;
        
        // Parse property values
        let mut values = HashMap::new();
        let start = struct_def.property_start as usize;
        let end = start + struct_def.property_count as usize;
        
        for i in start..end {
            if let Some(prop) = self.properties.get(i) {
                let value = self.read_value(file_ref, &prop.data_type)?;
                values.insert(prop.name.clone(), value);
            }
        }
        
        Ok(values)
    }
    
    /// Read a single value from the file
    fn read_value<R: Read>(
        &self,
        reader: &mut R,
        data_type: &DataType,
    ) -> ParseResult<RecordValue> {
        Ok(match data_type {
            DataType::Boolean => {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                RecordValue::Boolean(buf[0] != 0)
            }
            DataType::Int32 => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                RecordValue::Int32(i32::from_le_bytes(buf))
            }
            DataType::Int64 => {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                RecordValue::Int64(i64::from_le_bytes(buf))
            }
            DataType::UInt32 => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                RecordValue::UInt32(u32::from_le_bytes(buf))
            }
            DataType::UInt64 => {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                RecordValue::UInt64(u64::from_le_bytes(buf))
            }
            DataType::Float => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                RecordValue::Float(f32::from_le_bytes(buf))
            }
            DataType::Double => {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                RecordValue::Double(f64::from_le_bytes(buf))
            }
            DataType::String => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                let offset = u32::from_le_bytes(buf);
                let s = self.strings.get_by_offset(offset)
                    .cloned()
                    .unwrap_or_default();
                RecordValue::String(s)
            }
            DataType::Vec3 => {
                let mut buf = [0u8; 12];
                reader.read_exact(&mut buf)?;
                let x = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let y = f32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                let z = f32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
                RecordValue::Vec3([x, y, z])
            }
            DataType::Vec4 => {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf)?;
                let x = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let y = f32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                let z = f32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
                let w = f32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]);
                RecordValue::Vec4([x, y, z, w])
            }
            _ => RecordValue::Unknown(0),
        })
    }
    
    /// Find records by struct type (returns lazy records)
    pub fn find_by_struct(&self, struct_name: &str) -> Vec<&LazyRecord> {
        if let Some(&struct_idx) = self.struct_index.get(struct_name) {
            self.records.iter()
                .filter(|r| r.struct_id == struct_idx as u32)
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get total record count
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
    
    /// Get all struct names
    pub fn struct_names(&self) -> Vec<&str> {
        self.structs.iter().map(|s| s.name.as_str()).collect()
    }
    
    /// Convert to a fully-loaded DataCore (loads all records)
    pub fn to_eager(&self) -> ParseResult<DataCore> {
        let mut records = Vec::new();
        
        for lazy_record in &self.records {
            let loader = |offset: u64| self.load_record_values(offset, lazy_record.struct_id);
            records.push(lazy_record.to_record(loader)?);
        }
        
        Ok(DataCore {
            header: self.header.clone(),
            strings: (*self.strings).clone(),
            structs: (*self.structs).clone(),
            properties: (*self.properties).clone(),
            records,
            struct_index: self.struct_index.clone(),
            record_index: self.record_index.clone(),
        })
    }
    
    /// Unload all cached record values to free memory
    pub fn unload_all(&self) {
        for record in &self.records {
            record.unload();
        }
    }
}