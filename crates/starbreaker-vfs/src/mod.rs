//! Mount point implementations
//!
//! This module contains various mount point implementations for different
//! storage backends.

pub mod p4k;

pub use p4k::P4kMountPoint;