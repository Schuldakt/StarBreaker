// starbreaker-parsers/src/p4k/archive.rs
//! P4K Archive container structure

use std::collections::HashMap;
use super::entry::P4kEntry;

/// Parsed P4K archive structure
#[derive(Debug)]
pub struct P4kArchive {
    /// All entries in the archive
    pub entries: Vec<P4kEntry>,
    /// Path to entry index mapping for fast lookup
    pub path_index: HashMap<String, usize>,
}

impl P4kArchive {
    /// Create a new empty archive
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            path_index: HashMap::new(),
        }
    }

    /// Get total number of entries
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get number of files (non-directories)
    pub fn file_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.is_directory).count()
    }

    /// Get number of directories
    pub fn directory_count(&self) -> usize {
        self.entries.iter().filter(|e| e.is_directory).count()
    }

    /// Get total uncompressed size
    pub fn total_uncompressed_size(&self) -> u64 {
        self.entries.iter().map(|e| e.uncompressed_size).sum()
    }

    /// Get total compressed size
    pub fn total_compressed_size(&self) -> u64 {
        self.entries.iter().map(|e| e.compressed_size).sum()
    }

    /// Get an entry by path
    pub fn get (&self, path: &str) -> Option<&P4kEntry> {
        self.path_index.get(path).map(|idx| &self.entries[*idx])
    }

    /// Check if path exists in archive
    pub fn contains(&self, path: &str) -> bool {
        self.path_index.contains_key(path)
    }

    /// Find entries matching a pattern (glob-like)
    pub fn find(&self, pattern: &str) -> Vec<&P4kEntry> {
        let pattern = pattern.to_lowercase();
        let parts: Vec<&str> = pattern.split('*').collect();

        self.entries.iter().filter(|entry| {
            let path = entry.path.to_lowercase();

            if parts.len() == 1 {
                // No wildcards
                path.contains(&pattern)
            } else {
                // Handle wildcards
                let mut pos = 0;
                for (i, part) in parts.iter().enumerate() {
                    if part.is_empty() {
                        continue;
                    }

                    if i == 0 {
                        // Must start with first part
                        if !path.starts_with(*part) {
                            return false;
                        }
                        pos = part.len();
                    } else if i == parts.len() - 1 {
                        // Must end with last part
                        if !path.ends_with(*part) {
                            return false;
                        }
                    } else {
                        // Must contain middle part
                        if let Some(idx) = path[pos..].find(*part) {
                            pos += idx + part.len();
                        } else {
                            return false;
                        }
                    }
                }
                true
            }
        }).collect()
    }

    /// Find entries by extension
    pub fn find_by_extension(&self, ext: &str) -> Vec<&P4kEntry> {
        let ext = ext.trim_start_matches('.').to_lowercase();
        self.entries.iter()
            .filter(|e| {
                e.extension()
                    .map(|e| e.to_lowercase() == ext)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// List entries in a directory
    pub fn list_directory(&self, path: &str) -> Vec<&P4kEntry> {
        let path = path.trim_end_matches('/');
        let prefix = if path.is_empty() { String::new() } else { format!("{}/", path) };

        self.entries.iter()
            .filter(|entry| {
                if entry.path.starts_with(&prefix) {
                    let remainder = &entry.path[prefix.len()..];
                    // Only direct children (no additional slashes, or just trailing slash)
                    !remainder.trim_end_matches('/').contains('/')
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get all top-level directories
    pub fn root_directories(&self) -> Vec<String> {
        let mut roots: std::collections::HashSet<String> = std::collections::HashSet::new();

        for entry in &self.entries {
            if let Some(idx) = entry.path.find('/') {
                roots.insert(entry.path[..idx].to_string());
            }
        }

        let mut result: Vec<String> = roots.into_iter().collect();
        result.sort();
        result
    }

    /// Build a tree structure for navigation
    pub fn build_tree(&self) -> DirectoryNode {
        let mut root = DirectoryNode::new("".to_string());

        for entry in &self.entries {
            root.insert(&entry.path, entry.is_directory);
        }

        root
    }

    /// Get archive statistics
    pub fn statistics(&self) -> ArchiveStatistics {
        let mut stats = ArchiveStatistics::default();

        stats.total_entries = self.entries.len();

        for entry in &self.entries {
            if entry.is_directory {
                stats.directory_count += 1;
            } else {
                stats.file_count += 1;
                stats.total_uncompressed += entry.uncompressed_size;
                stats.total_compressed += entry.compressed_size;

                if let Some(ext) = entry.extension() {
                    *stats.extensions.entry(ext.to_lowercase()).or_insert(0) += 1;
                }
            }
        }

        if stats.total_uncompressed > 0 {
            stats.compression_ratio =
                stats.total_compressed as f64 / stats.total_uncompressed as f64;
        }

        stats
    }
}

impl Default for P4kArchive {
    fn default() -> Self {
        Self::new()
    }
}

/// Directory tree node for navigation
#[derive(Debug, Clone)]
pub struct DirectoryNode {
    /// Node name (directory or file name)
    pub name: String,
    /// Whether this is a file (leaf node)
    pub is_file: bool,
    /// Child nodes
    pub children: HashMap<String, DirectoryNode>,
}

impl DirectoryNode {
    /// Create a new directory node
    pub fn new(name: String) -> Self {
        Self {
            name,
            is_file: false,
            children: HashMap::new(),
        }
    }

    /// Insert a path into the tree
    pub fn insert(&mut self, path: &str, is_directory: bool) {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        self.insert_parts(&parts, !is_directory);
    }

    fn insert_parts(&mut self, parts: &[&str], is_file: bool) {
        if parts.is_empty() {
            return;
        }

        let name = parts[0].to_string();
        let is_leaf = parts.len() == 1;

        let child = self.children.entry(name.clone()).or_insert_with(|| {
            DirectoryNode::new(name)
        });

        if is_leaf {
            child.is_file = is_file;
        } else {
            child.insert_parts(&parts[1..], is_file);
        }
    }

    /// Get sorted child names
    pub fn sorted_children(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.children.keys().map(|s| s.as_str()).collect();
        // Sort directories first, then alphabetically
        names.sort_by(|a, b| {
            let a_is_dir = !self.children.get(*a).map(|n| n.is_file).unwrap_or(false);
            let b_is_dir = !self.children.get(*b).map(|n| n.is_file).unwrap_or(false);

            match (a_is_dir, b_is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.to_lowercase().cmp(&b.to_lowercase()),
            }
        });
        names
    }
}

/// Archive statistics
#[derive(Debug, Default)]
pub struct ArchiveStatistics {
    /// Total number of entries
    pub total_entries: usize,
    /// Number of files
    pub file_count: usize,
    /// Number of directories
    pub directory_count: usize,
    /// Total uncompressed size in bytes
    pub total_uncompressed: u64,
    /// Total compressed size in bytes
    pub total_compressed: u64,
    /// Overall compression ratio
    pub compression_ratio: f64,
    /// File count by extension
    pub extensions: HashMap<String, usize>,
}

impl ArchiveStatistics {
    /// Get top N extensions by file count
    pub fn top_extensions(&self, n: usize) -> Vec<(&str, usize)> {
        let mut exts: Vec<_> = self.extensions.iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect();
        exts.sort_by(|a, b| b.1.cmp(&a.1));
        exts.truncate(n);
        exts
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p4k::CompressionMethod;

    fn make_test_archive() -> P4kArchive {
        let entries = vec![
            ("Data/", true),
            ("Data/Libs/", true),
            ("Data/Libs/Config/", true),
            ("Data/Libs/Config/defaultprofile.xml", false),
            ("Data/Libx/Config/profiles.xml", false),
            ("Data/Textures/", true),
            ("Data/Textures/ship.dds", false),
            ("Data/Objects/", true),
            ("Data/Objects/ship.cgf", false),
        ];

        let entries: Vec<_> = entries.iter().map(|(path, is_dir)| {
            crate::p4k::P4kEntry {
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
                is_directory: *is_dir,
            }
        }).collect();

        let mut path_index = HashMap::new();
        for (idx, entry) in entries.iter().enumerate() {
            path_index.insert(entry.path.clone(), idx);
        }

        P4kArchive { entries, path_index }
    }

    #[test]
    fn test_find_by_extension() {
        let archive = make_test_archive();

        let dds_files = archive.find_by_extension("dds");
        assert_eq!(dds_files.len(), 1);
        assert_eq!(dds_files[0].path, "Data/Textures/ship.dds");

        let xml_files = archive.find_by_extension(".xml");
        assert_eq!(xml_files.len(), 2);
    }

    #[test]
    fn test_list_directory() {
        let archive = make_test_archive();

        let libs = archive.list_directory("Data/Libs");
        assert_eq!(libs.len(), 1);
        assert_eq!(libs[0].path, "Data/Libs/Config/");

        let config = archive.list_directory("Data/Libs/Config");
        assert_eq!(config.len(), 2);
    }

    #[test]
    fn test_find_pattern() {
        let archive = make_test_archive();
        
        let results = archive.find("*.xml");
        assert_eq!(results.len(), 2);

        let results = archive.find("Data/Textures/*");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_build_tree() {
        let archive = make_test_archive();
        let tree = archive.build_tree();

        assert!(tree.children.contains_key("Data"));
        let data = &tree.children["Data"];
        assert!(data.children.contains_key("Libs"));
        assert!(data.children.contains_key("Textures"));
    }
}