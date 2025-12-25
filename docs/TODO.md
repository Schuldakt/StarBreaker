# StarBreaker - Development TODO

> Last Updated: December 2025

This document tracks all development tasks, organized by priority and component. Check boxes indicate completion status.

---

## Table of Contents

- [Legend](#legend)
- [Critical Bug Fixes](#critical-bug-fixes)
- [Phase 1: Foundation](#phase-1-foundation-weeks-1-2)
- [Phase 2: Parser Completion](#phase-2-parser-completion-weeks-3-4)
- [Phase 3: Virtual File System](#phase-3-virtual-file-system-weeks-5-6)
- [Phase 4: Export Pipeline](#phase-4-export-pipeline-weeks-7-8)
- [Phase 5: GUI Application](#phase-5-gui-application-weeks-9-12)
- [Phase 6: Polish & Release](#phase-6-polish--release-weeks-13-14)
- [Optimization Tasks](#optimization-tasks)
- [Documentation Tasks](#documentation-tasks)
- [Future Features](#future-features-backlog)

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Complete |
| 🔨 | In Progress |
| 📋 | Planned |
| ⏸️ | On Hold |
| ❌ | Cancelled |
| 🐛 | Bug Fix |
| ⚡ | Performance |
| 🔒 | Security |

**Priority Levels:**
- 🔴 **Critical** - Blocks other work
- 🟠 **High** - Important for next release
- 🟡 **Medium** - Should be done soon
- 🟢 **Low** - Nice to have

---

## Critical Bug Fixes

> These must be fixed before any other work

- [x] ✅ **Fix duplicate import in lib.rs**
  - File: `crates/starbreaker-parsers/src/lib.rs`
  - Status: No duplicate imports found, exports are clean

- [x] ✅ **Add missing DataCore struct**
  - File: `crates/starbreaker-parsers/src/dcb/datacore.rs`
  - Status: DataCore, DataCoreHeader, and LazyDataCore fully implemented

- [x] ✅ **Fix incorrect test assertion**
  - File: `crates/starbreaker-parsers/src/p4k/entry.rs`
  - Status: Test correctly asserts `model.cgf` has extension `Some("cgf")`

- [x] ✅ **Export CGF module from lib.rs**
  - File: `crates/starbreaker-parsers/src/lib.rs`
  - Status: CGF module properly exported with all types

---

## Phase 1: Foundation (Weeks 1-2)

### Core Infrastructure

- [x] ✅ Set up workspace structure
- [x] ✅ Create all crate scaffolding
- [x] ✅ Define parser trait system (`traits.rs`)
- [x] ✅ Implement parser registry (`registry.rs`)
- [x] ✅ Set up error types with thiserror
- [ ] 📋 Add logging with tracing
- [ ] 📋 Set up CI/CD pipeline (GitHub Actions)

### P4K Parser (`starbreaker-parsers/src/p4k/`)

- [x] ✅ Parse End of Central Directory (EOCD)
- [x] ✅ Parse ZIP64 EOCD for large archives
- [x] ✅ Parse Central Directory entries
- [x] ✅ Extract local file headers
- [x] ✅ Implement Store (no compression)
- [x] ✅ Implement Deflate decompression
- [x] ✅ Implement ZStd decompression
- [x] ✅ Implement LZ4 decompression
- [x] ✅ CRC32 verification
- [x] ✅ Build path index for fast lookup
- [x] ✅ Directory tree builder
- [x] ✅ Pattern matching (glob-like)
- [ ] 📋 Progress reporting callback
- [ ] 📋 Memory-mapped I/O for large files
- [ ] 📋 Parallel entry extraction

### DCB Parser (`starbreaker-parsers/src/dcb/`)

- [x] ✅ Parse DCB header
- [x] ✅ Parse string table
- [x] ✅ Parse structure definitions
- [x] ✅ Parse property definitions
- [x] ✅ Parse records
- [x] ✅ All data types (bool, int, float, string, vec3, etc.)
- [x] ✅ Reference resolution
- [x] ✅ Build indices (struct, record)
- [x] ✅ Implement DataCore container struct
- [x] ✅ Lazy record loading (LazyDataCore with on-demand loading)
- [ ] 📋 Binary XML (CryXml) fallback parser
- [ ] 📋 String interning for memory efficiency

### Testing

- [ ] 📋 Unit tests for P4K parser
- [ ] 📋 Unit tests for DCB parser
- [ ] 📋 Integration tests with sample files
- [ ] 📋 Property-based tests with proptest
- [ ] 📋 Benchmarks with criterion

---

## Phase 2: Parser Completion (Weeks 3-4)

### CGF Parser (`starbreaker-parsers/src/cgf/`)

- [x] ✅ Parse file header (CryTek, Ivo, CrCh magic)
- [x] ✅ Parse chunk table
- [x] ✅ Define chunk types enum
- [x] ✅ Mesh structure (vertices, faces, subsets)
- [x] ✅ Vertex attributes (position, normal, UV, color, tangent)
- [x] ✅ Bone weights and indices
- [x] ✅ Skeleton structure
- [x] ✅ Bone hierarchy
- [x] ✅ Bone transforms (local, bind pose, inverse bind pose)
- [x] ✅ Bounding box calculations
- [x] ✅ Parse Mesh chunks (0x1000)
- [x] ✅ Parse Node chunks (0x100B)
- [x] ✅ Parse Material chunks (0x100C)
- [x] ✅ Parse CompiledBones (0xACDC0000)
- [x] ✅ Parse CompiledMesh (0xCCCC0000)
- [x] ✅ Parse MorphTargets (CompiledMorphTargets 0xACDC0002)
- [ ] 📋 Parse DataStream chunks
- [ ] 📋 Physics proxy parsing

### DDS Parser (`starbreaker-parsers/src/dds/`)

- [x] ✅ Parse DDS header
- [x] ✅ Parse DX10 extended header
- [x] ✅ Detect texture format (BC1-BC7, RGBA, etc.)
- [x] ✅ **Split file combiner** (.dds.1, .dds.2, etc.)
- [x] ✅ Mipmap level extraction
- [ ] 📋 Cubemap/array texture support

### Additional Parsers

- [ ] 📋 **MTL Parser** (XML material definitions)
  - [ ] Parse shader references
  - [ ] Parse texture slots
  - [ ] Parse shader parameters

- [ ] 📋 **SOC Parser** (Scene Object Container)
  - [ ] Parse scene hierarchy
  - [ ] Parse object transforms
  - [ ] Parse object references

- [ ] 📋 **SOCPAK Parser** (Packaged scenes)
  - [ ] Parse container structure
  - [ ] Extract embedded SOC files

- [ ] 📋 **CGA Parser** (Animation extension of CGF)
  - [ ] Parse animation controllers
  - [ ] Parse keyframes

- [ ] 📋 **CHR Parser** (Character)
  - [ ] Parse character-specific data
  - [ ] Parse attachment points

- [ ] 📋 **SKIN Parser** (Skinned mesh)
  - [ ] Parse skin-specific chunks

---

## Phase 3: Virtual File System (Weeks 5-6)

### VFS Core (`starbreaker-vfs/`)

- [x] ✅ Define VFS node structure
- [x] ✅ Define mount point abstraction
- [x] ✅ Implement path resolution
- [x] ✅ File/directory enumeration

### Mount Points

- [x] ✅ P4K archive mount (stub)
- [x] ✅ Local filesystem mount
- [ ] 📋 DCB virtual folders (by struct type)
- [ ] 📋 Overlay mount (multiple sources)

### Search & Index

- [ ] 📋 Full-text search index
- [ ] 📋 Metadata index (size, type, date)
- [ ] 📋 Regex pattern matching
- [ ] 📋 Search result caching

### File Operations

- [x] ✅ Read file contents
- [x] ✅ Stream large files
- [x] ✅ Extract to filesystem
- [x] ✅ Batch extraction

---

## Phase 4: Export Pipeline (Weeks 7-8)

### FBX Exporter (`starbreaker-export/src/fbx/`)

- [ ] 📋 🟠 FBX ASCII writer
- [ ] 📋 Geometry export (vertices, normals, UVs)
- [ ] 📋 Material export
- [ ] 📋 Skeleton/bone export
- [ ] 📋 Skin weights export
- [ ] 📋 Node hierarchy export
- [ ] 📋 Animation export (if applicable)

### glTF Exporter (`starbreaker-export/src/gltf/`)

- [x] ✅ glTF 2.0 JSON structure
- [x] ✅ Binary buffer generation (.bin)
- [x] ✅ GLB single-file export
- [x] ✅ Mesh primitives
- [x] ✅ PBR materials
- [ ] 📋 Skeleton/skin export
- [ ] 📋 Draco compression (optional)

### JSON Exporter (`starbreaker-export/src/json/`)

- [x] ✅ DCB DataCore export
- [x] ✅ Record export with property values
- [x] ✅ CGF mesh metadata export
- [x] ✅ P4K archive index export
- [x] ✅ Pretty-print and compact modes

### Texture Converter (`starbreaker-export/src/textures/`)

- [x] ✅ DDS to PNG conversion
- [x] ✅ DDS to TGA conversion
- [x] ✅ BC1-BC5 decompression
- [ ] 📋 BC6H/BC7 decompression (texpresso limitation)
- [x] ✅ Normal map handling (DX to OpenGL conversion)
- [x] ✅ Mipmap extraction
- [x] ✅ Batch conversion

### Data Exporters (`starbreaker-export/src/json/`)

- [ ] 📋 Ship data to JSON
- [ ] 📋 Weapon stats to JSON
- [ ] 📋 Item database export
- [ ] 📋 Localization export
- [ ] 📋 CSV export option

---

## Phase 5: GUI Application (Weeks 9-12)

### Framework Setup (`starbreaker-gui/`)

- [x] ✅ Set up egui + eframe
- [x] ✅ Application state management
- [x] ✅ Theme configuration (dark/light)
- [x] ✅ Keyboard shortcuts
- [x] ✅ Window management

### File Browser Panel

- [x] ✅ Tree view widget
- [x] ✅ Lazy loading for large directories
- [x] ✅ File type icons
- [ ] 📋 Context menu (extract, export, copy path)
- [ ] 📋 Drag and drop support
- [ ] 📋 Breadcrumb navigation

### Preview Panel

- [ ] 📋 Text file viewer
- [ ] 📋 Hex viewer for binary
- [ ] 📋 JSON/XML syntax highlighting
- [ ] 📋 Image viewer (DDS, PNG, etc.)

### 3D Preview (`starbreaker-render/`)

- [ ] 📋 wgpu renderer setup
- [ ] 📋 Camera controls (orbit, pan, zoom)
- [ ] 📋 Mesh rendering
- [ ] 📋 Wireframe mode
- [ ] 📋 Texture display
- [ ] 📋 Skeleton visualization
- [ ] 📋 Grid and axes helpers
- [ ] 📋 Lighting (basic 3-point)

### Inspector Panel

- [ ] 📋 Property grid widget
- [ ] 📋 DCB record display
- [ ] 📋 Mesh statistics
- [ ] 📋 Material properties
- [ ] 📋 Linked record navigation

### Search

- [ ] 📋 Global search bar
- [ ] 📋 Search results list
- [ ] 📋 Filter by type
- [ ] 📋 Recent searches

### Export Dialog

- [ ] 📋 Format selection
- [ ] 📋 Output path selection
- [ ] 📋 Options configuration
- [ ] 📋 Progress display
- [ ] 📋 Batch export queue

### Settings

- [ ] 📋 Game path configuration
- [ ] 📋 Default export settings
- [ ] 📋 Theme selection
- [ ] 📋 Keyboard shortcut customization
- [ ] 📋 Cache management

---

## Phase 6: Polish & Release (Weeks 13-14)

### Quality Assurance

- [ ] 📋 Full test coverage review
- [ ] 📋 Performance profiling
- [ ] 📋 Memory leak detection
- [ ] 📋 Cross-platform testing
- [ ] 📋 Accessibility review

### Packaging

- [ ] 📋 Windows installer (MSI/NSIS)
- [ ] 📋 macOS app bundle (.app)
- [ ] 📋 macOS universal binary (Intel + Apple Silicon)
- [ ] 📋 Linux AppImage
- [ ] 📋 Linux .deb package
- [ ] 📋 Portable ZIP releases

### Release

- [ ] 📋 Version tagging
- [ ] 📋 Changelog generation
- [ ] 📋 GitHub release automation
- [ ] 📋 Update checker in app

---

## Optimization Tasks

### Memory Optimizations

- [ ] ⚡ 🟡 String interning for DCB (`lasso` crate)
- [ ] ⚡ 🟡 SmallVec for vertex UVs
- [ ] ⚡ 🟡 Lazy record loading
- [ ] ⚡ 🟡 Decompression cache with LRU eviction
- [ ] ⚡ 🟢 Arena allocator for parsing

### CPU Optimizations

- [ ] ⚡ 🟠 Parallel chunk parsing (Rayon)
- [ ] ⚡ 🟠 Parallel file extraction
- [ ] ⚡ 🟡 SIMD for vertex processing
- [ ] ⚡ 🟢 Profile-guided optimization (PGO)

### I/O Optimizations

- [ ] ⚡ 🟠 Memory-mapped file support
- [ ] ⚡ 🟡 Buffered sequential reads
- [ ] ⚡ 🟡 Async I/O for GUI responsiveness
- [ ] ⚡ 🟢 Prefetching for tree navigation

### Build Optimizations

- [ ] ⚡ 🟡 LTO (Link-Time Optimization)
- [ ] ⚡ 🟡 Single codegen unit for release
- [ ] ⚡ 🟢 Strip symbols in release

---

## Documentation Tasks

### User Documentation

- [x] ✅ README.md
- [ ] 📋 Installation guide
- [ ] 📋 Quick start tutorial
- [ ] 📋 CLI command reference
- [ ] 📋 GUI user guide
- [ ] 📋 FAQ

### Developer Documentation

- [x] ✅ ARCHITECTURE.md
- [x] ✅ TODO.md (this file)
- [ ] 📋 CONTRIBUTING.md
- [ ] 📋 Code style guide
- [ ] 📋 Parser development guide
- [ ] 📋 Export format guide

### API Documentation

- [ ] 📋 Doc comments for all public APIs
- [ ] 📋 Usage examples in docs
- [ ] 📋 Module-level documentation
- [ ] 📋 Publish to docs.rs

### Format Documentation

- [ ] 📋 P4K format specification
- [ ] 📋 DCB format specification
- [ ] 📋 CGF format specification
- [ ] 📋 DDS handling notes

---

## Future Features (Backlog)

> These are ideas for post-1.0 releases

### Version Comparison Tool
- [ ] 📋 Compare two P4K archives
- [ ] 📋 Show added/removed/modified files
- [ ] 📋 Content diff for text files
- [ ] 📋 Export diff report

### Ship Loadout Builder
- [ ] 📋 Extract ship components from DCB
- [ ] 📋 Calculate combined stats
- [ ] 📋 Loadout presets
- [ ] 📋 Export loadout JSON

### Localization Tools
- [ ] 📋 Extract all game text
- [ ] 📋 Multi-language comparison
- [ ] 📋 Missing translation finder
- [ ] 📋 Export to translation formats

### WebAssembly Support
- [ ] 📋 Compile parsers to WASM
- [ ] 📋 Browser-based viewer
- [ ] 📋 Web API for queries

### Plugin System
- [ ] 📋 Plugin trait definition
- [ ] 📋 Dynamic loading
- [ ] 📋 Plugin manager
- [ ] 📋 Example plugins

### Advanced Features
- [ ] 📋 Audio file extraction (.wem)
- [ ] 📋 Video file extraction
- [ ] 📋 Shader decompilation
- [ ] 📋 Physics data extraction
- [ ] 📋 AI behavior tree parsing

---

## Progress Summary

| Phase | Status | Completion |
|-------|--------|------------|
| Bug Fixes | 🔨 In Progress | 0% |
| Phase 1: Foundation | 🔨 In Progress | 70% |
| Phase 2: Parsers | 🔨 In Progress | 40% |
| Phase 3: VFS | 📋 Planned | 0% |
| Phase 4: Export | 📋 Planned | 0% |
| Phase 5: GUI | 📋 Planned | 0% |
| Phase 6: Release | 📋 Planned | 0% |

**Overall Progress: ~25%**

---

## How to Contribute

1. Pick a task from this list
2. Comment on the related issue (or create one)
3. Fork the repository
4. Create a feature branch
5. Implement and test
6. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

*This document is updated regularly. Last review: December 2024*