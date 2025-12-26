//! P4K Archive Mount Point
//!
//! Provides a virtual filesystem mount for P4K archives, allowing transparent
//! access to archive contents as if they were regular filesystem paths.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use thiserror::Error;

use starbreaker_parsers::p4k::{P4kArchive, P4kEntry, P4kParser, DirectoryNode};
use starbreaker_parsers::traits::{Parser, RandomAccessParser};

use crate::{VfsNode, VfsEntry, VfsError, VfsResult, MountPoint};

/// Errors specific to P4K mounting
#[derive(Error, Debug)]
pub enum P4kMountError {
    #[error("Failed to open archive: {0}")]
    OpenFailed(#[from] std::io::Error),

    #[error("Failed to parse archive: {0}")]
    ParseFailed(String),

    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("Failed to extract entry: {0}")]
    ExtractionFailed(String),
}

impl From<P4kMountError> for VfsError {
    fn from(err: P4kMountError) -> Self {
        VfsError::MountError(err.to_string())
    }
}

/// P4K archive mount point
/// 
/// Mounts a P4K archive at a virtual path, providing read-only access
/// to the archive contents through the VFS interface.
pub struct P4kMountPoint {
    /// Path to the P4K archive file
    archive_path: PathBuf,
    /// Virtual mount path
    mount_path: PathBuf,
    /// Parsed archive metadata
    archive: Arc<P4kArchive>,
    /// Parser instance for extraction
    parser: P4kParser,
    /// Cache for recently extracted files
    cache: RwLock<LruCache>,
    /// Pre-built directory tree for fast navigation
    tree: DirectoryNode,
}

/// Simple LRU cache for extracted file data
struct LruCache {
    entries: HashMap<String, CacheEntry>,
    order: Vec<String>,
    max_size_bytes: usize,
    current_size: usize,
}

struct CacheEntry {
    data: Arc<Vec<u8>>,
    size: usize,
}

impl LruCache {
    fn new(max_size_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: Vec::new(),
            max_size_bytes,
            current_size: 0,
        }
    }

    fn get(&mut self, key: &str) -> Option<Arc<Vec<u8>>> {
        if let Some(entry) = self.entries.get(key) {
            // Move to end of order (most recently used)
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
                self.order.push(key.to_string());
            }
            Some(Arc::clone(&entry.data))
        } else {
            None
        }
    }

    fn insert(&mut self, key: String, data: Vec<u8>) {
        let size = data.len();
        
        // Evict old entries if necessary
        while self.current_size + size > self.max_size_bytes && !self.order.is_empty() {
            let oldest = self.order.remove(0);
            if let Some(entry) = self.entries.remove(&oldest) {
                self.current_size -= entry.size;
            }
        }

        // Only insert if it fits
        if size <= self.max_size_bytes {
            let entry = CacheEntry {
                data: Arc::new(data),
                size,
            };
            self.entries.insert(key.clone(), entry);
            self.order.push(key);
            self.current_size += size;
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.order.clear();
        self.current_size = 0;
    }

    fn size(&self) -> usize {
        self.current_size
    }
}

impl P4kMountPoint {
    /// Create a new P4K mount point
    ///
    /// # Arguments
    /// * `archive_path` - Path to the P4K archive file
    /// * `mount_path` - Virtual path where the archive will be mounted
    /// * `cache_size_mb` - Maximum cache size in megabytes (default: 256MB)
    ///
    /// # Returns
    /// A new P4kMountPoint or an error if the archive couldn't be opened/parsed
    pub fn new(
        archive_path: impl AsRef<Path>,
        mount_path: impl AsRef<Path>,
        cache_size_mb: Option<usize>,
    ) -> Result<Self, P4kMountError> {
        let archive_path = archive_path.as_ref().to_path_buf();
        let mount_path = mount_path.as_ref().to_path_buf();

        let parser = P4kParser::new();
        let archive = parser.parse_file(&archive_path)
            .map_err(|e| P4kMountError::ParseFailed(e.to_string()))?;

        let tree = archive.build_tree();
        let cache_size = cache_size_mb.unwrap_or(256) * 1024 * 1024;

        Ok(Self {
            archive_path,
            mount_path,
            archive: Arc::new(archive),
            parser,
            cache: RwLock::new(LruCache::new(cache_size)),
            tree,
        })
    }

    /// Get the archive metadata
    pub fn archive(&self) -> &P4kArchive {
        &self.archive
    }

    /// Get archive statistics
    pub fn statistics(&self) -> ArchiveStatistics {
        let stats = self.archive.statistics();
        let cache = self.cache.read();
        
        ArchiveStatistics {
            total_entries: stats.total_entries,
            file_count: stats.file_count,
            directory_count: stats.directory_count,
            total_size: stats.total_uncompressed,
            compressed_size: stats.total_compressed,
            compression_ratio: stats.compression_ratio,
            cache_size: cache.size(),
            cache_entries: cache.entries.len(),
        }
    }

    /// Clear the extraction cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Resolve a virtual path to an archive path
    fn resolve_path(&self, vfs_path: &Path) -> Option<String> {
        let relative = vfs_path.strip_prefix(&self.mount_path).ok()?;
        let archive_path = relative.to_string_lossy().replace('\\', "/");
        Some(archive_path)
    }

    /// Extract file data, using cache if available
    fn extract_cached(&self, path: &str) -> VfsResult<Arc<Vec<u8>>> {
        // Check cache first
        if let Some(data) = self.cache.write().get(path) {
            return Ok(data);
        }

        // Extract from archive
        let file = File::open(&self.archive_path)?;
        let mut reader = BufReader::new(file);

        let data = self.parser.extract_entry(&mut reader, &path.to_string())
            .map_err(|e| VfsError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string()
            )))?;

        // Cache the result
        let data_arc = {
            let mut cache = self.cache.write();
            cache.insert(path.to_string(), data.clone());
            Arc::new(data)
        };

        Ok(data_arc)
    }

    /// Find directory node for a path
    fn find_node(&self, path: &str) -> Option<&DirectoryNode> {
        if path.is_empty() || path == "/" {
            return Some(&self.tree);
        }

        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current = &self.tree;

        for part in parts {
            current = current.children.get(part)?;
        }

        Some(current)
    }
}

impl MountPoint for P4kMountPoint {
    fn mount_path(&self) -> &Path {
        &self.mount_path
    }

    fn is_read_only(&self) -> bool {
        true // P4K archives are read-only
    }

    fn exists(&self, path: &Path) -> bool {
        if let Some(archive_path) = self.resolve_path(path) {
            self.archive.contains(&archive_path) || self.find_node(&archive_path).is_some()
        } else {
            false
        }
    }

    fn is_file(&self, path: &Path) -> bool {
        if let Some(archive_path) = self.resolve_path(path) {
            self.archive.get(&archive_path)
                .map(|e| !e.is_directory)
                .unwrap_or(false)
        } else {
            false
        }
    }

    fn is_directory(&self, path: &Path) -> bool {
        if let Some(archive_path) = self.resolve_path(path) {
            if archive_path.is_empty() {
                return true; // Root is always a directory
            }
            self.archive.get(&format!("{}/", archive_path))
                .map(|e| e.is_directory)
                .unwrap_or_else(|| self.find_node(&archive_path).is_some())
        } else {
            false
        }
    }

    fn read(&self, path: &Path) -> VfsResult<Vec<u8>> {
        let archive_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;

        let data = self.extract_cached(&archive_path)?;
        Ok((*data).clone())
    }

    fn read_to_string(&self, path: &Path) -> VfsResult<String> {
        let data = self.read(path)?;
        String::from_utf8(data)
            .map_err(|e| VfsError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string()
            )))
    }

    fn list(&self, path: &Path) -> VfsResult<Vec<VfsEntry>> {
        let archive_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;

        let entries = self.archive.list_directory(&archive_path);
        
        Ok(entries.into_iter().map(|e| {
            VfsEntry {
                name: e.filename().to_string(),
                path: self.mount_path.join(&e.path),
                is_directory: e.is_directory,
                size: Some(e.uncompressed_size),
                compressed_size: Some(e.compressed_size),
            }
        }).collect())
    }

    fn metadata(&self, path: &Path) -> VfsResult<VfsNode> {
        let archive_path = self.resolve_path(path)
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;

        let entry = self.archive.get(&archive_path)
            .or_else(|| self.archive.get(&format!("{}/", archive_path)))
            .ok_or_else(|| VfsError::NotFound(path.to_path_buf()))?;

        Ok(VfsNode {
            path: path.to_path_buf(),
            name: entry.filename().to_string(),
            is_directory: entry.is_directory,
            size: entry.uncompressed_size,
            compressed_size: Some(entry.compressed_size),
            modified: None, // Could parse DOS datetime if needed
        })
    }

    fn find(&self, pattern: &str) -> VfsResult<Vec<PathBuf>> {
        let results = self.archive.find(pattern);
        Ok(results.into_iter()
            .map(|e| self.mount_path.join(&e.path))
            .collect())
    }
}

/// Archive statistics including cache info
#[derive(Debug, Clone)]
pub struct ArchiveStatistics {
    pub total_entries: usize,
    pub file_count: usize,
    pub directory_count: usize,
    pub total_size: u64,
    pub compressed_size: u64,
    pub compression_ratio: f64,
    pub cache_size: usize,
    pub cache_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_cache_basic() {
        let mut cache = LruCache::new(1000);
        
        cache.insert("key1".to_string(), vec![1, 2, 3]);
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_none());
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(10);
        
        cache.insert("key1".to_string(), vec![1, 2, 3, 4, 5]); // 5 bytes
        cache.insert("key2".to_string(), vec![1, 2, 3, 4, 5]); // 5 bytes
        
        // This should evict key1
        cache.insert("key3".to_string(), vec![1, 2, 3, 4, 5]); // 5 bytes
        
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_lru_cache_clear() {
        let mut cache = LruCache::new(1000);
        
        cache.insert("key1".to_string(), vec![1, 2, 3]);
        cache.insert("key2".to_string(), vec![4, 5, 6]);
        
        cache.clear();
        
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_none());
        assert_eq!(cache.size(), 0);
    }
}