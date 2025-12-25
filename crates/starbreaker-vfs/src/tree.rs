//! VFS tree implementation

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::mount::{MountPoint, MountResult, MountError};
use crate::node::VfsNode;
use crate::path;

/// Virtual File System tree
/// Manages multiple mount points and provides unified file access
pub struct VfsTree {
    /// Mounted file sources (ordered by priority)
    mounts: RwLock<Vec<Arc<dyn MountPoint>>>,
    /// Mount point cache
    mount_cache: RwLock<HashMap<usize, Arc<dyn MountPoint>>>,
}

impl VfsTree {
    /// Create a new empty VFS tree
    pub fn new() -> Self {
        Self {
            mounts: RwLock::new(Vec::new()),
            mount_cache: RwLock::new(HashMap::new()),
        }
    }

    /// Add a mount point
    pub fn add_mount(&self, mount: Arc<dyn MountPoint>) {
        let id = mount.id();
        
        let mut mounts = self.mounts.write().unwrap();
        mounts.push(Arc::clone(&mount));
        
        // Sort by priority (descending)
        mounts.sort_by(|a, b| b.priority().cmp(&a.priority()));
        
        let mut cache = self.mount_cache.write().unwrap();
        cache.insert(id, mount);
    }

    /// Remove a mount point by ID
    pub fn remove_mount(&self, id: usize) -> bool {
        let mut mounts = self.mounts.write().unwrap();
        let initial_len = mounts.len();
        
        mounts.retain(|m| m.id() != id);
        
        if mounts.len() < initial_len {
            let mut cache = self.mount_cache.write().unwrap();
            cache.remove(&id);
            true
        } else {
            false
        }
    }

    /// Get mount point by ID
    pub fn get_mount(&self, id: usize) -> Option<Arc<dyn MountPoint>> {
        let cache = self.mount_cache.read().unwrap();
        cache.get(&id).cloned()
    }

    /// List all mount points
    pub fn list_mounts(&self) -> Vec<Arc<dyn MountPoint>> {
        let mounts = self.mounts.read().unwrap();
        mounts.clone()
    }

    /// Check if a path exists in any mount
    pub fn exists(&self, path: &str) -> bool {
        let normalized = path::normalize_path(path);
        let mounts = self.mounts.read().unwrap();
        
        for mount in mounts.iter() {
            if mount.exists(&normalized) {
                return true;
            }
        }
        
        false
    }

    /// Get node metadata for a path
    /// Searches mounts in priority order
    pub fn get_node(&self, path: &str) -> MountResult<VfsNode> {
        let normalized = path::normalize_path(path);
        let mounts = self.mounts.read().unwrap();
        
        for mount in mounts.iter() {
            if let Ok(node) = mount.get_node(&normalized) {
                return Ok(node);
            }
        }
        
        Err(MountError::PathNotFound { path: normalized })
    }

    /// List directory contents
    /// Merges results from all mounts
    pub fn list_directory(&self, path: &str) -> MountResult<Vec<VfsNode>> {
        let normalized = path::normalize_path(path);
        let mounts = self.mounts.read().unwrap();
        
        let mut all_nodes = HashMap::new();
        let mut found_any = false;
        
        for mount in mounts.iter() {
            if let Ok(nodes) = mount.list_directory(&normalized) {
                found_any = true;
                for node in nodes {
                    // Keep highest priority version of each file
                    all_nodes.entry(node.name.clone()).or_insert(node);
                }
            }
        }
        
        if found_any {
            Ok(all_nodes.into_values().collect())
        } else {
            Err(MountError::PathNotFound { path: normalized })
        }
    }

    /// Open file for reading
    /// Searches mounts in priority order
    pub fn open_file(&self, path: &str) -> MountResult<Box<dyn std::io::Read + Send>> {
        let normalized = path::normalize_path(path);
        let mounts = self.mounts.read().unwrap();
        
        for mount in mounts.iter() {
            if let Ok(reader) = mount.open_file(&normalized) {
                return Ok(reader);
            }
        }
        
        Err(MountError::PathNotFound { path: normalized })
    }

    /// Search for files matching a glob pattern
    pub fn search_glob(&self, _pattern: &str) -> Vec<(String, VfsNode)> {
        let results = Vec::new();
        
        // This is a simplified implementation
        // A full implementation would recursively traverse all mounts
        // For now, just return empty results
        
        results
    }

    /// Get total file count across all mounts
    pub fn total_file_count(&self) -> usize {
        let mounts = self.mounts.read().unwrap();
        mounts.iter().map(|m| m.file_count()).sum()
    }

    /// Get total size across all mounts
    pub fn total_size(&self) -> u64 {
        let mounts = self.mounts.read().unwrap();
        mounts.iter().map(|m| m.total_size()).sum()
    }

    /// Get number of mounted sources
    pub fn mount_count(&self) -> usize {
        let mounts = self.mounts.read().unwrap();
        mounts.len()
    }

    /// Read entire file contents into memory
    pub fn read_file_to_vec(&self, path: &str) -> MountResult<Vec<u8>> {
        let mut reader = self.open_file(path)?;
        let mut buffer = Vec::new();
        std::io::Read::read_to_end(&mut reader, &mut buffer)
            .map_err(MountError::Io)?;
        Ok(buffer)
    }

    /// Read file contents as a string
    pub fn read_file_to_string(&self, path: &str) -> MountResult<String> {
        let bytes = self.read_file_to_vec(path)?;
        String::from_utf8(bytes)
            .map_err(|_| MountError::InvalidPath("File is not valid UTF-8".to_string()))
    }

    /// Extract a file to the local filesystem
    pub fn extract_file(&self, vfs_path: &str, output_path: impl AsRef<std::path::Path>) -> MountResult<u64> {
        let mut reader = self.open_file(vfs_path)?;
        let mut file = std::fs::File::create(output_path)
            .map_err(MountError::Io)?;
        
        std::io::copy(&mut reader, &mut file)
            .map_err(MountError::Io)
    }

    /// Extract multiple files to a directory
    /// Returns (success_count, total_bytes_written)
    pub fn extract_batch(&self, file_list: &[String], output_dir: impl AsRef<std::path::Path>) -> (usize, u64) {
        let output_dir = output_dir.as_ref();
        let mut success_count = 0;
        let mut total_bytes = 0u64;

        for vfs_path in file_list {
            // Create output path preserving directory structure
            let relative_path = vfs_path.trim_start_matches('/');
            let output_path = output_dir.join(relative_path);

            // Create parent directories
            if let Some(parent) = output_path.parent() {
                if let Err(_) = std::fs::create_dir_all(parent) {
                    continue;
                }
            }

            // Extract file
            if let Ok(bytes) = self.extract_file(vfs_path, &output_path) {
                success_count += 1;
                total_bytes += bytes;
            }
        }

        (success_count, total_bytes)
    }

    /// Extract all files from a directory recursively
    pub fn extract_directory(&self, vfs_dir: &str, output_dir: impl AsRef<std::path::Path>) -> MountResult<(usize, u64)> {
        let output_dir = output_dir.as_ref();
        
        // Get all files in directory (this is simplified - real implementation would be recursive)
        let nodes = self.list_directory(vfs_dir)?;
        
        let mut file_paths = Vec::new();
        for node in nodes {
            if node.is_file() {
                let file_path = path::join_paths(vfs_dir, &node.name);
                file_paths.push(file_path);
            }
        }

        Ok(self.extract_batch(&file_paths, output_dir))
    }
}

impl Default for VfsTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mount::FilesystemMount;
    use std::sync::Arc;

    #[test]
    fn test_vfs_new() {
        let vfs = VfsTree::new();
        assert_eq!(vfs.mount_count(), 0);
    }

    #[test]
    fn test_add_remove_mount() {
        let vfs = VfsTree::new();
        
        // Create temp directory for testing
        let temp_dir = std::env::temp_dir().join("vfs_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        
        let mount = Arc::new(FilesystemMount::new(1, "test", &temp_dir).unwrap());
        vfs.add_mount(mount);
        
        assert_eq!(vfs.mount_count(), 1);
        
        assert!(vfs.remove_mount(1));
        assert_eq!(vfs.mount_count(), 0);
        
        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
