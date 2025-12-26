//! StarBreaker Core Library
//!
//! This crate provides common types, utilities, and error handling
//! shared across all StarBreaker components.

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::*;

/// Re-export commonly used items
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::types::*;
}