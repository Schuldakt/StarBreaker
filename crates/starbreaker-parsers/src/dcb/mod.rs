// stardust-parsers/src/dcb/mod.rs
//! DataCore Binary (DCB) Parser
//!
//! The DCB format is Star Citizen's binary data format that stores all game
//! entity definitions, item stats, ship configurations, and other game data.
//! It uses a structure similar to CryXml but with a custom binary encoding.
//!
//! # Format Structure
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    DCB File Structure                       │
//! ├─────────────────────────────────────────────────────────────┤
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                    Header (36 bytes)                    ││
//! │  │  - Magic: 0x44434231 ("DCB1")                           ││
//! │  │  - Version                                              ││
//! │  │  - Section counts & offsets                             ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                   String Table                          ││
//! │  │  - All strings used in the file                         ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │               Structure Definitions                     ││
//! │  │  - Type definitions for data structures                 ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                  Property Definitions                   ││
//! │  │  - Property names and types                             ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                    Record Data                          ││
//! │  │  - Actual data records                                  ││
//! │  └─────────────────────────────────────────────────────────┘│
//! │  ┌─────────────────────────────────────────────────────────┐│
//! │  │                 Reference Data                          ││
//! │  │  - Cross-references between records                     ││
//! │  └─────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────┘
//! ```

mod datacore;
mod records;
mod structs;
mod cryxml;

pub use datacore::{DataCore, DataCoreHeader};
pub use records::{Record, RecordValue, RecordRef};
pub use structs::{StructDef, PropertyDef, DataType};

use std::io::{Read, Seek, SeekFrom, BufReader};
use std::collections::HashMap;
use std::sync::Arc;

use crate::traits::{
    Parser, ParseResult, ParseError,
    ParseOptions, ParseProgress, ParsePhase, ProgressCallback
};

/// DCB file magic bytes
const DCB_MAGIC: &[u8] = &[0x44, 0x43, 0x42, 0x31]; // "DCB1"

/// Alternate CryXml magic (for older formats)
const CRYXML_MAGIC: &[u8] = &[0x43, 0x72, 0x79, 0x58]; // "CryX"

/// Binary XML magic
const BINXML_MAGIC: u32 = 0x4D584C42; // "BXLM"

/// DataCore Binary parser
pub struct DcbParser {
    /// Cache parsed structures
    cache: parking_lot::RwLock<HashMap<String, Arc<DataCore>>>,
}

impl DcbParser {
    /// Create a new DCB parser
    pub fn new() -> Self {
        Self {
            cache: parking_lot::RwLock::new(HashMap::new()),
        }
    }
    
    /// Parse the file header
    fn parse_header<R: Read + Seek>(&self, reader: &mut R) -> ParseResult<DataCoreHeader> {
        let mut header_data = [0u8; 36];
        reader.read_exact(&mut header_data)?;
        
        // Check magic
        let magic = &header_data[0..4];
        if magic != DCB_MAGIC && magic != CRYXML_MAGIC {
            // Check for binary XML format
            let binxml_magic = u32::from_le_bytes([
                header_data[0], header_data[1], header_data[2], header_data[3]
            ]);
            
            if binxml_magic == BINXML_MAGIC {
                return self.parse_binxml_header(reader, &header_data);
            }
            
            return Err(ParseError::InvalidMagic {
                expected: DCB_MAGIC.to_vec(),
                found: magic.to_vec(),
            });
        }
        
        let version = u32::from_le_bytes([
            header_data[4], header_data[5], header_data[6], header_data[7]
        ]);
        
        let struct_count = u32::from_le_bytes([
            header_data[8], header_data[9], header_data[10], header_data[11]
        ]);
        
        let property_count = u32::from_le_bytes([
            header_data[12], header_data[13], header_data[14], header_data[15]
        ]);
        
        let record_count = u32::from_le_bytes([
            header_data[16], header_data[17], header_data[18], header_data[19]
        ]);
        
        let string_offset = u32::from_le_bytes([
            header_data[20], header_data[21], header_data[22], header_data[23]
        ]) as u64;
        
        let struct_offset = u32::from_le_bytes([
            header_data[24], header_data[25], header_data[26], header_data[27]
        ]) as u64;
        
        let property_offset = u32::from_le_bytes([
            header_data[28], header_data[29], header_data[30], header_data[31]
        ]) as u64;
        
        let record_offset = u32::from_le_bytes([
            header_data[32], header_data[33], header_data[34], header_data[35]
        ]) as u64;
        
        Ok(DataCoreHeader {
            version,
            struct_count,
            property_count,
            record_count,
            string_offset,
            struct_offset,
            property_offset,
            record_offset,
        })
    }
    
    /// Parse binary XML header (alternate format)
    fn parse_binxml_header<R: Read + Seek>(
        &self,
        reader: &mut R,
        initial_data: &[u8; 36]
    ) -> ParseResult<DataCoreHeader> {
        // Binary XML has a different structure
        // Re-read with correct offsets
        reader.seek(SeekFrom::Start(0))?;
        
        let mut header = [0u8; 20];
        reader.read_exact(&mut header)?;
        
        let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
        let node_count = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);
        let attr_count = u32::from_le_bytes([header[12], header[13], header[14], header[15]]);
        
        // For binary XML, we treat nodes as records
        Ok(DataCoreHeader {
            version,
            struct_count: 0,
            property_count: attr_count,
            record_count: node_count,
            string_offset: 20, // Right after header
            struct_offset: 0,
            property_offset: 0,
            record_offset: 0,
        })
    }
    
    /// Parse the string table
    fn parse_string_table<R: Read + Seek>(
        &self,
        reader: &mut R,
        offset: u64,
    ) -> ParseResult<StringTable> {
        reader.seek(SeekFrom::Start(offset))?;
        
        // Read string count
        let mut count_buf = [0u8; 4];
        reader.read_exact(&mut count_buf)?;
        let count = u32::from_le_bytes(count_buf) as usize;
        
        // Read string offsets
        let mut offsets = Vec::with_capacity(count);
        for _ in 0..count {
            let mut offset_buf = [0u8; 4];
            reader.read_exact(&mut offset_buf)?;
            offsets.push(u32::from_le_bytes(offset_buf));
        }
        
        // Read string data
        let data_start = reader.stream_position()?;
        let mut string_data = Vec::new();
        reader.read_to_end(&mut string_data)?;
        
        // Build string map
        let mut strings = Vec::with_capacity(count);
        let mut by_offset = HashMap::new();
        
        for (idx, &str_offset) in offsets.iter().enumerate() {
            let start = str_offset as usize;
            
            // Find null terminator
            let end = string_data[start..]
                .iter()
                .position(|&b| b == 0)
                .map(|p| start + p)
                .unwrap_or(string_data.len());
            
            let s = String::from_utf8_lossy(&string_data[start..end]).to_string();
            by_offset.insert(str_offset, idx);
            strings.push(s);
        }
        
        Ok(StringTable { strings, by_offset })
    }
    
    /// Parse structure definitions
    fn parse_struct_definitions<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &DataCoreHeader,
        strings: &StringTable,
        progress: Option<&ProgressCallback>,
    ) -> ParseResult<Vec<StructDef>> {
        reader.seek(SeekFrom::Start(header.struct_offset))?;
        
        let mut structs = Vec::with_capacity(header.struct_count as usize);
        
        for i in 0..header.struct_count {
            let mut struct_data = [0u8; 24];
            reader.read_exact(&mut struct_data)?;
            
            let name_offset = u32::from_le_bytes([
                struct_data[0], struct_data[1], struct_data[2], struct_data[3]
            ]);
            
            let parent_id = u32::from_le_bytes([
                struct_data[4], struct_data[5], struct_data[6], struct_data[7]
            ]);
            
            let property_start = u32::from_le_bytes([
                struct_data[8], struct_data[9], struct_data[10], struct_data[11]
            ]);
            
            let property_count = u32::from_le_bytes([
                struct_data[12], struct_data[13], struct_data[14], struct_data[15]
            ]);
            
            let size = u32::from_le_bytes([
                struct_data[16], struct_data[17], struct_data[18], struct_data[19]
            ]);
            
            let flags = u32::from_le_bytes([
                struct_data[20], struct_data[21], struct_data[22], struct_data[23]
            ]);
            
            let name = strings.get_by_offset(name_offset)
                .cloned()
                .unwrap_or_else(|| format!("Unknown_{}", i));
            
            structs.push(StructDef {
                id: i,
                name,
                parent_id: if parent_id == 0xFFFFFFFF { None } else { Some(parent_id) },
                property_start,
                property_count,
                size,
                flags,
            });
            
            if let Some(cb) = progress {
                if i % 100 == 0 {
                    cb(ParseProgress {
                        phase: ParsePhase::ParsingRecords,
                        bytes_processed: reader.stream_position()?,
                        total_bytes: None,
                        current_item: Some(format!("Struct: {}", structs.last().unwrap().name)),
                        items_processed: i as u64,
                        total_items: Some(header.struct_count as u64),
                    });
                }
            }
        }
        
        Ok(structs)
    }
    
    /// Parse property definitions
    fn parse_property_definitions<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &DataCoreHeader,
        strings: &StringTable,
    ) -> ParseResult<Vec<PropertyDef>> {
        reader.seek(SeekFrom::Start(header.property_offset))?;
        
        let mut properties = Vec::with_capacity(header.property_count as usize);
        
        for i in 0..header.property_count {
            let mut prop_data = [0u8; 16];
            reader.read_exact(&mut prop_data)?;
            
            let name_offset = u32::from_le_bytes([
                prop_data[0], prop_data[1], prop_data[2], prop_data[3]
            ]);
            
            let data_type = u32::from_le_bytes([
                prop_data[4], prop_data[5], prop_data[6], prop_data[7]
            ]);
            
            let struct_id = u32::from_le_bytes([
                prop_data[8], prop_data[9], prop_data[10], prop_data[11]
            ]);
            
            let conversion = u32::from_le_bytes([
                prop_data[12], prop_data[13], prop_data[14], prop_data[15]
            ]);
            
            let name = strings.get_by_offset(name_offset)
                .cloned()
                .unwrap_or_else(|| format!("prop_{}", i));
            
            properties.push(PropertyDef {
                id: i,
                name,
                data_type: DataType::from_u32(data_type),
                struct_id: if struct_id == 0xFFFFFFFF { None } else { Some(struct_id) },
                conversion,
            });
        }
        
        Ok(properties)
    }
    
    /// Parse records
    fn parse_records<R: Read + Seek>(
        &self,
        reader: &mut R,
        header: &DataCoreHeader,
        strings: &StringTable,
        structs: &[StructDef],
        properties: &[PropertyDef],
        progress: Option<&ProgressCallback>,
    ) -> ParseResult<Vec<Record>> {
        reader.seek(SeekFrom::Start(header.record_offset))?;
        
        let mut records = Vec::with_capacity(header.record_count as usize);
        
        for i in 0..header.record_count {
            // Each record has a header followed by property values
            let mut record_header = [0u8; 16];
            reader.read_exact(&mut record_header)?;
            
            let struct_id = u32::from_le_bytes([
                record_header[0], record_header[1], record_header[2], record_header[3]
            ]);
            
            let name_offset = u32::from_le_bytes([
                record_header[4], record_header[5], record_header[6], record_header[7]
            ]);
            
            let guid_lo = u32::from_le_bytes([
                record_header[8], record_header[9], record_header[10], record_header[11]
            ]);
            
            let guid_hi = u32::from_le_bytes([
                record_header[12], record_header[13], record_header[14], record_header[15]
            ]);
            
            let name = strings.get_by_offset(name_offset)
                .cloned()
                .unwrap_or_default();
            
            let guid = ((guid_hi as u64) << 32) | (guid_lo as u64);
            
            // Get struct definition for this record
            let struct_def = structs.get(struct_id as usize);
            
            // Parse property values based on struct definition
            let values = if let Some(sd) = struct_def {
                self.parse_record_values(reader, sd, properties, strings)?
            } else {
                HashMap::new()
            };
            
            records.push(Record {
                id: i,
                struct_id,
                name,
                guid,
                values,
            });
            
            if let Some(cb) = progress {
                if i % 10000 == 0 {
                    cb(ParseProgress {
                        phase: ParsePhase::ParsingRecords,
                        bytes_processed: reader.stream_position()?,
                        total_bytes: None,
                        current_item: Some(format!("Record: {}", records.last().unwrap().name)),
                        items_processed: i as u64,
                        total_items: Some(header.record_count as u64),
                    });
                }
            }
        }
        
        Ok(records)
    }
    
    /// Parse property values for a record
    fn parse_record_values<R: Read + Seek>(
        &self,
        reader: &mut R,
        struct_def: &StructDef,
        properties: &[PropertyDef],
        strings: &StringTable,
    ) -> ParseResult<HashMap<String, RecordValue>> {
        let mut values = HashMap::new();
        
        let start = struct_def.property_start as usize;
        let end = start + struct_def.property_count as usize;
        
        for i in start..end {
            if let Some(prop) = properties.get(i) {
                let value = self.read_value(reader, &prop.data_type, strings)?;
                values.insert(prop.name.clone(), value);
            }
        }
        
        Ok(values)
    }
    
    /// Read a single value based on type
    fn read_value<R: Read>(
        &self,
        reader: &mut R,
        data_type: &DataType,
        strings: &StringTable,
    ) -> ParseResult<RecordValue> {
        Ok(match data_type {
            DataType::Boolean => {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                RecordValue::Boolean(buf[0] != 0)
            }
            
            DataType::Int8 => {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                RecordValue::Int32(buf[0] as i8 as i32)
            }
            
            DataType::Int16 => {
                let mut buf = [0u8; 2];
                reader.read_exact(&mut buf)?;
                RecordValue::Int32(i16::from_le_bytes(buf) as i32)
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
            
            DataType::UInt8 => {
                let mut buf = [0u8; 1];
                reader.read_exact(&mut buf)?;
                RecordValue::UInt32(buf[0] as u32)
            }
            
            DataType::UInt16 => {
                let mut buf = [0u8; 2];
                reader.read_exact(&mut buf)?;
                RecordValue::UInt32(u16::from_le_bytes(buf) as u32)
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
                let s = strings.get_by_offset(offset).cloned().unwrap_or_default();
                RecordValue::String(s)
            }
            
            DataType::Guid => {
                let mut buf = [0u8; 16];
                reader.read_exact(&mut buf)?;
                RecordValue::Guid(buf)
            }
            
            DataType::Reference => {
                let mut buf = [0u8; 8];
                reader.read_exact(&mut buf)?;
                let record_id = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let struct_id = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                RecordValue::Reference(RecordRef { record_id, struct_id })
            }
            
            DataType::Vec3 => {
                let mut buf = [0u8; 12];
                reader.read_exact(&mut buf)?;
                let x = f32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let y = f32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
                let z = f32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
                RecordValue::Vec3([x, y, z])
            }
            
            DataType::Enum => {
                let mut buf = [0u8; 4];
                reader.read_exact(&mut buf)?;
                RecordValue::Enum(u32::from_le_bytes(buf))
            }
            
            DataType::Array(_) => {
                // Array handling - read count first
                let mut count_buf = [0u8; 4];
                reader.read_exact(&mut count_buf)?;
                let count = u32::from_le_bytes(count_buf) as usize;
                
                // For now, return as bytes
                RecordValue::Array(vec![])
            }
            
            DataType::Unknown(type_id) => {
                RecordValue::Unknown(*type_id)
            }
        })
    }
}

impl Default for DcbParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for DcbParser {
    type Output = DataCore;
    
    fn extensions(&self) -> &[&str] {
        &["dcb"]
    }
    
    fn magic_bytes(&self) -> Option<&[u8]> {
        Some(DCB_MAGIC)
    }
    
    fn name(&self) -> &str {
        "DataCore Binary Parser"
    }
    
    fn parse_with_options<R: Read + Seek>(
        &self,
        mut reader: R,
        options: &ParseOptions,
        progress: Option<ProgressCallback>,
    ) -> ParseResult<Self::Output> {
        // Report start
        if let Some(ref cb) = progress {
            cb(ParseProgress {
                phase: ParsePhase::ReadingHeader,
                bytes_processed: 0,
                total_bytes: None,
                current_item: None,
                items_processed: 0,
                total_items: None,
            });
        }
        
        // Parse header
        let header = self.parse_header(&mut reader)?;
        
        // Parse string table
        let strings = self.parse_string_table(&mut reader, header.string_offset)?;
        
        // Parse struct definitions
        let structs = self.parse_struct_definitions(
            &mut reader,
            &header,
            &strings,
            progress.as_ref()
        )?;
        
        // Parse property definitions
        let properties = self.parse_property_definitions(&mut reader, &header, &strings)?;
        
        // Parse records
        let records = self.parse_records(
            &mut reader,
            &header,
            &strings,
            &structs,
            &properties,
            progress.as_ref()
        )?;
        
        // Build indices
        let mut struct_index = HashMap::new();
        for (idx, s) in structs.iter().enumerate() {
            struct_index.insert(s.name.clone(), idx);
        }
        
        let mut record_index = HashMap::new();
        for (idx, r) in records.iter().enumerate() {
            record_index.insert(r.guid, idx);
            if !r.name.is_empty() {
                record_index.insert(r.id as u64, idx);
            }
        }
        
        // Report completion
        if let Some(ref cb) = progress {
            cb(ParseProgress {
                phase: ParsePhase::Complete,
                bytes_processed: reader.stream_position()?,
                total_bytes: None,
                current_item: None,
                items_processed: records.len() as u64,
                total_items: Some(records.len() as u64),
            });
        }
        
        Ok(DataCore {
            header,
            strings,
            structs,
            properties,
            records,
            struct_index,
            record_index,
        })
    }
}

/// String table for DCB file
#[derive(Debug)]
pub struct StringTable {
    /// All strings indexed by ID
    pub strings: Vec<String>,
    /// Offset to ID mapping
    pub by_offset: HashMap<u32, usize>,
}

impl StringTable {
    /// Get string by ID
    pub fn get(&self, id: usize) -> Option<&String> {
        self.strings.get(id)
    }
    
    /// Get string by offset
    pub fn get_by_offset(&self, offset: u32) -> Option<&String> {
        self.by_offset.get(&offset).and_then(|id| self.strings.get(*id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_type_conversion() {
        assert_eq!(DataType::from_u32(0), DataType::Boolean);
        assert_eq!(DataType::from_u32(4), DataType::Int32);
        assert_eq!(DataType::from_u32(8), DataType::Float);
    }
}