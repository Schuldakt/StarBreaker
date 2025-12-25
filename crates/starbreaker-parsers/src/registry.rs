//! Parser registry for dynamic parser discovery and management.
//!
//! The registry provides a centralized way to register, discovr, and
//! instantiate parsers for various file formats. This enables the
//! plugin-style architecture where new parsers can be added without
//! modifying existing code.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::traits::Parser;

/// Type-erased parser wrapper for storage in the registry
pub trait AnyParser: Send + Sync {
    /// Get the parser name
    fn name(&self) -> &str;

    /// Get supported file extensions
    fn extensions(&self) -> &[&str];

    /// Get magic bytes if applicable
    fn magic_bytes(&self) -> Option<&[u8]>;

    /// Check if this parser can handle the given path
    fn can_parse(&self, path: &Path) -> bool;

    /// Get a type identifier for downcasting
    fn type_id(&self) -> std::any::TypeId;
}

impl<T: Parser + 'static> AnyParser for T {
    fn name(&self) -> &str {
        Parser::name(self)
    }

    fn extensions (&self) -> &[&str] {
        Parser::extensions(self)
    }

    fn magic_bytes(&self) -> Option<&[u8]> {
        Parser::magic_bytes(self)
    }

    fn can_parse(&self, path: &Path) -> bool {
        Parser::can_parse(self, path)
    }

    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<T>()
    }
}

/// Fctory function typoe for creating parser instances
pub type ParserFactory = Box<dyn Fn() -> Arc<dyn AnyParser> + Send + Sync>;

/// Registration entry for a parser
pub struct ParserRegistration {
    /// Unique identifier for this parser
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this parser handles
    pub description: String,
    /// File extensions handled (lowercase)
    pub extensions: Vec<String>,
    /// Priority for extension conflics (higher = preferred)
    pub priority: i32,
    /// Factory function to create parser instance
    pub factory: ParserFactory,
}

/// Global parser registry
pub struct ParserRegistry {
    /// Map of parser ID to registration
    parsers: RwLock<HashMap<String, ParserRegistration>>,
    /// Map of extensions to parser IDs (sorted by priority)
    extension_map: RwLock<HashMap<String, Vec<String>>>,
    /// Cached parser instances
    instances: RwLock<HashMap<String, Arc<dyn AnyParser>>>,
}

impl ParserRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            parsers: RwLock::new(HashMap::new()),
            extension_map: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new parser
    pub fn register(&self, registration: ParserRegistration) -> Result<(), RegistryError> {
        let id = registration.id.clone();

        // Check for duplicate ID
        {
            let parsers = self.parsers.read().map_err(|_| RegistryError::LockPoisoned)?;
            if parsers.contains_key(&id) {
                return Err(RegistryError::DuplicateId(id));
            }
        }

        // Register parser
        {
            let mut parsers = self.parsers.write().map_err(|_| RegistryError::LockPoisoned)?;
            let mut ext_map = self.extension_map.write().map_err(|_| RegistryError::LockPoisoned)?;

            // Update extension map
            for ext in &registration.extensions {
                let ext_lower = ext.to_lowercase();
                let ids = ext_map.entry(ext_lower).or_insert_with(Vec::new);
                ids.push(id.clone());

                // Sort by priority (descending)
                ids.sort_by(|a, b| {
                    let pa = parsers.get(a).map(|p| p.priority).unwrap_or(0);
                    let pb = parsers.get(b).map(|p| p.priority).unwrap_or(0);
                    pb.cmp(&pa)
                });
            }

            parsers.insert(id, registration);
        }

        Ok(())
    }

    /// Unregister a parser by ID
    pub fn unregister(&self, id: &str) -> Result<(), RegistryError> {
        let mut parsers = self.parsers.write().map_err(|_| RegistryError::LockPoisoned)?;
        let mut ext_map = self.extension_map.write().map_err(|_| RegistryError::LockPoisoned)?;
        let mut instances = self.instances.write().map_err(|_| RegistryError::LockPoisoned)?;

        if let Some(registration) = parsers.remove(id) {
            // Remove from extension map
            for ext in &registration.extensions {
                if let Some(ids) = ext_map.get_mut(&ext.to_lowercase()) {
                    ids.retain(|i| i != id);
                }
            }

            // Remove cached instance
            instances.remove(id);

            Ok(())
        } else {
            Err(RegistryError::NotFound(id.to_string()))
        }
    }

    /// Get a parser instance by ID
    pub fn get(&self, id: &str) -> Result<Arc<dyn AnyParser>, RegistryError> {
        // Check cache first
        {
            let instances = self.instances.read().map_err(|_| RegistryError::LockPoisoned)?;
            if let Some(instance) = instances.get(id) {
                return Ok(Arc::clone(instance));
            }
        }

        // Create new instance
        let instance = {
            let parsers = self.parsers.read().map_err(|_| RegistryError::LockPoisoned)?;
            let registration = parsers.get(id)
                .ok_or_else(|| RegistryError::NotFound(id.to_string()))?;
            (registration.factory)()
        };

        // Cache it
        {
            let mut instances = self.instances.write().map_err(|_| RegistryError::LockPoisoned)?;
            instances.insert(id.to_string(), Arc::clone(&instance));
        }

        Ok(instance)
    }

    /// Get a parser for a file extension
    pub fn get_for_extension(&self, ext: &str) -> Result<Arc<dyn AnyParser>, RegistryError> {
        let ext_lower = ext.to_lowercase().trim_start_matches('.').to_string();

        let id = {
            let ext_map = self.extension_map.read().map_err(|_| RegistryError::LockPoisoned)?;
            ext_map.get(&ext_lower)
                .and_then(|ids| ids.first())
                .cloned()
                .ok_or_else(|| RegistryError::NoParserForExtension(ext_lower.clone()))?
        };

        self.get(&id)
    }

    /// Get a parser for a file path
    pub fn get_for_path(&self, path: &Path) -> Result<Arc<dyn AnyParser>, RegistryError> {
        // Try extension first
        if let Some(ext) = path.extension() {
            if let Ok(parser) = self.get_for_extension(&ext.to_string_lossy()) {
                return Ok(parser);
            }
        }

        // Try magic bytes detection
        let parsers = self.parsers.read().map_err(|_| RegistryError::LockPoisoned)?;
        for (id, _) in parsers.iter() {
            if let Ok(parser) = self.get(id) {
                if parser.can_parse(path) {
                    return Ok(parser);
                }
            }
        }

        Err(RegistryError::NoParserForPath(path.to_path_buf()))
    }

    /// List all registered parsers
    pub fn list(&self) -> Result<Vec<ParserInfo>, RegistryError> {
        let parsers = self.parsers.read().map_err(|_| RegistryError::LockPoisoned)?;

        Ok(parsers.values().map(|p| ParserInfo {
            id: p.id.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
            extensions: p.extensions.clone(),
            priority: p.priority,
        }).collect())
    }

    /// Get typed parser instance
    pub fn get_typed<T: Parser + 'static>(&self, id: &str) -> Result<Arc<T>, RegistryError> {
        let parser = self.get(id)?;

        // Check type matches
        if parser.type_id() != std::any::TypeId::of::<T>() {
            return Err(RegistryError::TypeMismatch {
                expected: std::any::type_name::<T>().to_string(),
                found: parser.name().to_string(),
            });
        }

        // This is safe because we verified the type
        // However, we can't actually downcast Arc<dyn AnyParser> to Arc<T>
        // So we need a different approach - store typed instances separately
        Err(RegistryError::TypeMismatch {
            expected: std::any::type_name::<T>().to_string(),
            found: "dynamic parser".to_string(),
        })
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser information for display
#[derive(Debug, Clone)]
pub struct ParserInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub extensions: Vec<String>,
    pub priority: i32,
}

/// Registry errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Parser with ID '{0}' already registered")]
    DuplicateId(String),

    #[error("Parser with ID '{0}' not found")]
    NotFound(String),

    #[error("No parser available for extensions '.{0}'")]
    NoParserForExtension(String),

    #[error("No parser available for path: {0}")]
    NoParserForPath(std::path::PathBuf),

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Registry lock poisoned")]
    LockPoisoned,
}

/// Global registry instance
pub static GLOBAL_REGISTRY: Lazy<ParserRegistry> = Lazy::new(|| {
    let registry = ParserRegistry::new();

    // Register built-in parsers
    register_builtin_parsers(&registry);

    registry
});

/// Register all built-in parsers
fn register_builtin_parsers(_registry: &ParserRegistry) {
    // These will be implemented in their respective modules
    // and registered here during initialization

    // P4k parser
    // registry.register(ParserRegistration {
    //      id: "p4l".to_string(),
    //      name: "P3K Archive Parser".to_string(),
    //      description: "Parses Star Citizen .p4k archive files".to_string(),
    //      extensions: vec!["p4k".to_string()],
    //      priority: 100,
    //      factory: Box::new(|| Arc::new(crate::p4k::P4kParser::new())),
    // }).ok();
}

/// Builder for parser registration
pub struct ParserRegistrationBuilder {
    id: Option<String>,
    name: Option<String>,
    description: String,
    extensions: Vec<String>,
    priority: i32,
    factory: Option<ParserFactory>,
}

impl ParserRegistrationBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: String::new(),
            extensions: Vec::new(),
            priority: 0,
            factory: None,
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn extensions(mut self, exts: &[&str]) -> Self {
        self.extensions = exts.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn factory<F, P>(mut self, factory: F) -> Self
    where
        F: Fn() -> P + Send + Sync + 'static,
        P: Parser + 'static,
    {
        self.factory = Some(Box::new(move || Arc::new(factory())));
        self
    }

    pub fn build(self) -> Result<ParserRegistration, &'static str> {
        let id = self.id.ok_or("ID is required")?;
        let factory = self.factory.ok_or("Factory is required")?;

        Ok(ParserRegistration {
            id: id.clone(),
            name: self.name.unwrap_or_else(|| id.clone()),
            description: self.description,
            extensions: self.extensions,
            priority: self.priority,
            factory,
        })
    }
}

impl Default for ParserRegistrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek};
    use crate::traits::{ParseOptions, ProgressCallback, ParseResult};

    // Mock parser for testing
    struct MockParser;
    
    impl Parser for MockParser {
        type Output = Vec<u8>;

        fn extensions(&self) -> &[&str] {
            &["mock", "test"]
        }

        fn name(&self) -> &str {
            "Mock Parser"
        }

        fn parse_with_options<R: Read + Seek>(
            &self,
            _reader: R,
            _options: &ParseOptions,
            _progress: Option<ProgressCallback>,
        ) -> ParseResult<Self::Output> {
            Ok(vec![1, 2, 3])
        }
    }

    #[test]
    fn test_registry_registration() {
        let registry = ParserRegistry::new();

        let registration = ParserRegistrationBuilder::new()
            .id("mock")
            .name("Mock Parser")
            .extensions(&["mock", "test"])
            .priority(10)
            .factory(|| MockParser)
            .build()
            .unwrap();
        
        registry.register(registration).unwrap();

        let parser = registry.get("mock").unwrap();
        assert_eq!(parser.name(), "Mock Parser");
    }

    #[test]
    fn test_extension_lookup() {
        let registry = ParserRegistry::new();

        let registration = ParserRegistrationBuilder::new()
            .id("mock")
            .extensions(&["mock"])
            .factory(|| MockParser)
            .build()
            .unwrap();

        registry.register(registration).unwrap();

        let parser = registry.get_for_extension("mock").unwrap();
        assert_eq!(parser.name(), "Mock Parser");

        let parser = registry.get_for_extension(".mock").unwrap();
        assert_eq!(parser.name(), "Mock Parser");
    }
}