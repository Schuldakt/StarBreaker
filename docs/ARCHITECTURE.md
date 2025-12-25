# StarBreaker Architecture

> Technical documentation for developers and contributors

This document describes the technical architecture of StarBreaker, including crate organization, data flow, design patterns, and implementation details.

---

## Table of Contents

- [Overview](#overview)
- [Crate Structure](#crate-structure)
- [Core Concepts](#core-concepts)
- [Data Flow](#data-flow)
- [File Format Details](#file-format-details)
- [Design Patterns](#design-patterns)
- [Performance Considerations](#performance-considerations)
- [Error Handling](#error-handling)
- [Testing Strategy](#testing-strategy)

---

## Overview

StarBreaker is designed as a modular Rust workspace with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        StarBreaker Application                       │
├─────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │   CLI Tool  │  │  GUI (egui) │  │   Library   │   ← Entry Points │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                  │
│         │                │                │                          │
│  ┌──────┴────────────────┴────────────────┴──────┐                  │
│  │              starbreaker-gui                   │   ← Presentation │
│  │         (widgets, panels, state)              │                  │
│  └──────────────────────┬────────────────────────┘                  │
│                         │                                            │
│  ┌──────────────────────┴────────────────────────┐                  │
│  │            starbreaker-render                  │   ← 3D Preview   │
│  │         (wgpu, camera, shaders)               │                  │
│  └──────────────────────┬────────────────────────┘                  │
│                         │                                            │
│  ┌──────────────────────┴────────────────────────┐                  │
│  │             starbreaker-export                 │   ← Export       │
│  │          (FBX, glTF, textures)                │                  │
│  └──────────────────────┬────────────────────────┘                  │
│                         │                                            │
│  ┌──────────────────────┴────────────────────────┐                  │
│  │            starbreaker-datacore               │   ← Game Data    │
│  │       (ships, weapons, items, stats)          │                  │
│  └──────────────────────┬────────────────────────┘                  │
│                         │                                            │
│  ┌──────────────────────┴────────────────────────┐                  │
│  │               starbreaker-vfs                  │   ← Virtual FS   │
│  │         (mount points, search, tree)          │                  │
│  └──────────────────────┬────────────────────────┘                  │
│                         │                                            │
│  ┌──────────────────────┴────────────────────────┐                  │
│  │            starbreaker-parsers                 │   ← Parsing      │
│  │     (P4K, DCB, CGF, DDS, MTL, SOC, etc.)     │                  │
│  └──────────────────────┬────────────────────────┘                  │
│                         │                                            │
│  ┌──────────────────────┴────────────────────────┐                  │
│  │              starbreaker-core                  │   ← Foundation   │
│  │       (types, compression, utilities)         │                  │
│  └───────────────────────────────────────────────┘                  │
└─────────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Modularity** - Each crate has a single responsibility
2. **Zero-Copy** - Parse data in place where possible
3. **Lazy Loading** - Only parse what's needed
4. **Parallel by Default** - Use Rayon for CPU-bound work
5. **Memory Efficiency** - Stream large files, cache intelligently
6. **Error Transparency** - Rich error types with context

---

## Crate Structure

### `starbreaker-core`

**Purpose:** Foundational types and utilities shared across all crates.

```
starbreaker-core/
├── src/
│   ├── lib.rs
│   ├── error.rs           # Base error types
│   ├── types/
│   │   ├── mod.rs
│   │   ├── vector.rs      # Vec2, Vec3, Vec4
│   │   ├── quaternion.rs  # Quat for rotations
│   │   ├── matrix.rs      # Mat3x3, Mat4x4
│   │   └── bounds.rs      # AABB, OBB
│   ├── compression/
│   │   ├── mod.rs
│   │   ├── zlib.rs        # Deflate
│   │   ├── lz4.rs         # LZ4 (SC custom)
│   │   └── zstd.rs        # Zstandard
│   └── utils/
│       ├── mod.rs
│       ├── binary_reader.rs  # Endian-aware reading
│       └── string_pool.rs    # String interning
```

**Key Types:**
```rust
// Vector types (aligned for SIMD)
#[repr(C, align(16))]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Quaternion for rotations
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

// 4x4 transformation matrix (row-major)
#[repr(C)]
pub struct Mat4x4 {
    pub rows: [[f32; 4]; 4],
}
```

---

### `starbreaker-parsers`

**Purpose:** All file format parsers for Star Citizen/CryEngine formats.

```
starbreaker-parsers/
├── src/
│   ├── lib.rs             # Public exports, registry init
│   ├── traits.rs          # Parser, StreamingParser, RandomAccessParser
│   ├── registry.rs        # Dynamic parser registration
│   ├── p4k/               # P4K archive parser
│   │   ├── mod.rs         # P4kParser implementation
│   │   ├── archive.rs     # P4kArchive container
│   │   ├── entry.rs       # P4kEntry, P4kEntryInfo
│   │   └── compression.rs # P4kCompression helpers
│   ├── dcb/               # DataCore Binary parser
│   │   ├── mod.rs         # DcbParser implementation
│   │   ├── datacore.rs    # DataCore container
│   │   ├── records.rs     # Record, RecordValue
│   │   ├── structs.rs     # StructDef, PropertyDef, DataType
│   │   └── cryxml.rs      # CryXml fallback parser
│   ├── cgf/               # CryEngine Geometry parser
│   │   ├── mod.rs         # CgfParser implementation
│   │   ├── chunks.rs      # ChunkType, ChunkHeader
│   │   ├── mesh.rs        # Mesh, Vertex, Face
│   │   └── bones.rs       # Skeleton, Bone
│   ├── dds/               # DirectDraw Surface parser
│   │   ├── mod.rs
│   │   ├── header.rs      # DDS_HEADER, DDS_HEADER_DXT10
│   │   ├── format.rs      # DXGI_FORMAT handling
│   │   └── combiner.rs    # Split file reassembly
│   ├── mtl/               # Material parser
│   ├── soc/               # Scene Object Container
│   ├── socpak/            # SOC Package
│   ├── cga/               # Animation
│   ├── chr/               # Character
│   └── skin/              # Skinned mesh
```

**Core Traits:**

```rust
/// Base trait for all parsers
pub trait Parser: Send + Sync {
    type Output: Send + Sync;
    
    fn extensions(&self) -> &[&str];
    fn magic_bytes(&self) -> Option<&[u8]>;
    fn name(&self) -> &str;
    
    fn parse<R: Read + Seek>(&self, reader: R) -> ParseResult<Self::Output>;
    
    fn parse_with_options<R: Read + Seek>(
        &self,
        reader: R,
        options: &ParseOptions,
        progress: Option<ProgressCallback>,
    ) -> ParseResult<Self::Output>;
}

/// For archives with random access to entries
pub trait RandomAccessParser: Parser {
    type EntryId: Clone + Send;
    type EntryMeta: Send;
    
    fn list_entries<R: Read + Seek>(&self, reader: R) 
        -> ParseResult<Vec<(Self::EntryId, Self::EntryMeta)>>;
    
    fn extract_entry<R: Read + Seek>(
        &self, reader: R, entry_id: &Self::EntryId
    ) -> ParseResult<Vec<u8>>;
}

/// For incremental/streaming parsing
pub trait StreamingParser: Parser {
    type State: Send;
    
    fn begin_parse(&self, options: &ParseOptions) -> ParseResult<Self::State>;
    fn feed_data(&self, state: &mut Self::State, data: &[u8]) -> ParseResult<()>;
    fn finalize(&self, state: Self::State) -> ParseResult<Self::Output>;
}
```

**Parser Registry:**

```rust
// Dynamic parser registration
pub static GLOBAL_REGISTRY: Lazy<ParserRegistry> = Lazy::new(|| {
    let registry = ParserRegistry::new();
    // Built-in parsers registered here
    registry
});

// Usage
let parser = GLOBAL_REGISTRY.get_for_extension("p4k")?;
let parser = GLOBAL_REGISTRY.get_for_path(path)?;
```

---

### `starbreaker-vfs`

**Purpose:** Virtual file system abstraction over multiple data sources.

```
starbreaker-vfs/
├── src/
│   ├── lib.rs
│   ├── tree.rs        # VirtualFileSystem, directory tree
│   ├── node.rs        # VfsNode (file/directory)
│   ├── path.rs        # VfsPath, path resolution
│   ├── mount.rs       # MountPoint, mount operations
│   └── search.rs      # Search index, queries
```

**Key Abstractions:**

```rust
/// Mount point types
pub enum MountPoint {
    /// P4K archive mount
    P4k {
        archive: Arc<P4kArchive>,
        parser: Arc<P4kParser>,
        reader: Arc<Mutex<BufReader<File>>>,
    },
    /// Local filesystem
    FileSystem {
        root: PathBuf,
    },
    /// DCB database (virtual folders by struct type)
    DataCore {
        datacore: Arc<DataCore>,
    },
    /// Overlay multiple sources
    Overlay {
        layers: Vec<MountPoint>,
    },
}

/// Virtual file system
pub struct VirtualFileSystem {
    mounts: HashMap<String, MountPoint>,
    root: VfsNode,
}

impl VirtualFileSystem {
    pub fn mount(&mut self, path: &str, mount: MountPoint);
    pub fn unmount(&mut self, path: &str);
    pub fn read(&self, path: &str) -> Result<Vec<u8>>;
    pub fn list(&self, path: &str) -> Result<Vec<VfsNode>>;
    pub fn search(&self, query: &str) -> Vec<SearchResult>;
    pub fn exists(&self, path: &str) -> bool;
}
```

---

### `starbreaker-datacore`

**Purpose:** High-level game data extraction and query APIs.

```
starbreaker-datacore/
├── src/
│   ├── lib.rs
│   ├── items/
│   │   ├── mod.rs
│   │   ├── ships.rs       # Ship, ShipLoadout
│   │   ├── weapons.rs     # Weapon, WeaponStats
│   │   ├── armor.rs       # Armor, ArmorStats
│   │   ├── components.rs  # Component types
│   │   └── locations.rs   # Location, LandingZone
│   ├── lookup.rs          # EntityLookup, cross-references
│   ├── stats.rs           # Stat calculations
│   └── localization.rs    # Text extraction
```

**High-Level APIs:**

```rust
/// Ship data with calculated stats
pub struct Ship {
    pub name: String,
    pub manufacturer: String,
    pub class: ShipClass,
    pub size: ShipSize,
    pub crew: CrewRequirements,
    pub components: Vec<ComponentSlot>,
    pub hardpoints: Vec<Hardpoint>,
    pub stats: ShipStats,
}

impl Ship {
    pub fn from_datacore(dc: &DataCore, name: &str) -> Option<Self>;
    pub fn calculate_stats(&mut self);
    pub fn with_loadout(&self, loadout: &Loadout) -> ShipStats;
}

/// Entity lookup service
pub struct EntityLookup {
    datacore: Arc<DataCore>,
    ships: HashMap<String, Ship>,
    weapons: HashMap<String, Weapon>,
    // ... cached entities
}

impl EntityLookup {
    pub fn new(datacore: Arc<DataCore>) -> Self;
    pub fn get_ship(&self, name: &str) -> Option<&Ship>;
    pub fn search_ships(&self, query: &str) -> Vec<&Ship>;
    pub fn get_weapon(&self, name: &str) -> Option<&Weapon>;
}
```

---

### `starbreaker-export`

**Purpose:** Export game assets to standard formats.

```
starbreaker-export/
├── src/
│   ├── lib.rs
│   ├── fbx/
│   │   ├── mod.rs
│   │   ├── writer.rs      # FBX ASCII writer
│   │   ├── nodes.rs       # FBX node types
│   │   ├── geometry.rs    # Mesh export
│   │   ├── materials.rs   # Material export
│   │   └── skeleton.rs    # Bone/skin export
│   ├── gltf/
│   │   ├── mod.rs
│   │   └── exporter.rs    # glTF 2.0 export
│   ├── textures/
│   │   ├── mod.rs
│   │   ├── converter.rs   # DDS to PNG/TGA
│   │   └── formats.rs     # Format detection
│   └── json/
│       ├── mod.rs
│       └── serializers.rs # Game data to JSON
```

**Export Pipeline:**

```rust
/// Export configuration
pub struct ExportConfig {
    pub format: ExportFormat,
    pub include_textures: bool,
    pub include_materials: bool,
    pub include_skeleton: bool,
    pub texture_format: TextureFormat,
    pub coordinate_system: CoordinateSystem,
}

/// Model exporter
pub struct ModelExporter {
    config: ExportConfig,
}

impl ModelExporter {
    pub fn export_fbx(&self, model: &CgfModel, path: &Path) -> Result<()>;
    pub fn export_gltf(&self, model: &CgfModel, path: &Path) -> Result<()>;
}

/// Texture converter
pub struct TextureConverter;

impl TextureConverter {
    pub fn convert(
        dds_data: &[u8],
        output_format: TextureFormat,
    ) -> Result<Vec<u8>>;
    
    pub fn combine_split(parts: &[&[u8]]) -> Result<Vec<u8>>;
}
```

---

### `starbreaker-render`

**Purpose:** Real-time 3D rendering for preview.

```
starbreaker-render/
├── src/
│   ├── lib.rs
│   ├── renderer.rs    # Main Renderer struct
│   ├── camera.rs      # OrbitCamera, FlyCamera
│   ├── mesh.rs        # GpuMesh, upload to GPU
│   ├── texture.rs     # GpuTexture
│   ├── lighting.rs    # Lights, environment
│   └── shaders/
│       ├── mod.rs
│       ├── pbr.wgsl   # PBR shader
│       └── preview.wgsl # Simple preview shader
```

**Renderer Architecture:**

```rust
/// Main renderer using wgpu
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    pipeline: wgpu::RenderPipeline,
    camera: OrbitCamera,
    meshes: Vec<GpuMesh>,
    textures: HashMap<String, GpuTexture>,
}

impl Renderer {
    pub fn new(window: &Window) -> Result<Self>;
    pub fn load_model(&mut self, model: &CgfModel) -> Result<()>;
    pub fn render(&mut self, view: &egui::Rect);
    pub fn set_camera(&mut self, camera: OrbitCamera);
    pub fn resize(&mut self, width: u32, height: u32);
}

/// Orbit camera for model viewing
pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
}

impl OrbitCamera {
    pub fn orbit(&mut self, delta_x: f32, delta_y: f32);
    pub fn zoom(&mut self, delta: f32);
    pub fn pan(&mut self, delta_x: f32, delta_y: f32);
    pub fn view_matrix(&self) -> Mat4x4;
    pub fn projection_matrix(&self, aspect: f32) -> Mat4x4;
}
```

---

### `starbreaker-gui`

**Purpose:** Cross-platform GUI application.

```
starbreaker-gui/
├── src/
│   ├── lib.rs
│   ├── app.rs         # Main StarBreakerApp
│   ├── state.rs       # AppState, selection, etc.
│   ├── theme.rs       # Colors, fonts, styling
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── file_tree.rs     # File browser tree
│   │   ├── preview_3d.rs    # 3D model preview
│   │   ├── texture_view.rs  # Texture viewer
│   │   ├── data_table.rs    # DCB record table
│   │   ├── search_bar.rs    # Global search
│   │   └── export_dialog.rs # Export wizard
│   └── panels/
│       ├── mod.rs
│       ├── main_panel.rs    # Central content
│       ├── inspector.rs     # Property inspector
│       └── settings.rs      # Settings panel
```

**Application Structure:**

```rust
/// Main application
pub struct StarBreakerApp {
    state: AppState,
    vfs: VirtualFileSystem,
    renderer: Option<Renderer>,
    widgets: Widgets,
}

impl eframe::App for StarBreakerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Menu bar
        self.render_menu(ctx);
        
        // Left panel: file tree
        egui::SidePanel::left("files").show(ctx, |ui| {
            self.widgets.file_tree.show(ui, &self.vfs, &mut self.state);
        });
        
        // Right panel: inspector
        egui::SidePanel::right("inspector").show(ctx, |ui| {
            self.widgets.inspector.show(ui, &self.state);
        });
        
        // Central panel: preview
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.state.preview_mode {
                PreviewMode::Model => self.render_3d_preview(ui),
                PreviewMode::Texture => self.render_texture_preview(ui),
                PreviewMode::Text => self.render_text_preview(ui),
                PreviewMode::Hex => self.render_hex_preview(ui),
            }
        });
    }
}

/// Application state
pub struct AppState {
    pub selected_path: Option<String>,
    pub selected_record: Option<u64>,
    pub preview_mode: PreviewMode,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub export_queue: Vec<ExportJob>,
}
```

---

## Core Concepts

### Progress Reporting

All long-running operations support progress callbacks:

```rust
pub type ProgressCallback = Box<dyn Fn(ParseProgress) + Send + Sync>;

pub struct ParseProgress {
    pub phase: ParsePhase,
    pub bytes_processed: u64,
    pub total_bytes: Option<u64>,
    pub current_item: Option<String>,
    pub items_processed: u64,
    pub total_items: Option<u64>,
}

pub enum ParsePhase {
    ReadingHeader,
    Indexing,
    Decompressing,
    ParsingRecords,
    LinkingReferences,
    Validating,
    Complete,
}
```

### Parse Options

Configurable parsing behavior:

```rust
pub struct ParseOptions {
    pub strict_validation: bool,        // Full validation (slower)
    pub parse_nested: bool,             // Parse referenced files
    pub max_nesting_depth: u32,         // Prevent infinite recursion
    pub skip_unknown_chunks: bool,      // Ignore unknown data
    pub decompression_memory_limit: usize,
    pub use_memory_mapping: bool,
    pub memory_mapping_threshold: u64,
}
```

---

## Data Flow

### P4K Extraction Flow

```
┌──────────┐     ┌──────────────┐     ┌────────────────┐     ┌──────────┐
│ P4K File │────▶│ Parse EOCD   │────▶│ Parse Central  │────▶│ Build    │
│          │     │ (find end)   │     │ Directory      │     │ Index    │
└──────────┘     └──────────────┘     └────────────────┘     └────┬─────┘
                                                                  │
     ┌────────────────────────────────────────────────────────────┘
     │
     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ User Request │────▶│ Find Entry   │────▶│ Read Local   │
│ (path)       │     │ by Path      │     │ Header       │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
     ┌────────────────────────────────────────────┘
     │
     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Read         │────▶│ Decompress   │────▶│ Return Data  │
│ Compressed   │     │ (if needed)  │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
```

### DCB Query Flow

```
┌──────────┐     ┌──────────────┐     ┌──────────────┐
│ DCB File │────▶│ Parse Header │────▶│ Parse String │
│          │     │              │     │ Table        │
└──────────┘     └──────────────┘     └──────┬───────┘
                                              │
     ┌────────────────────────────────────────┘
     │
     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Parse Struct │────▶│ Parse Props  │────▶│ Parse        │
│ Definitions  │     │ Definitions  │     │ Records      │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
     ┌────────────────────────────────────────────┘
     │
     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Build        │────▶│ User Query   │────▶│ Return       │
│ Indices      │     │ (by struct,  │     │ Records      │
│              │     │  name, GUID) │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
```

### Model Export Flow

```
┌──────────┐     ┌──────────────┐     ┌──────────────┐
│ CGF File │────▶│ Parse Chunks │────▶│ Build Mesh   │
│          │     │              │     │ Structure    │
└──────────┘     └──────────────┘     └──────┬───────┘
                                              │
     ┌────────────────────────────────────────┘
     │
     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Load         │────▶│ Load         │────▶│ Transform    │
│ Materials    │     │ Textures     │     │ Coords       │
└──────────────┘     └──────────────┘     └──────┬───────┘
                                                  │
     ┌────────────────────────────────────────────┘
     │
     ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ FBX/glTF     │────▶│ Write        │────▶│ Output       │
│ Exporter     │     │ To Disk      │     │ File(s)      │
└──────────────┘     └──────────────┘     └──────────────┘
```

---

## File Format Details

### P4K Archive Structure

```
┌─────────────────────────────────────────────────────────────┐
│                      P4K File Layout                         │
├─────────────────────────────────────────────────────────────┤
│  Offset 0                                                    │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Local File Header 1 (30+ bytes)                        ││
│  │  - Signature: 0x04034B50                                ││
│  │  - Compression method, CRC, sizes                       ││
│  │  - Filename                                             ││
│  │  - Extra field (ZIP64 info)                             ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  File Data 1 (compressed)                               ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  Local File Header 2                                    ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  File Data 2                                            ││
│  │  ...                                                    ││
│  ├─────────────────────────────────────────────────────────┤│
│  │                                                         ││
│  │  Central Directory                                      ││
│  │  ┌─────────────────────────────────────────────────────┐││
│  │  │  CD Entry 1 (46+ bytes)                             │││
│  │  │  - Signature: 0x02014B50                            │││
│  │  │  - Version, flags, compression                      │││
│  │  │  - CRC, sizes, offset to local header               │││
│  │  │  - Filename, extra, comment                         │││
│  │  ├─────────────────────────────────────────────────────┤││
│  │  │  CD Entry 2                                         │││
│  │  │  ...                                                │││
│  │  └─────────────────────────────────────────────────────┘││
│  ├─────────────────────────────────────────────────────────┤│
│  │  ZIP64 End of Central Directory (56 bytes)              ││
│  │  - Signature: 0x06064B50                                ││
│  │  - 64-bit sizes and offsets                             ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  ZIP64 EOCD Locator (20 bytes)                          ││
│  │  - Signature: 0x07064B50                                ││
│  │  - Offset to ZIP64 EOCD                                 ││
│  ├─────────────────────────────────────────────────────────┤│
│  │  End of Central Directory (22 bytes)                    ││
│  │  - Signature: 0x06054B50                                ││
│  │  - Entry count, CD offset (may be 0xFFFFFFFF for ZIP64) ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### DCB File Structure

```
┌─────────────────────────────────────────────────────────────┐
│                      DCB File Layout                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Header (36 bytes)                                      ││
│  │  - Magic: "DCB1" (0x44434231)                           ││
│  │  - Version                                              ││
│  │  - Struct count, Property count, Record count           ││
│  │  - Offsets to each section                              ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  String Table                                           ││
│  │  - Count of strings                                     ││
│  │  - Array of offsets                                     ││
│  │  - Null-terminated string data                          ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Structure Definitions                                  ││
│  │  - Name offset, parent ID                               ││
│  │  - Property start/count                                 ││
│  │  - Size, flags                                          ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Property Definitions                                   ││
│  │  - Name offset, data type                               ││
│  │  - Struct ID (for complex types)                        ││
│  │  - Conversion flags                                     ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Record Data                                            ││
│  │  - Struct ID, name offset, GUID                         ││
│  │  - Property values (based on struct definition)         ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

### CGF File Structure

```
┌─────────────────────────────────────────────────────────────┐
│                      CGF File Layout                         │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Header                                                 ││
│  │  - Magic: "CryTek\0\0" / "#ivo" / "CrCh"               ││
│  │  - Version                                              ││
│  │  - Chunk count, chunk table offset                      ││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Chunk Table                                            ││
│  │  ┌─────────────────────────────────────────────────────┐││
│  │  │  Chunk Header (16-20 bytes each)                    │││
│  │  │  - Type, version, offset, ID, size                  │││
│  │  └─────────────────────────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────┘│
│  ┌─────────────────────────────────────────────────────────┐│
│  │  Chunk Data                                             ││
│  │  ┌─────────────────────────────────────────────────────┐││
│  │  │  Mesh Chunk (0x1000)                                │││
│  │  │  - Vertices, normals, UVs, faces                    │││
│  │  ├─────────────────────────────────────────────────────┤││
│  │  │  Node Chunk (0x100B)                                │││
│  │  │  - Transform, parent, mesh reference                │││
│  │  ├─────────────────────────────────────────────────────┤││
│  │  │  Material Chunk (0x100C)                            │││
│  │  │  - Shader, textures, parameters                     │││
│  │  ├─────────────────────────────────────────────────────┤││
│  │  │  CompiledBones (0xACDC0000)                         │││
│  │  │  - Bone hierarchy, transforms                       │││
│  │  ├─────────────────────────────────────────────────────┤││
│  │  │  CompiledMesh (0xCCCC0000)                          │││
│  │  │  - Optimized mesh data                              │││
│  │  └─────────────────────────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

---

## Design Patterns

### Factory Pattern (Parser Registry)

```rust
// Registration
GLOBAL_REGISTRY.register(
    ParserRegistrationBuilder::new()
        .id("cgf")
        .extensions(&["cgf", "cga"])
        .factory(|| CgfParser::new())
        .build()
)?;

// Usage
let parser = GLOBAL_REGISTRY.get_for_extension("cgf")?;
```

### Builder Pattern (Export Config)

```rust
let config = ExportConfigBuilder::new()
    .format(ExportFormat::Fbx)
    .include_textures(true)
    .coordinate_system(CoordinateSystem::Blender)
    .build();
```

### Strategy Pattern (Compression)

```rust
impl P4kCompression {
    pub fn decompress(data: &[u8], method: CompressionMethod, size: usize) 
        -> ParseResult<Vec<u8>> 
    {
        match method {
            CompressionMethod::Store => Ok(data.to_vec()),
            CompressionMethod::Deflate => Self::decompress_deflate(data, size),
            CompressionMethod::Zstd => Self::decompress_zstd(data, size),
            CompressionMethod::Lz4 => Self::decompress_lz4(data, size),
            CompressionMethod::Unknown(m) => Err(/* ... */),
        }
    }
}
```

### Observer Pattern (Progress Callbacks)

```rust
let progress = |p: ParseProgress| {
    println!("Phase: {:?}, Progress: {:.1}%", 
        p.phase, 
        p.percentage().unwrap_or(0.0) * 100.0
    );
};

parser.parse_with_options(reader, &options, Some(Box::new(progress)))?;
```

---

## Performance Considerations

### Memory Management

1. **Large Files**: Use memory mapping for files > 10MB
2. **Strings**: Intern repeated strings (DCB has many duplicates)
3. **Collections**: Use `SmallVec` for typically-small vectors
4. **Caching**: LRU cache for decompressed data

### Parallelism

1. **Chunk Parsing**: Parse CGF chunks in parallel with Rayon
2. **File Extraction**: Extract multiple P4K entries concurrently
3. **Texture Loading**: Load and decompress textures in parallel

### I/O Optimization

1. **Buffering**: Use `BufReader` for sequential reads
2. **Seek Reduction**: Sort operations by file offset
3. **Prefetching**: Read ahead for directory traversal

### Build Optimization

```toml
[profile.release]
lto = "fat"           # Link-time optimization
codegen-units = 1     # Better optimization, slower build
panic = "abort"       # Smaller binary
strip = true          # Remove symbols
opt-level = 3         # Maximum optimization
```

---

## Error Handling

### Error Types

```rust
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid magic bytes: expected {expected:?}, found {found:?}")]
    InvalidMagic { expected: Vec<u8>, found: Vec<u8> },

    #[error("Unsupported version: {version}")]
    UnsupportedVersion { version: u32 },

    #[error("Corrupted data at offset {offset}: {message}")]
    CorruptedData { offset: u64, message: String },

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Nested error in {context}: {source}")]
    Nested {
        context: String,
        #[source]
        source: Box<ParseError>,
    },
}
```

### Error Context

```rust
impl ParseError {
    pub fn with_context(self, context: impl Into<String>) -> Self {
        ParseError::Nested {
            context: context.into(),
            source: Box::new(self),
        }
    }
}

// Usage
mesh_parser.parse(data)
    .map_err(|e| e.with_context("parsing ship mesh"))?;
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let data = include_bytes!("../test_data/sample.cgf");
        let parser = CgfParser::new();
        let result = parser.parse(Cursor::new(data));
        assert!(result.is_ok());
    }

    #[test]
    fn test_roundtrip() {
        let original = Mesh::new("test");
        let exported = export_fbx(&original);
        let reimported = import_fbx(&exported);
        assert_eq!(original.vertex_count(), reimported.vertex_count());
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_compression_roundtrip(data: Vec<u8>) {
        let compressed = P4kCompression::compress(&data, CompressionMethod::Zstd)?;
        let decompressed = P4kCompression::decompress(
            &compressed, 
            CompressionMethod::Zstd, 
            data.len()
        )?;
        prop_assert_eq!(data, decompressed);
    }
}
```

### Integration Tests

```rust
// tests/integration/p4k_test.rs
#[test]
#[ignore] // Requires game files
fn test_parse_real_p4k() {
    let path = std::env::var("SC_PATH")
        .expect("SC_PATH must be set");
    let p4k_path = Path::new(&path).join("Data.p4k");
    
    let parser = P4kParser::new();
    let archive = parser.parse_file(&p4k_path).unwrap();
    
    assert!(archive.entry_count() > 100_000);
}
```

### Benchmarks

```rust
// benches/parsing.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_p4k_index(c: &mut Criterion) {
    let data = std::fs::read("test_data/sample.p4k").unwrap();
    
    c.bench_function("p4k_index", |b| {
        b.iter(|| {
            let parser = P4kParser::new();
            parser.parse(Cursor::new(&data)).unwrap()
        })
    });
}

criterion_group!(benches, benchmark_p4k_index);
criterion_main!(benches);
```

---

## Appendix: Chunk Type Reference

| ID | Name | Description |
|----|------|-------------|
| 0x0000 | SourceInfo | Source file information |
| 0x0001 | Timing | Animation timing |
| 0x0014 | MtlName | Material name reference |
| 0x1000 | Mesh | Raw mesh data |
| 0x1001 | MeshSubsets | Mesh subset definitions |
| 0x100B | Node | Scene node |
| 0x100C | Material | Material definition |
| 0x1016 | BoneAnim | Bone animation data |
| 0x1017 | BoneNameList | Bone names |
| 0x1018 | BoneInitialPos | Initial bone positions |
| 0x1019 | BoneMesh | Bone collision mesh |
| 0xACDC0000 | CompiledBones | Compiled skeleton |
| 0xACDC0001 | CompiledPhysicalBones | Physics bones |
| 0xCCCC0000 | CompiledMesh | Optimized mesh |

---

*Document version: 1.0 | Last updated: December 2024*