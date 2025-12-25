use lasso::{Rodeo, Spur};
use std::sync::Arc;
use parking_lot::RwLock;

/// Thread-safe string interner for DCB parsing
pub struct StringInterner {
    rodeo: RwLock<Rodeo>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            rodeo: RwLock::new(Rodeo::default()),
        }
    }
    
    /// Intern a string, returning a cheap-to-copy key
    pub fn intern(&self, s: &str) -> Spur {
        self.rodeo.write().get_or_intern(s)
    }
    
    /// Resolve a key back to a string
    pub fn resolve(&self, key: Spur) -> Option<String> {
        self.rodeo.read().try_resolve(&key).map(|s| s.to_string())
    }
}

/// Interned string reference (8 bytes instead of 24 for String)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternedStr(Spur);