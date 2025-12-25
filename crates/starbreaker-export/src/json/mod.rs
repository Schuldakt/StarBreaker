//! JSON data export for game assets
//!
//! Exports DCB records, CGF metadata, and P4K indices to JSON format.

use starbreaker_parsers::dcb::DataCore;
use starbreaker_parsers::p4k::P4kArchive;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::BufWriter;
use thiserror::Error;

/// JSON export errors
#[derive(Error, Debug)]
pub enum JsonError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Export failed: {0}")]
    ExportFailed(String),
}

pub type JsonResult<T> = Result<T, JsonError>;

/// JSON export options
#[derive(Debug, Clone)]
pub struct JsonExportOptions {
    /// Use pretty-print formatting
    pub pretty: bool,
    
    /// Include metadata (file info, counts, etc.)
    pub include_metadata: bool,
    
    /// Maximum nesting depth for arrays/objects
    pub max_depth: usize,
}

impl Default for JsonExportOptions {
    fn default() -> Self {
        Self {
            pretty: true,
            include_metadata: true,
            max_depth: 10,
        }
    }
}

/// JSON data exporter
pub struct JsonExporter {
    options: JsonExportOptions,
}

impl JsonExporter {
    /// Create new exporter with default options
    pub fn new() -> Self {
        Self {
            options: JsonExportOptions::default(),
        }
    }
    
    /// Create exporter with custom options
    pub fn with_options(options: JsonExportOptions) -> Self {
        Self { options }
    }
    
    /// Export DataCore records to JSON
    /// Records are already in a user-friendly format with Record::to_json()
    pub fn export_datacore(&self, datacore: &DataCore, output_path: impl AsRef<Path>) -> JsonResult<()> {
        // Group records by struct type
        let mut by_struct: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
        
        for record in &datacore.records {
            let struct_name = datacore.structs.get(record.struct_id as usize)
                .map(|s| s.name.clone())
                .unwrap_or_else(|| format!("Unknown_{:08X}", record.struct_id));
            
            let record_json = record.to_json();
            
            by_struct.entry(struct_name)
                .or_insert_with(Vec::new)
                .push(record_json);
        }
        
        // Build output JSON
        let output = if self.options.include_metadata {
            json!({
                "metadata": {
                    "version": datacore.header.version,
                    "record_count": datacore.records.len(),
                    "struct_count": datacore.structs.len(),
                },
                "structs": by_struct,
            })
        } else {
            json!(by_struct)
        };
        
        self.write_json(&output, output_path)?;
        
        Ok(())
    }
    
    /*
    /// Export CGF mesh metadata to JSON
    /// Includes vertex counts, material info, bounding boxes
    /// 
    /// Note: Currently commented out as CgfFile structure needs to be defined
    pub fn export_cgf_metadata(&self, cgf: &CgfFile, output_path: impl AsRef<Path>) -> JsonResult<()> {
        let mut meshes = Vec::new();
        
        for mesh in &cgf.meshes {
            meshes.push(json!({
                "vertex_count": mesh.vertices.len(),
                "face_count": mesh.indices.len() / 3,
                "has_normals": !mesh.normals.is_empty(),
                "has_uvs": !mesh.uvs.is_empty(),
                "has_colors": !mesh.colors.is_empty(),
                "has_tangents": !mesh.tangents.is_empty(),
                "bounds": {
                    "min": mesh.bounds_min,
                    "max": mesh.bounds_max,
                }
            }));
        }
        
        let output = if self.options.include_metadata {
            json!({
                "metadata": {
                    "version": cgf.header.version,
                    "chunk_count": cgf.header.chunk_count,
                    "mesh_count": meshes.len(),
                },
                "meshes": meshes,
            })
        } else {
            json!({ "meshes": meshes })
        };
        
        self.write_json(&output, output_path)?;
        
        Ok(())
    }
    */
    
    /// Export P4K archive index to JSON
    /// Lists all files with sizes and compression info
    pub fn export_p4k_index(&self, archive: &P4kArchive, output_path: impl AsRef<Path>) -> JsonResult<()> {
        let mut entries = Vec::new();
        
        for entry in &archive.entries {
            entries.push(json!({
                "path": entry.path,
                "uncompressed_size": entry.uncompressed_size,
                "compressed_size": entry.compressed_size,
                "compression": format!("{:?}", entry.compression),
                "is_directory": entry.is_directory,
            }));
        }
        
        let output = if self.options.include_metadata {
            json!({
                "metadata": {
                    "entry_count": archive.entry_count(),
                    "file_count": archive.file_count(),
                    "directory_count": archive.directory_count(),
                    "total_uncompressed_size": archive.total_uncompressed_size(),
                    "total_compressed_size": archive.total_compressed_size(),
                },
                "entries": entries,
            })
        } else {
            json!(entries)
        };
        
        self.write_json(&output, output_path)?;
        
        Ok(())
    }
    
    /// Write JSON to file
    fn write_json(&self, value: &serde_json::Value, output_path: impl AsRef<Path>) -> JsonResult<()> {
        let file = File::create(output_path)?;
        let writer = BufWriter::new(file);
        
        if self.options.pretty {
            serde_json::to_writer_pretty(writer, value)?;
        } else {
            serde_json::to_writer(writer, value)?;
        }
        
        Ok(())
    }
}

impl Default for JsonExporter {
    fn default() -> Self {
        Self::new()
    }
}
