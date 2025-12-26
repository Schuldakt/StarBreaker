//! Unified error handling for StarBreaker
//!
//! This module provides a comprehensive error type that encompasses
//! all possible errors across the StarBreaker crates.

use std::path::PathBuf;
use thiserror::Error;

/// Unified error type for all StarBreaker operations
#[derive(Error, Debug)]
pub enum Error {
    // ==================== I/O Errors ====================
    
    /// Standard I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// File not found
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    /// Permission denied
    #[error("Permission denied: {0}")]
    PermissionDenied(PathBuf),

    // ==================== Parse Errors ====================
    
    /// Invalid magic bytes at file start
    #[error("Invalid magic bytes: expected {expected:?}, found {found:?}")]
    InvalidMagic {
        expected: Vec<u8>,
        found: Vec<u8>,
    },

    /// Unsupported format version
    #[error("Unsupported version: {version} (supported: {supported})")]
    UnsupportedVersion {
        version: String,
        supported: String,
    },

    /// Unexpected end of file
    #[error("Unexpected end of file at offset {offset}")]
    UnexpectedEof {
        offset: u64,
    },

    /// Invalid data structure
    #[error("Invalid data: {message}")]
    InvalidData {
        message: String,
    },

    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingField {
        field: String,
    },

    /// Checksum mismatch
    #[error("Checksum mismatch: expected {expected:08X}, got {actual:08X}")]
    ChecksumMismatch {
        expected: u32,
        actual: u32,
    },

    // ==================== Compression Errors ====================
    
    /// Unsupported compression method
    #[error("Unsupported compression method: {method}")]
    UnsupportedCompression {
        method: String,
    },

    /// Decompression failed
    #[error("Decompression failed: {message}")]
    DecompressionFailed {
        message: String,
    },

    // ==================== VFS Errors ====================
    
    /// Path not found in VFS
    #[error("VFS path not found: {0}")]
    VfsNotFound(PathBuf),

    /// No mount point for path
    #[error("No mount point for path: {0}")]
    VfsNoMount(PathBuf),

    /// VFS is read-only
    #[error("VFS is read-only")]
    VfsReadOnly,

    /// Mount point conflict
    #[error("Mount point conflict: {0}")]
    MountConflict(String),

    // ==================== Export Errors ====================
    
    /// Unsupported export format
    #[error("Unsupported export format: {format}")]
    UnsupportedFormat {
        format: String,
    },

    /// Export failed
    #[error("Export failed: {message}")]
    ExportFailed {
        message: String,
    },

    // ==================== Archive Errors ====================
    
    /// Entry not found in archive
    #[error("Archive entry not found: {path}")]
    EntryNotFound {
        path: String,
    },

    /// Archive is corrupted
    #[error("Archive corrupted: {message}")]
    ArchiveCorrupted {
        message: String,
    },

    // ==================== Database Errors ====================
    
    /// Record not found in database
    #[error("Record not found: {id}")]
    RecordNotFound {
        id: String,
    },

    /// Struct type not found
    #[error("Struct type not found: {name}")]
    StructNotFound {
        name: String,
    },

    /// Invalid reference
    #[error("Invalid reference: {reference}")]
    InvalidReference {
        reference: String,
    },

    // ==================== Configuration Errors ====================
    
    /// Invalid configuration
    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        message: String,
    },

    /// Missing configuration
    #[error("Missing configuration: {key}")]
    MissingConfig {
        key: String,
    },

    // ==================== General Errors ====================
    
    /// Operation cancelled
    #[error("Operation cancelled")]
    Cancelled,

    /// Operation timed out
    #[error("Operation timed out after {seconds} seconds")]
    Timeout {
        seconds: u64,
    },

    /// Internal error (should not happen)
    #[error("Internal error: {message}")]
    Internal {
        message: String,
    },

    /// Custom error with context
    #[error("{context}: {source}")]
    WithContext {
        context: String,
        #[source]
        source: Box<Error>,
    },

    /// Multiple errors occurred
    #[error("Multiple errors occurred: {0:?}")]
    Multiple(Vec<Error>),

    /// External error (from other crates)
    #[error("{0}")]
    External(String),
}

/// Result type using the unified Error
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create an error with additional context
    pub fn with_context(self, context: impl Into<String>) -> Self {
        Error::WithContext {
            context: context.into(),
            source: Box::new(self),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Error::Internal {
            message: message.into(),
        }
    }

    /// Create an invalid data error
    pub fn invalid_data(message: impl Into<String>) -> Self {
        Error::InvalidData {
            message: message.into(),
        }
    }

    /// Create a missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        Error::MissingField {
            field: field.into(),
        }
    }

    /// Check if this is a "not found" type error
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Error::FileNotFound(_)
                | Error::VfsNotFound(_)
                | Error::EntryNotFound { .. }
                | Error::RecordNotFound { .. }
                | Error::StructNotFound { .. }
        )
    }

    /// Check if this is a permission/access error
    pub fn is_permission_error(&self) -> bool {
        matches!(self, Error::PermissionDenied(_) | Error::VfsReadOnly)
    }

    /// Check if this is a parse/format error
    pub fn is_parse_error(&self) -> bool {
        matches!(
            self,
            Error::InvalidMagic { .. }
                | Error::UnsupportedVersion { .. }
                | Error::InvalidData { .. }
                | Error::MissingField { .. }
                | Error::ChecksumMismatch { .. }
                | Error::ArchiveCorrupted { .. }
        )
    }
}

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, context: impl Into<String>) -> Result<T>;
    
    /// Add context with a closure (lazy evaluation)
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T> ResultExt<T> for Result<T> {
    fn context(self, context: impl Into<String>) -> Result<T> {
        self.map_err(|e| e.with_context(context))
    }

    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| e.with_context(f()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_with_context() {
        let err = Error::FileNotFound(PathBuf::from("/test"));
        let contextualized = err.with_context("while loading config");
        
        assert!(contextualized.to_string().contains("while loading config"));
    }

    #[test]
    fn test_is_not_found() {
        assert!(Error::FileNotFound(PathBuf::from("/test")).is_not_found());
        assert!(Error::EntryNotFound { path: "test".into() }.is_not_found());
        assert!(!Error::VfsReadOnly.is_not_found());
    }

    #[test]
    fn test_is_parse_error() {
        assert!(Error::InvalidMagic {
            expected: vec![],
            found: vec![],
        }.is_parse_error());
        
        assert!(!Error::FileNotFound(PathBuf::from("/test")).is_parse_error());
    }

    #[test]
    fn test_result_context() {
        let result: Result<()> = Err(Error::FileNotFound(PathBuf::from("/test")));
        let with_context = result.context("loading data");
        
        assert!(with_context.is_err());
        assert!(with_context.unwrap_err().to_string().contains("loading data"));
    }
}