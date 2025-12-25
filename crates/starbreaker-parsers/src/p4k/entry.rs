// starbreaker-parsers/src/p4k/entry.rs
//! P4K archive entry structures

use super::CompressionMethod;
use serde::{Deserialize, Serialize};

/// Represents a single file entry in a P4K archive
#[derive(Debug, Clone)]
pub struct P4kEntry {
    /// Full path within the archive
    pub path: String,
    /// Compression method used
    pub compression: CompressionMethod,
    /// CRC32 checksum of uncompressed data
    pub crc32: u32,
    /// Size of compressed data
    pub compressed_size: u64,
    /// Size of uncompressed data
    pub uncompressed_size: u64,
    /// Offset to local file header
    pub local_header_offset: u64,
    /// General purpose bit flags
    pub flags: u16,
    /// DOS modification time
    pub mod_time: u16,
    /// DOS modification date
    pub mod_date: u16,
    /// Whether entry is encrypted
    pub is_encrypted: bool,
    /// Whether entry is a directory
    pub is_directory: bool,
}

impl P4kEntry {
    /// Get the filename without path
    pub fn filename(&self) -> &str {
        self.path
            .rsplit('/')
            .next()
            .unwrap_or(&self.path)
    }

    /// Get the parent directory path
    pub fn parent(&self) -> Option<&str> {
        let path = self.path.trim_end_matches('/');
        path.rfind('/').map(|idx| &path[..idx])
    }

    /// Get the file extension
    pub fn extension(&self) -> Option<&str> {
        if self.is_directory {
            return None;
        }

        let filename = self.filename();
        filename.rfind('.').map(|idx| &filename[idx + 1..])
    }

    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.uncompressed_size == 0 {
            return 1.0;
        }
        self.compressed_size as f64 / self.uncompressed_size as f64
    }

    /// Parse DOS date/time to components
    pub fn modification_datetime(&self) -> (u16, u8, u8, u8, u8, u8) {
        let year = 1980 + ((self.mod_date >> 9) & 0x7F);
        let month = ((self.mod_date >> 5) & 0x0F) as u8;
        let day = (self.mod_date & 0x1F) as u8;
        let hour = ((self.mod_time >> 11) & 0x1F) as u8;
        let minute = ((self.mod_time >> 5) & 0x3F) as u8;
        let second = ((self.mod_time & 0x1F) * 2) as u8;

        (year, month, day, hour, minute, second)
    }
}

/// Lightweight entry info for listing/searching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4kEntryInfo {
    /// Full path within the archive
    pub path: String,
    /// Size of compressed data
    pub compressed_size: u64,
    /// Size of uncompressed data
    pub uncompressed_size: u64,
    /// Whether entry is a directory
    pub is_directory: bool,
    /// Compression method used
    #[serde(skip)]
    pub compression: CompressionMethod,
}

impl P4kEntryInfo {
    /// Format size as human-readable string
    pub fn formatted_size(&self) -> String {
        format_bytes(self.uncompressed_size)
    }

    /// Format compressed size as human-readable string
    pub fn formatted_compressed_size(&self) -> String {
        format_bytes(self.compressed_size)
    }
}

/// Format byte count as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_entry(path: &str) -> P4kEntry {
        P4kEntry {
            path: path.to_string(),
            compression: CompressionMethod::Store,
            crc32: 0,
            compressed_size: 100,
            uncompressed_size: 100,
            local_header_offset: 0,
            flags: 0,
            mod_time: 0,
            mod_date: 0,
            is_encrypted: false,
            is_directory: path.ends_with('/'),
        }
    }

    #[test]
    fn test_filename() {
        let entry = make_test_entry("Data/Libs/Config/defaultprofile.xml");
        assert_eq!(entry.filename(), "defaultprofile.xml");

        let entry = make_test_entry("Data/");
        assert_eq!(entry.filename(), "Data");
    }

    #[test]
    fn test_parent() {
        let entry = make_test_entry("Data/Libs/Config/defaultprofile.xml");
        assert_eq!(entry.parent(), Some("Data/Libs/Config"));

        let entry = make_test_entry("Data");
        assert_eq!(entry.parent(), None);
    }

    #[test]
    fn test_extension() {
        let entry = make_test_entry("texture.dds.1");
        assert_eq!(entry.extension(), Some("1"));

        let entry = make_test_entry("model.cgf");
        assert_eq!(entry.extension(), Some("cgf"));

        let entry = make_test_entry("noextension");
        assert_eq!(entry.extension(), None);

        let entry = make_test_entry("Data/");
        assert_eq!(entry.extension(), None);
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }
}