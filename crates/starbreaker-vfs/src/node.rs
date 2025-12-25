//! VFS node structures

use serde::{Deserialize, Serialize};

/// VFS node type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    /// Directory node
    Directory,
    /// File node
    File,
    /// Symbolic link
    Symlink,
}

/// VFS node representing a file or directory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsNode {
    /// Node name (without path)
    pub name: String,
    /// Node type
    pub node_type: NodeType,
    /// File size (0 for directories)
    pub size: u64,
    /// Mount point ID this node belongs to
    pub mount_id: usize,
    /// Offset within the mount source (for archive files)
    pub offset: Option<u64>,
    /// Compressed size (if different from size)
    pub compressed_size: Option<u64>,
    /// Metadata
    pub metadata: NodeMetadata,
}

/// Node metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// File extension (if file)
    pub extension: Option<String>,
    /// MIME type
    pub mime_type: Option<String>,
    /// Compression method
    pub compression: Option<String>,
    /// CRC32 checksum
    pub crc32: Option<u32>,
    /// MD5 hash
    pub md5: Option<String>,
    /// Custom tags
    pub tags: Vec<String>,
}

impl VfsNode {
    /// Create a new file node
    pub fn new_file(name: impl Into<String>, size: u64, mount_id: usize) -> Self {
        let name = name.into();
        let extension = std::path::Path::new(&name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        Self {
            name,
            node_type: NodeType::File,
            size,
            mount_id,
            offset: None,
            compressed_size: None,
            metadata: NodeMetadata {
                extension,
                ..Default::default()
            },
        }
    }

    /// Create a new directory node
    pub fn new_directory(name: impl Into<String>, mount_id: usize) -> Self {
        Self {
            name: name.into(),
            node_type: NodeType::Directory,
            size: 0,
            mount_id,
            offset: None,
            compressed_size: None,
            metadata: Default::default(),
        }
    }

    /// Check if this is a file
    pub fn is_file(&self) -> bool {
        self.node_type == NodeType::File
    }

    /// Check if this is a directory
    pub fn is_directory(&self) -> bool {
        self.node_type == NodeType::Directory
    }

    /// Check if this is a symlink
    pub fn is_symlink(&self) -> bool {
        self.node_type == NodeType::Symlink
    }

    /// Get the file extension
    pub fn extension(&self) -> Option<&str> {
        self.metadata.extension.as_deref()
    }

    /// Check if file has a specific extension (case-insensitive)
    pub fn has_extension(&self, ext: &str) -> bool {
        self.extension()
            .map(|e| e.eq_ignore_ascii_case(ext))
            .unwrap_or(false)
    }

    /// Check if this node is compressed
    pub fn is_compressed(&self) -> bool {
        self.compressed_size.is_some() && 
        self.compressed_size != Some(self.size)
    }

    /// Get compression ratio (0.0-1.0, lower = better compression)
    pub fn compression_ratio(&self) -> Option<f64> {
        if let Some(compressed) = self.compressed_size {
            if self.size > 0 {
                return Some(compressed as f64 / self.size as f64);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_file() {
        let node = VfsNode::new_file("test.txt", 1024, 0);
        assert!(node.is_file());
        assert_eq!(node.name, "test.txt");
        assert_eq!(node.size, 1024);
        assert_eq!(node.extension(), Some("txt"));
    }

    #[test]
    fn test_new_directory() {
        let node = VfsNode::new_directory("folder", 0);
        assert!(node.is_directory());
        assert_eq!(node.name, "folder");
        assert_eq!(node.size, 0);
    }

    #[test]
    fn test_has_extension() {
        let node = VfsNode::new_file("model.CGF", 2048, 0);
        assert!(node.has_extension("cgf"));
        assert!(node.has_extension("CGF"));
        assert!(!node.has_extension("dds"));
    }

    #[test]
    fn test_compression_ratio() {
        let mut node = VfsNode::new_file("data.bin", 1000, 0);
        node.compressed_size = Some(500);
        
        assert!(node.is_compressed());
        assert_eq!(node.compression_ratio(), Some(0.5));
    }
}
