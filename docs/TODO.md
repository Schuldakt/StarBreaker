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
- [x] ✅ Add logging with tracing
  - File: `crates/starbreaker-parsers/src/logging.rs`
  - Feature-gated tracing support with configurable levels
- [x] ✅ Set up CI/CD pipeline (GitHub Actions)
  - File: `.github/workflows/ci.yml`
  - Multi-platform builds (Linux, Windows, macOS)
  - Automated testing, linting, and release builds

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

- [x] ✅ Unit tests for P4K parser
  - File: `crates/starbreaker-parsers/src/p4k/tests.rs`
  - Comprehensive tests for entries, archives, patterns, tree building
- [ ] 📋 Unit tests for DCB parser
- [ ] 📋 Integration tests with sample files
- [x] ✅ Property-based tests with proptest
- [x] ✅ Benchmarks with criterion
  - File: `crates/starbreaker-parsers/benches/parser_benchmarks.rs`

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
- [x] ✅ Unified error handling
- [x] ✅ Local filesystem mount
- [x] ✅ Multiple mount support

### Mount Points

- [x] ✅ P4K archive mount
  - File: `crates/starbreaker-vfs/src/mounts/p4k.rs`
  - Full implementation with LRU caching
- [x] ✅ Local filesystem mount
- [ ] 📋 DCB virtual folders (by struct type)
- [ ] 📋 Overlay mount (combine multiple sources)

### VFS Features

- [x] ✅ LRU decompression cache
- [ ] 📋 File watching for local mounts
- [ ] 📋 Write support for local mounts
- [ ] 📋 Async I/O support

---

## Phase 4: Export Pipeline (Weeks 7-8)

### Model Export (`starbreaker-export/`)

- [ ] 📋 glTF 2.0 exporter
  - [ ] Mesh geometry
  - [ ] Materials (PBR conversion)
  - [ ] Skeleton/bones
  - [ ] Animations
  - [ ] Binary (.glb) output
- [ ] 📋 FBX exporter
  - [ ] ASCII FBX format
  - [ ] Binary FBX format
- [ ] 📋 OBJ exporter (simple mesh only)

### Texture Export

- [ ] 📋 PNG export
- [ ] 📋 TGA export
- [ ] 📋 Keep original DDS option

### Data Export

- [ ] 📋 JSON export for DCB records
- [ ] 📋 CSV export for tabular data
- [ ] 📋 XML export

---

## Phase 5: GUI Application (Weeks 9-12)

### Main Window (`starbreaker-gui/`)

- [x] ✅ Basic egui/eframe setup
- [x] ✅ Theme configuration
- [ ] 📋 Menu bar (File, Edit, View, Tools, Help)
- [ ] 📋 Toolbar
- [ ] 📋 Status bar

### File Browser Panel

- [ ] 📋 Tree view for P4K contents
- [ ] 📋 List view alternative
- [ ] 📋 Breadcrumb navigation
- [ ] 📋 Context menus

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

- [x] ✅ Property grid widget
- [x] ✅ DCB record display
- [x] ✅ Mesh statistics
- [x] ✅ Material properties
- [ ] 📋 Linked record navigation

### Search

- [x] ✅ Global search bar
- [x] ✅ Search results list
- [x] ✅ Filter by type
- [ ] 📋 Recent searches

### Export Dialog

- [x] ✅ Format selection
- [x] ✅ Output path selection
- [x] ✅ Options configuration
- [ ] 📋 Progress display
- [ ] 📋 Batch export queue

### Settings

- [x] ✅ Game path configuration
- [x] ✅ Default export settings
- [x] ✅ Theme selection
- [x] ✅ Keyboard shortcut customization
- [ ] 📋 Cache management

### Debug Console

- [x] ✅ Toggleable debug console panel
- [x] ✅ Log message capture
- [x] ✅ Error display
- [ ] 📋 Command input
- [x] ✅ Copy to clipboard

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
- [x] ✅ GitHub release automation (in CI workflow)
- [ ] 📋 Update checker in app

---

## Optimization Tasks

### Memory Optimizations

- [ ] ⚡ 🟡 String interning for DCB (`lasso` crate) - Configured but not implemented
- [ ] ⚡ 🟡 SmallVec for vertex UVs
- [x] ⚡ 🟡 Lazy record loading - Implemented in LazyDataCore
- [x] ⚡ 🟡 Decompression cache with LRU eviction - In VFS P4K mount
- [ ] ⚡ 🟢 Arena allocator for parsing

### CPU Optimizations

- [ ] ⚡ 🟠 Parallel chunk parsing (Rayon) - Feature-gated, not implemented
- [ ] ⚡ 🟠 Parallel file extraction
- [ ] ⚡ 🟡 SIMD for vertex processing
- [ ] ⚡ 🟢 Profile-guided optimization (PGO)

### I/O Optimizations

- [ ] ⚡ 🟠 Memory-mapped file support - Feature-gated, not implemented
- [ ] ⚡ 🟡 Buffered sequential reads
- [ ] ⚡ 🟡 Async I/O for GUI responsiveness
- [ ] ⚡ 🟢 Prefetching for tree navigation

### Build Optimizations

- [x] ⚡ 🟡 LTO (Link-Time Optimization) - Configured in Cargo.toml
- [x] ⚡ 🟡 Single codegen unit for release - Configured in Cargo.toml
- [x] ⚡ 🟢 Strip symbols in release - Configured in Cargo.toml

---

## Documentation Tasks

### User Documentation

- [x] ✅ README.md
- [ ] 📋 Installation guide
- [ ] 📋 Quick start tutorial
- [x] ✅ CLI command reference - In CLI binary help
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

## CLI Tool

> Added: December 2025

- [x] ✅ CLI binary (`src/bin/starbreaker-cli.rs`)
- [x] ✅ List command - List P4K archive contents
- [x] ✅ Extract command - Extract files from P4K
- [x] ✅ Info command - Show file/archive information
- [x] ✅ Search command - Search for files
- [x] ✅ DCB command - Query DataCore database
- [x] ✅ Diff command - Compare two archives
- [x] ✅ Stats command - Show archive statistics
- [ ] 📋 Export command - Full implementation
- [ ] 📋 GUI launch command

---

## Future Features (Backlog)

> These are ideas for post-1.0 releases

### Version Comparison Tool
- [x] ✅ Compare two P4K archives (in CLI diff command)
- [x] ✅ Show added/removed/modified files
- [ ] 📋 Content diff for text files
- [x] ✅ Export diff report (JSON)

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
| Bug Fixes | ✅ Complete | 100% |
| Phase 1: Foundation | ✅ Complete | 95% |
| Phase 2: Parsers | 🔨 In Progress | 75% |
| Phase 3: VFS | ✅ Complete | 90% |
| Phase 4: Export | 📋 Planned | 5% |
| Phase 5: GUI | 🔨 In Progress | 30% |
| Phase 6: Release | 📋 Planned | 10% |

**Overall Progress: ~55%**

---

## Recent Updates

### December 2025
- Added GitHub Actions CI/CD workflow
- Implemented CLI tool with comprehensive commands
- Added tracing/logging support (feature-gated)
- Created P4K unit tests with proptest
- Implemented VFS with P4K mount and LRU caching
- Added unified error types in starbreaker-core
- Configured build optimizations (LTO, strip, codegen-units)
- Created benchmark suite

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

*This document is updated regularly. Last review: December 2025*