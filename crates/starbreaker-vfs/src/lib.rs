//! StarBreaker Virtual File System
//!
//! Provides a unified view over multiple file sources including:
//! - P4K archives
//! - Local filesystem directories
//! - DCB virtual folders
//! - Overlay/mod systems
//!
//! # Example
//! ```no_run
//! use starbreaker_vfs::{VfsTree, mount::FilesystemMount};
//! use std::sync::Arc;
//!
//! let vfs = VfsTree::new();
//!
//! // Mount a local directory
//! let mount = FilesystemMount::new(0, "game_data", "/path/to/game").unwrap();
//! vfs.add_mount(Arc::new(mount));
//!
//! // Check if file exists
//! if vfs.exists("/Data/Scripts/test.lua") {
//!     // Open and read file
//!     let mut file = vfs.open_file("/Data/Scripts/test.lua").unwrap();
//!     // ... read from file
//! }
//! ```

pub mod node;
pub mod mount;
pub mod path;
pub mod tree;
pub mod search;
pub mod stream;

pub use node::{VfsNode, NodeType, NodeMetadata};
pub use mount::{MountPoint, MountError, MountResult, P4kMount, FilesystemMount};
pub use tree::VfsTree;
pub use stream::{VfsStreamReader, ChunkedReader};
