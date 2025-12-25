//! starbreaker-parsers
//! 
//! A comprehensive library for parsing Star Citizen and CryEngine file formats.
//! 
//! # Supported Formats
//! 
//! | Format | Extension | Description |
//! |--------|-----------|-------------|
//! | P4K    | `.p4k`    | Main game archive (ZIP-based) |
//! | DCB    | `.dcb`    | DataCore Binary (game database) |
//! | SOC    | `.soc`    | Scene Object Container |
//! | SOCPAK | `.socpak` | Packaged Scene Objects |
//! | CGF    | `.cgf`    | CryEngine Geometry |
//! | CGA    | `.cga`    | CryEngine Animation |
//! | CHR    | `.chr`    | Character Model |
//! | SKIN   | `.skin`   | Skinned Mesh |
//! | MTL    | `.mtl`    | Material Definition |
//! | DDS    | `.dds`    | DirectDraw Surface Texture |
//! 
//! # Example
//! 
//! ```rust,ignore
//! use starbreaker_parsers::{p4kParser, Parser};
//! 
//! let parser = P4kParser::new();
//! let archive = parser.parse_file("Data.p4k")?;
//! 
//! println!("Found {} entires", archive.entry_county())
//! ```

pub mod traits;
pub mod registry;
pub mod p4k;
pub mod dcb;
pub mod cgf;
pub mod dds;

// Re-export main types
pub use traits::{
    Parser, StreamingParser, RandomAccessParser, HierarchicalParser,
    ParseError, ParseResult, ParseOptions, ParseProgress, ParsePhase,
    ProgressCallback
};

pub use registry::{
    ParserRegistry, ParserRegistration, ParserRegistrationBuilder,
    ParserInfo, RegistryError, AnyParser, GLOBAL_REGISTRY,
};

pub use p4k::{P4kParser, P4kArchive, P4kEntry, P4kEntryInfo, P4kCompression, CompressionMethod};
pub use dcb::{DcbParser, DataCore, DataCoreHeader, Record, RecordValue, RecordRef, StructDef, PropertyDef, DataType};
pub use cgf::{CgfParser, CgfModel, Mesh, Vertex, Face, Skeleton, Bone};
pub use dds::{DdsParser, DdsTexture, DdsCombiner, DdsHeader, TextureFormat};

/// Initialize the global parser registry with all built-in parsers
pub fn init_registry() {

    // Register P4K parser
    let _ = GLOBAL_REGISTRY.register(
        ParserRegistrationBuilder::new()
            .id("p4k")
            .name("P4K Archive Parser")
            .description("Parses Star Citizen .p4k archive files")
            .extensions(&["p4k"])
            .priority(100)
            .factory(|| p4k::P4kParser::new())
            .build()
            .unwrap()
    );

    // Register DCB Parser
    let _ = GLOBAL_REGISTRY.register(
        ParserRegistrationBuilder::new()
            .id("dcb")
            .name("DataCore Binary Parser")
            .description("Parses Star Citizen Game2.dcb database files")
            .extensions(&["dcb"])
            .priority(100)
            .factory(|| dcb::DcbParser::new())
            .build()
            .unwrap()
    );
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");