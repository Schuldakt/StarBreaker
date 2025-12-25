//! DDS split file combiner
//!
//! Star Citizen splits large DDS textures across multiple files with extensions
//! like .dds.1, .dds.2, .dds.3a, .dds.3b, etc. This module combines them back
//! into a single texture.

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use crate::traits::{ParseResult, ParseError, Parser};
use super::{DdsTexture, DdsParser, DDS_MAGIC};

/// DDS split file combiner
pub struct DdsCombiner {
    parser: DdsParser,
}

impl DdsCombiner {
    /// Create a new combiner
    pub fn new() -> Self {
        Self {
            parser: DdsParser::new(),
        }
    }

    /// Detect if a path refers to a split DDS file
    /// 
    /// Examples:
    /// - texture.dds.1 -> true
    /// - texture.dds.2 -> true
    /// - texture.dds.3a -> true
    /// - texture.dds -> false
    pub fn is_split_file<P: AsRef<Path>>(path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        
        // Check for .dds.N pattern
        if let Some(dds_pos) = path_str.rfind(".dds.") {
            let suffix = &path_str[dds_pos + 5..];
            // Must have numeric or numeric+alpha suffix
            return !suffix.is_empty() && suffix.chars().next().unwrap().is_numeric();
        }
        
        false
    }

    /// Get base path without split suffix
    /// 
    /// Example: "texture.dds.1" -> "texture.dds"
    pub fn get_base_path<P: AsRef<Path>>(path: P) -> PathBuf {
        let path_str = path.as_ref().to_string_lossy();
        
        if let Some(dds_pos) = path_str.rfind(".dds.") {
            PathBuf::from(&path_str[..dds_pos + 4])
        } else {
            path.as_ref().to_path_buf()
        }
    }

    /// Find all split files for a given base path
    /// 
    /// Looks for files like base.dds.1, base.dds.2, etc.
    pub fn find_split_files<P: AsRef<Path>>(base_path: P) -> Vec<PathBuf> {
        let base = base_path.as_ref();
        let parent = base.parent().unwrap_or(Path::new("."));
        let base_name = base.file_name().unwrap().to_string_lossy();
        
        let mut split_files = Vec::new();
        
        // Try common split patterns
        for i in 1..=99 {
            // Try .dds.N
            let mut candidate = parent.join(format!("{}.{}", base_name, i));
            if candidate.exists() {
                split_files.push(candidate);
                continue;
            }
            
            // Try .dds.Na and .dds.Nb for mipmap levels
            candidate = parent.join(format!("{}.{}a", base_name, i));
            if candidate.exists() {
                split_files.push(candidate);
                
                // Check for 'b' variant
                let b_candidate = parent.join(format!("{}.{}b", base_name, i));
                if b_candidate.exists() {
                    split_files.push(b_candidate);
                }
            }
        }
        
        split_files.sort();
        split_files
    }

    /// Combine split DDS files into a single texture
    /// 
    /// # Arguments
    /// * `path` - Path to any split file (e.g., texture.dds.1) or the base file
    /// 
    /// # Returns
    /// Combined DDS texture with data from all split files
    pub fn combine<P: AsRef<Path>>(&self, path: P) -> ParseResult<DdsTexture> {
        let path_ref = path.as_ref();
        
        // Get base path
        let base_path = if Self::is_split_file(path_ref) {
            Self::get_base_path(path_ref)
        } else {
            path_ref.to_path_buf()
        };

        // Find all split files
        let split_files = Self::find_split_files(&base_path);

        if split_files.is_empty() {
            // No split files found, try to parse as regular DDS
            let file = File::open(path_ref)?;
            let mut texture = self.parser.parse_with_options(
                file,
                &crate::traits::ParseOptions::default(),
                None
            )?;
            texture.was_split = false;
            return Ok(texture);
        }

        // Parse header from first split file
        let mut first_file = File::open(&split_files[0])?;
        
        // Read magic
        let mut magic_buf = [0u8; 4];
        first_file.read_exact(&mut magic_buf)?;
        let magic = u32::from_le_bytes(magic_buf);

        if magic != DDS_MAGIC {
            return Err(ParseError::InvalidMagic {
                expected: DDS_MAGIC.to_le_bytes().to_vec(),
                found: magic_buf.to_vec(),
            });
        }

        // Parse headers
        let header = super::header::DdsHeader::parse(&mut first_file)?;
        let dx10_header = if header.has_dx10_header() {
            Some(super::header::DX10Header::parse(&mut first_file)?)
        } else {
            None
        };

        let format = super::format::TextureFormat::from_header(&header, dx10_header.as_ref());

        // Combine data from all split files
        let mut combined_data = Vec::new();

        // Read remaining data from first file
        let mut first_data = Vec::new();
        first_file.read_to_end(&mut first_data)?;
        combined_data.extend_from_slice(&first_data);

        // Read data from subsequent split files
        for split_path in split_files.iter().skip(1) {
            let mut file = File::open(split_path)?;
            
            // Each split file is just raw data (no header)
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            combined_data.extend_from_slice(&data);
        }

        Ok(DdsTexture {
            header,
            dx10_header,
            data: combined_data,
            format,
            was_split: true,
        })
    }

    /// Combine split files from a list of paths
    /// 
    /// Useful when you already know the split file paths
    pub fn combine_from_paths(&self, paths: &[PathBuf]) -> ParseResult<DdsTexture> {
        if paths.is_empty() {
            return Err(ParseError::InvalidStructure(
                "No paths provided to combine".to_string()
            ));
        }

        // Sort paths to ensure correct order
        let mut sorted_paths = paths.to_vec();
        sorted_paths.sort();

        // Parse header from first file
        let mut first_file = File::open(&sorted_paths[0])?;
        
        // Read magic
        let mut magic_buf = [0u8; 4];
        first_file.read_exact(&mut magic_buf)?;
        let magic = u32::from_le_bytes(magic_buf);

        if magic != DDS_MAGIC {
            return Err(ParseError::InvalidMagic {
                expected: DDS_MAGIC.to_le_bytes().to_vec(),
                found: magic_buf.to_vec(),
            });
        }

        // Parse headers
        let header = super::header::DdsHeader::parse(&mut first_file)?;
        let dx10_header = if header.has_dx10_header() {
            Some(super::header::DX10Header::parse(&mut first_file)?)
        } else {
            None
        };

        let format = super::format::TextureFormat::from_header(&header, dx10_header.as_ref());

        // Combine data
        let mut combined_data = Vec::new();

        // Read first file data
        first_file.seek(SeekFrom::Start(0))?;
        first_file.read_exact(&mut magic_buf)?; // Re-read magic
        
        // Skip header
        let header_size = 4 + 124 + if dx10_header.is_some() { 20 } else { 0 };
        first_file.seek(SeekFrom::Start(header_size as u64))?;
        
        let mut first_data = Vec::new();
        first_file.read_to_end(&mut first_data)?;
        combined_data.extend_from_slice(&first_data);

        // Read subsequent files
        for path in sorted_paths.iter().skip(1) {
            let mut file = File::open(path)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            combined_data.extend_from_slice(&data);
        }

        Ok(DdsTexture {
            header,
            dx10_header,
            data: combined_data,
            format,
            was_split: true,
        })
    }
}

impl Default for DdsCombiner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_split_file() {
        assert!(DdsCombiner::is_split_file("texture.dds.1"));
        assert!(DdsCombiner::is_split_file("texture.dds.2"));
        assert!(DdsCombiner::is_split_file("texture.dds.3a"));
        assert!(DdsCombiner::is_split_file("path/to/texture.dds.10"));
        
        assert!(!DdsCombiner::is_split_file("texture.dds"));
        assert!(!DdsCombiner::is_split_file("texture.png"));
        assert!(!DdsCombiner::is_split_file("texture"));
    }

    #[test]
    fn test_get_base_path() {
        assert_eq!(
            DdsCombiner::get_base_path("texture.dds.1"),
            PathBuf::from("texture.dds")
        );
        assert_eq!(
            DdsCombiner::get_base_path("path/to/texture.dds.3a"),
            PathBuf::from("path/to/texture.dds")
        );
        assert_eq!(
            DdsCombiner::get_base_path("texture.dds"),
            PathBuf::from("texture.dds")
        );
    }
}
