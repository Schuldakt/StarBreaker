//! VFS search functionality

use crate::node::VfsNode;

/// Search query builder
pub struct SearchQuery {
    /// File name pattern (glob)
    pub pattern: Option<String>,
    /// File extension filter
    pub extension: Option<String>,
    /// Minimum file size
    pub min_size: Option<u64>,
    /// Maximum file size
    pub max_size: Option<u64>,
    /// Tags to match
    pub tags: Vec<String>,
}

impl SearchQuery {
    /// Create a new search query
    pub fn new() -> Self {
        Self {
            pattern: None,
            extension: None,
            min_size: None,
            max_size: None,
            tags: Vec::new(),
        }
    }

    /// Set file name pattern
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set file extension filter
    pub fn with_extension(mut self, ext: impl Into<String>) -> Self {
        self.extension = Some(ext.into());
        self
    }

    /// Set size range
    pub fn with_size_range(mut self, min: u64, max: u64) -> Self {
        self.min_size = Some(min);
        self.max_size = Some(max);
        self
    }

    /// Add tag filter
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Check if a node matches this query
    pub fn matches(&self, node: &VfsNode) -> bool {
        // Extension filter
        if let Some(ref ext) = self.extension {
            if !node.has_extension(ext) {
                return false;
            }
        }

        // Size filters
        if let Some(min) = self.min_size {
            if node.size < min {
                return false;
            }
        }

        if let Some(max) = self.max_size {
            if node.size > max {
                return false;
            }
        }

        // Tag filters
        if !self.tags.is_empty() {
            if !self.tags.iter().all(|tag| node.metadata.tags.contains(tag)) {
                return false;
            }
        }

        true
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self::new()
    }
}
