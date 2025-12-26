//! VFS mount point abstraction

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::io::Cursor;
use std::sync::Arc;
use crate::path;
use crate::node::VfsNode;
use starbreaker_parsers::{P4kArchive, P4kCompression, P4kEntry};

/// Result type for mount operations
pub type MountResult<T> = Result<T, MountError>;

/// Mount operation errors
#[derive(Debug, thiserror::Error)]
pub enum MountError {
    #[error("Mount point not found: {0}")]
    NotFound(String),
    
    #[error("Path not found: {path}")]
    PathNotFound { path: String },
    
    #[error("Access denied: {path}")]
    AccessDenied { path: String },
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    
    #[error("Already mounted: {0}")]
    AlreadyMounted(String),
}

/// Mount point trait
/// Provides abstraction over different file sources (archives, filesystem, virtual)
pub trait MountPoint: Send + Sync {
    /// Get mount point unique ID
    fn id(&self) -> usize;
    
    /// Get mount point name/label
    fn name(&self) -> &str;
    
    /// Get mount priority (higher = checked first)
    fn priority(&self) -> i32 {
        0
    }
    
    /// Check if path exists in this mount
    fn exists(&self, path: &str) -> bool;
    
    /// Get node metadata for a path
    fn get_node(&self, path: &str) -> MountResult<VfsNode>;
    
    /// List all nodes in a directory
    fn list_directory(&self, path: &str) -> MountResult<Vec<VfsNode>>;
    
    /// Open file for reading
    fn open_file(&self, path: &str) -> MountResult<Box<dyn Read + Send>>;
    
    /// Get total file count
    fn file_count(&self) -> usize;
    
    /// Get total size in bytes
    fn total_size(&self) -> u64;
    
    /// Check if this mount is read-only
    fn is_readonly(&self) -> bool {
        true
    }
}

/// P4K archive mount point
pub struct P4kMount {
    id: usize,
    name: String,
    archive_path: PathBuf,
    archive: Arc<P4kArchive>,
}

impl P4kMount {
    /// Create a new P4K archive mount
    pub fn new(
        id: usize,
        name: impl Into<String>,
        archive_path: impl AsRef<Path>,
        archive: Arc<P4kArchive>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            archive_path: archive_path.as_ref().to_path_buf(),
            archive,
        }
    }

    /// Normalize and trim VFS path to archive-relative form
    fn normalize_path(&self, path: &str) -> String {
        let normalized = path::normalize_path(path);
        normalized.trim_start_matches('/').trim_end_matches('/').to_string()
    }

    /// Lookup an entry by normalized VFS path
    fn find_entry(&self, path: &str) -> Option<&P4kEntry> {
        if path.is_empty() {
            return None;
        }

        if let Some(entry) = self.archive.get(path) {
            return Some(entry);
        }

        // Some directory entries include a trailing slash
        if !path.ends_with('/') {
            let mut with_slash = String::with_capacity(path.len() + 1);
            with_slash.push_str(path);
            with_slash.push('/');
            if let Some(entry) = self.archive.get(&with_slash) {
                return Some(entry);
            }
        }

        None
    }

    /// Convert a P4K entry to a VFS node
    fn entry_to_node(&self, entry: &P4kEntry) -> VfsNode {
        if entry.is_directory {
            VfsNode::new_directory(entry.filename(), self.id)
        } else {
            VfsNode::new_file(entry.filename(), entry.uncompressed_size, self.id)
        }
    }

    /// Read and decompress a P4K entry into memory
    fn read_entry_data(&self, entry: &P4kEntry) -> MountResult<Vec<u8>> {
        const LOCAL_HEADER_SIGNATURE: u32 = 0x0403_4B50;

        let mut file = std::fs::File::open(&self.archive_path)?;
        file.seek(SeekFrom::Start(entry.local_header_offset))?;

        let mut local_header = [0u8; 30];
        file.read_exact(&mut local_header)?;

        let sig = u32::from_le_bytes([
            local_header[0],
            local_header[1],
            local_header[2],
            local_header[3],
        ]);

        if sig != LOCAL_HEADER_SIGNATURE {
            return Err(MountError::InvalidPath(format!(
                "Invalid local header signature for {}", entry.path
            )));
        }

        let name_len = u16::from_le_bytes([local_header[26], local_header[27]]) as u64;
        let extra_len = u16::from_le_bytes([local_header[28], local_header[29]]) as u64;

        // Skip filename and extra fields to reach data
        file.seek(SeekFrom::Current((name_len + extra_len) as i64))?;

        let mut compressed = vec![0u8; entry.compressed_size as usize];
        file.read_exact(&mut compressed)?;

        let data = P4kCompression::decompress(
            &compressed,
            entry.compression,
            entry.uncompressed_size as usize,
        )
        .map_err(|e| MountError::InvalidPath(e.to_string()))?;

        if !P4kCompression::verify_crc32(&data, entry.crc32) {
            return Err(MountError::InvalidPath(format!(
                "CRC mismatch for {}", entry.path
            )));
        }

        Ok(data)
    }
}

impl MountPoint for P4kMount {
    fn id(&self) -> usize {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn exists(&self, _path: &str) -> bool {
        let rel = self.normalize_path(_path);
        rel.is_empty() || self.find_entry(&rel).is_some()
    }
    
    fn get_node(&self, path: &str) -> MountResult<VfsNode> {
        let rel = self.normalize_path(path);

        if rel.is_empty() {
            return Ok(VfsNode::new_directory("/", self.id));
        }

        if let Some(entry) = self.find_entry(&rel) {
            Ok(self.entry_to_node(entry))
        } else {
            Err(MountError::PathNotFound { path: path.to_string() })
        }
    }
    
    fn list_directory(&self, path: &str) -> MountResult<Vec<VfsNode>> {
        let rel = self.normalize_path(path);
        let entries = self.archive.list_directory(&rel);

        if entries.is_empty() {
            return Err(MountError::PathNotFound { path: path.to_string() });
        }

        Ok(entries.into_iter().map(|e| self.entry_to_node(e)).collect())
    }
    
    fn open_file(&self, path: &str) -> MountResult<Box<dyn Read + Send>> {
        let rel = self.normalize_path(path);

        let entry = self.find_entry(&rel)
            .ok_or_else(|| MountError::PathNotFound { path: path.to_string() })?;

        if entry.is_directory {
            return Err(MountError::AccessDenied { path: path.to_string() });
        }

        let data = self.read_entry_data(entry)?;
        Ok(Box::new(Cursor::new(data)))
    }
    
    fn file_count(&self) -> usize {
        self.archive.file_count()
    }
    
    fn total_size(&self) -> u64 {
        self.archive.total_uncompressed_size()
    }
}

/// Local filesystem mount point
pub struct FilesystemMount {
    id: usize,
    name: String,
    root_path: PathBuf,
}

impl FilesystemMount {
    /// Create a new filesystem mount
    pub fn new(id: usize, name: impl Into<String>, root_path: impl AsRef<Path>) -> MountResult<Self> {
        let root_path = root_path.as_ref().to_path_buf();
        
        if !root_path.exists() {
            return Err(MountError::NotFound(root_path.display().to_string()));
        }
        
        if !root_path.is_dir() {
            return Err(MountError::InvalidPath(
                format!("{} is not a directory", root_path.display())
            ));
        }
        
        Ok(Self {
            id,
            name: name.into(),
            root_path,
        })
    }
    
    /// Get absolute path from VFS path
    fn resolve_path(&self, path: &str) -> PathBuf {
        let path = path.trim_start_matches('/');
        self.root_path.join(path)
    }
}

impl MountPoint for FilesystemMount {
    fn id(&self) -> usize {
        self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn exists(&self, path: &str) -> bool {
        self.resolve_path(path).exists()
    }
    
    fn get_node(&self, path: &str) -> MountResult<VfsNode> {
        let abs_path = self.resolve_path(path);
        
        if !abs_path.exists() {
            return Err(MountError::PathNotFound { path: path.to_string() });
        }
        
        let metadata = std::fs::metadata(&abs_path)?;
        let name = abs_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        if metadata.is_dir() {
            Ok(VfsNode::new_directory(name, self.id))
        } else {
            Ok(VfsNode::new_file(name, metadata.len(), self.id))
        }
    }
    
    fn list_directory(&self, path: &str) -> MountResult<Vec<VfsNode>> {
        let abs_path = self.resolve_path(path);
        
        if !abs_path.exists() {
            return Err(MountError::PathNotFound { path: path.to_string() });
        }
        
        if !abs_path.is_dir() {
            return Err(MountError::InvalidPath(
                format!("{} is not a directory", path)
            ));
        }
        
        let mut nodes = Vec::new();
        
        for entry in std::fs::read_dir(&abs_path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            let node = if metadata.is_dir() {
                VfsNode::new_directory(name, self.id)
            } else {
                VfsNode::new_file(name, metadata.len(), self.id)
            };
            
            nodes.push(node);
        }
        
        Ok(nodes)
    }
    
    fn open_file(&self, path: &str) -> MountResult<Box<dyn Read + Send>> {
        let abs_path = self.resolve_path(path);
        
        if !abs_path.exists() {
            return Err(MountError::PathNotFound { path: path.to_string() });
        }
        
        let file = std::fs::File::open(&abs_path)?;
        Ok(Box::new(file))
    }
    
    fn file_count(&self) -> usize {
        // Recursive count would be expensive, return 0 for now
        0
    }
    
    fn total_size(&self) -> u64 {
        // Recursive sum would be expensive, return 0 for now
        0
    }
}
