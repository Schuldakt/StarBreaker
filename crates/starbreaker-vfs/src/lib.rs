//! Virtual File System Core
//!
//! Provides a unified interface for accessing files across different storage backends
//! including local filesystem, P4K archives, and DCB virtual folders.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use thiserror::Error;

pub mod mounts;

pub use mounts::p4k::P4kMountPoint;

/// VFS errors
#[derive(Error, Debug)]
pub enum VfsError {
    #[error("File or directory not found: {0}")]
    NotFound(PathBuf),

    #[error("Path is not a file: {0}")]
    NotAFile(PathBuf),

    #[error("Path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    #[error("Read-only filesystem")]
    ReadOnly,

    #[error("Mount error: {0}")]
    MountError(String),

    #[error("No mount point for path: {0}")]
    NoMountPoint(PathBuf),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Result type for VFS operations
pub type VfsResult<T> = Result<T, VfsError>;

/// A node in the virtual filesystem
#[derive(Debug, Clone)]
pub struct VfsNode {
    /// Full path of the node
    pub path: PathBuf,
    /// Node name (filename or directory name)
    pub name: String,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Size in bytes (for files)
    pub size: u64,
    /// Compressed size (if applicable)
    pub compressed_size: Option<u64>,
    /// Last modification time (if available)
    pub modified: Option<std::time::SystemTime>,
}

/// Entry returned when listing directories
#[derive(Debug, Clone)]
pub struct VfsEntry {
    /// Entry name
    pub name: String,
    /// Full path
    pub path: PathBuf,
    /// Whether this is a directory
    pub is_directory: bool,
    /// Size in bytes (for files)
    pub size: Option<u64>,
    /// Compressed size (if applicable)
    pub compressed_size: Option<u64>,
}

/// Trait for mount point implementations
pub trait MountPoint: Send + Sync {
    /// Get the mount path
    fn mount_path(&self) -> &Path;

    /// Check if the mount is read-only
    fn is_read_only(&self) -> bool;

    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;

    /// Check if a path is a file
    fn is_file(&self, path: &Path) -> bool;

    /// Check if a path is a directory
    fn is_directory(&self, path: &Path) -> bool;

    /// Read file contents
    fn read(&self, path: &Path) -> VfsResult<Vec<u8>>;

    /// Read file as string
    fn read_to_string(&self, path: &Path) -> VfsResult<String>;

    /// List directory contents
    fn list(&self, path: &Path) -> VfsResult<Vec<VfsEntry>>;

    /// Get file/directory metadata
    fn metadata(&self, path: &Path) -> VfsResult<VfsNode>;

    /// Find files matching a pattern
    fn find(&self, pattern: &str) -> VfsResult<Vec<PathBuf>>;

    /// Write file contents (optional, returns error for read-only mounts)
    fn write(&self, _path: &Path, _data: &[u8]) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }

    /// Create a directory (optional, returns error for read-only mounts)
    fn create_dir(&self, _path: &Path) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }

    /// Delete a file or directory (optional, returns error for read-only mounts)
    fn delete(&self, _path: &Path) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }
}

/// The Virtual File System
pub struct Vfs {
    /// Registered mount points, sorted by path length (longest first)
    mounts: RwLock<Vec<Arc<dyn MountPoint>>>,
}

impl Vfs {
    /// Create a new VFS instance
    pub fn new() -> Self {
        Self {
            mounts: RwLock::new(Vec::new()),
        }
    }

    /// Mount a new mount point
    pub fn mount(&self, mount: impl MountPoint + 'static) -> VfsResult<()> {
        let mount = Arc::new(mount);
        let mut mounts = self.mounts.write();
        
        // Check for conflicts
        let new_path = mount.mount_path();
        for existing in mounts.iter() {
            let existing_path = existing.mount_path();
            if new_path.starts_with(existing_path) || existing_path.starts_with(new_path) {
                return Err(VfsError::MountError(format!(
                    "Mount path conflict: {} vs {}",
                    new_path.display(),
                    existing_path.display()
                )));
            }
        }

        mounts.push(mount);
        
        // Sort by path length (longest first) for correct matching
        mounts.sort_by(|a, b| {
            b.mount_path().as_os_str().len().cmp(&a.mount_path().as_os_str().len())
        });

        Ok(())
    }

    /// Unmount a mount point by path
    pub fn unmount(&self, path: &Path) -> VfsResult<()> {
        let mut mounts = self.mounts.write();
        let initial_len = mounts.len();
        
        mounts.retain(|m| m.mount_path() != path);
        
        if mounts.len() == initial_len {
            Err(VfsError::NoMountPoint(path.to_path_buf()))
        } else {
            Ok(())
        }
    }

    /// Get the mount point for a path
    fn get_mount(&self, path: &Path) -> Option<Arc<dyn MountPoint>> {
        let mounts = self.mounts.read();
        for mount in mounts.iter() {
            if path.starts_with(mount.mount_path()) {
                return Some(Arc::clone(mount));
            }
        }
        None
    }

    /// Check if a path exists
    pub fn exists(&self, path: &Path) -> bool {
        self.get_mount(path)
            .map(|m| m.exists(path))
            .unwrap_or(false)
    }

    /// Check if a path is a file
    pub fn is_file(&self, path: &Path) -> bool {
        self.get_mount(path)
            .map(|m| m.is_file(path))
            .unwrap_or(false)
    }

    /// Check if a path is a directory
    pub fn is_directory(&self, path: &Path) -> bool {
        self.get_mount(path)
            .map(|m| m.is_directory(path))
            .unwrap_or(false)
    }

    /// Read file contents
    pub fn read(&self, path: &Path) -> VfsResult<Vec<u8>> {
        self.get_mount(path)
            .ok_or_else(|| VfsError::NoMountPoint(path.to_path_buf()))?
            .read(path)
    }

    /// Read file as string
    pub fn read_to_string(&self, path: &Path) -> VfsResult<String> {
        self.get_mount(path)
            .ok_or_else(|| VfsError::NoMountPoint(path.to_path_buf()))?
            .read_to_string(path)
    }

    /// List directory contents
    pub fn list(&self, path: &Path) -> VfsResult<Vec<VfsEntry>> {
        self.get_mount(path)
            .ok_or_else(|| VfsError::NoMountPoint(path.to_path_buf()))?
            .list(path)
    }

    /// Get file/directory metadata
    pub fn metadata(&self, path: &Path) -> VfsResult<VfsNode> {
        self.get_mount(path)
            .ok_or_else(|| VfsError::NoMountPoint(path.to_path_buf()))?
            .metadata(path)
    }

    /// Find files matching a pattern across all mounts
    pub fn find(&self, pattern: &str) -> VfsResult<Vec<PathBuf>> {
        let mounts = self.mounts.read();
        let mut results = Vec::new();
        
        for mount in mounts.iter() {
            if let Ok(found) = mount.find(pattern) {
                results.extend(found);
            }
        }
        
        Ok(results)
    }

    /// Write file contents
    pub fn write(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        self.get_mount(path)
            .ok_or_else(|| VfsError::NoMountPoint(path.to_path_buf()))?
            .write(path, data)
    }

    /// Create a directory
    pub fn create_dir(&self, path: &Path) -> VfsResult<()> {
        self.get_mount(path)
            .ok_or_else(|| VfsError::NoMountPoint(path.to_path_buf()))?
            .create_dir(path)
    }

    /// Delete a file or directory
    pub fn delete(&self, path: &Path) -> VfsResult<()> {
        self.get_mount(path)
            .ok_or_else(|| VfsError::NoMountPoint(path.to_path_buf()))?
            .delete(path)
    }

    /// List all mount points
    pub fn list_mounts(&self) -> Vec<MountInfo> {
        self.mounts.read()
            .iter()
            .map(|m| MountInfo {
                path: m.mount_path().to_path_buf(),
                read_only: m.is_read_only(),
            })
            .collect()
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a mounted filesystem
#[derive(Debug, Clone)]
pub struct MountInfo {
    /// Mount path
    pub path: PathBuf,
    /// Whether the mount is read-only
    pub read_only: bool,
}

/// Local filesystem mount point
pub struct LocalMount {
    root: PathBuf,
    mount_path: PathBuf,
    read_only: bool,
}

impl LocalMount {
    /// Create a new local filesystem mount
    pub fn new(root: impl AsRef<Path>, mount_path: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            mount_path: mount_path.as_ref().to_path_buf(),
            read_only: false,
        }
    }

    /// Create a read-only local mount
    pub fn read_only(root: impl AsRef<Path>, mount_path: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            mount_path: mount_path.as_ref().to_path_buf(),
            read_only: true,
        }
    }

    fn resolve_path(&self, vfs_path: &Path) -> Option<PathBuf> {
        let relative = vfs_path.strip_prefix(&self.mount_path).ok()?;
        Some(self.root.join(relative))
    }
}

impl MountPoint for LocalMount {
    fn mount_path(&self) -> &Path {
        &self.mount_path
    }

    fn is_read_only(&self) -> bool {
        self.read_only
    }

    fn exists(&self, path: &Path) -> bool {
        self.resolve_path(path)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    fn is_file(&self, path: &Path) -> bool {
        self.resolve_path(path)
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    fn is_directory(&self, path: &Path) -> bool {
        self.resolve_path(path)
            .map(|p| p.is_dir())
            .unwrap_or(false)
    }

    fn read(&self, path: &Path) -> VfsResult<Vec<u8>> {
        let real_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;
        std::fs::read(real_path).map_err(VfsError::from)
    }

    fn read_to_string(&self, path: &Path) -> VfsResult<String> {
        let real_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;
        std::fs::read_to_string(real_path).map_err(VfsError::from)
    }

    fn list(&self, path: &Path) -> VfsResult<Vec<VfsEntry>> {
        let real_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;

        let entries = std::fs::read_dir(real_path)?;
        let mut results = Vec::new();

        for entry in entries {
            let entry = entry?;
            let metadata = entry.metadata()?;
            
            results.push(VfsEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: path.join(entry.file_name()),
                is_directory: metadata.is_dir(),
                size: if metadata.is_file() { Some(metadata.len()) } else { None },
                compressed_size: None,
            });
        }

        Ok(results)
    }

    fn metadata(&self, path: &Path) -> VfsResult<VfsNode> {
        let real_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;

        let metadata = std::fs::metadata(&real_path)?;
        
        Ok(VfsNode {
            path: path.to_path_buf(),
            name: real_path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            is_directory: metadata.is_dir(),
            size: metadata.len(),
            compressed_size: None,
            modified: metadata.modified().ok(),
        })
    }

    fn find(&self, pattern: &str) -> VfsResult<Vec<PathBuf>> {
        // Simple recursive search
        let mut results = Vec::new();
        self.find_recursive(&self.root, &self.mount_path, pattern, &mut results)?;
        Ok(results)
    }

    fn write(&self, path: &Path, data: &[u8]) -> VfsResult<()> {
        if self.read_only {
            return Err(VfsError::ReadOnly);
        }

        let real_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::InvalidPath(path.display().to_string()))?;

        if let Some(parent) = real_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(real_path, data).map_err(VfsError::from)
    }

    fn create_dir(&self, path: &Path) -> VfsResult<()> {
        if self.read_only {
            return Err(VfsError::ReadOnly);
        }

        let real_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::InvalidPath(path.display().to_string()))?;

        std::fs::create_dir_all(real_path).map_err(VfsError::from)
    }

    fn delete(&self, path: &Path) -> VfsResult<()> {
        if self.read_only {
            return Err(VfsError::ReadOnly);
        }

        let real_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;

        if real_path.is_dir() {
            std::fs::remove_dir_all(real_path)?;
        } else {
            std::fs::remove_file(real_path)?;
        }

        Ok(())
    }
}

impl LocalMount {
    fn find_recursive(
        &self,
        dir: &Path,
        vfs_base: &Path,
        pattern: &str,
        results: &mut Vec<PathBuf>,
    ) -> VfsResult<()> {
        let pattern_lower = pattern.to_lowercase();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_lowercase();

            // Simple pattern matching
            let matches = if pattern.contains('*') {
                let parts: Vec<&str> = pattern_lower.split('*').collect();
                if parts.len() == 2 {
                    (parts[0].is_empty() || name.starts_with(parts[0])) &&
                    (parts[1].is_empty() || name.ends_with(parts[1]))
                } else {
                    name.contains(&pattern_lower)
                }
            } else {
                name.contains(&pattern_lower)
            };

            if matches {
                let relative = path.strip_prefix(&self.root)
                    .unwrap_or(&path);
                results.push(vfs_base.join(relative));
            }

            if path.is_dir() {
                let new_vfs_base = vfs_base.join(entry.file_name());
                self.find_recursive(&path, &new_vfs_base, pattern, results)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        
        // Create test structure
        fs::create_dir_all(dir.path().join("subdir")).unwrap();
        fs::write(dir.path().join("file1.txt"), "hello").unwrap();
        fs::write(dir.path().join("file2.txt"), "world").unwrap();
        fs::write(dir.path().join("subdir/nested.txt"), "nested").unwrap();
        
        dir
    }

    #[test]
    fn test_local_mount_exists() {
        let dir = setup_test_dir();
        let mount = LocalMount::new(dir.path(), "/test");
        
        assert!(mount.exists(Path::new("/test/file1.txt")));
        assert!(mount.exists(Path::new("/test/subdir")));
        assert!(!mount.exists(Path::new("/test/nonexistent")));
    }

    #[test]
    fn test_local_mount_read() {
        let dir = setup_test_dir();
        let mount = LocalMount::new(dir.path(), "/test");
        
        let content = mount.read_to_string(Path::new("/test/file1.txt")).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_local_mount_list() {
        let dir = setup_test_dir();
        let mount = LocalMount::new(dir.path(), "/test");
        
        let entries = mount.list(Path::new("/test")).unwrap();
        assert_eq!(entries.len(), 3); // file1.txt, file2.txt, subdir
    }

    #[test]
    fn test_vfs_mount_and_read() {
        let dir = setup_test_dir();
        let vfs = Vfs::new();
        
        let mount = LocalMount::new(dir.path(), "/data");
        vfs.mount(mount).unwrap();
        
        assert!(vfs.exists(Path::new("/data/file1.txt")));
        let content = vfs.read_to_string(Path::new("/data/file1.txt")).unwrap();
        assert_eq!(content, "hello");
    }

    #[test]
    fn test_vfs_multiple_mounts() {
        let dir1 = setup_test_dir();
        let dir2 = setup_test_dir();
        let vfs = Vfs::new();
        
        vfs.mount(LocalMount::new(dir1.path(), "/mount1")).unwrap();
        vfs.mount(LocalMount::new(dir2.path(), "/mount2")).unwrap();
        
        assert!(vfs.exists(Path::new("/mount1/file1.txt")));
        assert!(vfs.exists(Path::new("/mount2/file1.txt")));
    }

    #[test]
    fn test_read_only_mount() {
        let dir = setup_test_dir();
        let mount = LocalMount::read_only(dir.path(), "/test");
        
        let result = mount.write(Path::new("/test/new.txt"), b"data");
        assert!(matches!(result, Err(VfsError::ReadOnly)));
    }
}