//! VFS mount point abstraction

use std::io::Read;
use std::path::{Path, PathBuf};
use crate::node::VfsNode;

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
    // Will store parsed P4K data when implemented
}

impl P4kMount {
    /// Create a new P4K archive mount
    pub fn new(id: usize, name: impl Into<String>, archive_path: impl AsRef<Path>) -> Self {
        Self {
            id,
            name: name.into(),
            archive_path: archive_path.as_ref().to_path_buf(),
        }
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
        // TODO: implement with P4kArchive
        false
    }
    
    fn get_node(&self, path: &str) -> MountResult<VfsNode> {
        // TODO: implement with P4kArchive
        Err(MountError::PathNotFound { path: path.to_string() })
    }
    
    fn list_directory(&self, path: &str) -> MountResult<Vec<VfsNode>> {
        // TODO: implement with P4kArchive
        Err(MountError::PathNotFound { path: path.to_string() })
    }
    
    fn open_file(&self, path: &str) -> MountResult<Box<dyn Read + Send>> {
        // TODO: implement with P4kArchive
        Err(MountError::PathNotFound { path: path.to_string() })
    }
    
    fn file_count(&self) -> usize {
        // TODO: implement with P4kArchive
        0
    }
    
    fn total_size(&self) -> u64 {
        // TODO: implement with P4kArchive
        0
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
