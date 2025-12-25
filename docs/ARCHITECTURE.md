stardust/
├── Cargo.toml
├── Cargo.lock
├── build.rs                          # Build script for cross-compilation
├── assets/
│   ├── icons/
│   ├── fonts/
│   └── shaders/
├── crates/
│   ├── stardust-core/               # Core parsing and data types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types/
│   │       │   ├── mod.rs
│   │       │   ├── vector.rs
│   │       │   ├── quaternion.rs
│   │       │   ├── matrix.rs
│   │       │   └── bounds.rs
│   │       ├── compression/
│   │       │   ├── mod.rs
│   │       │   ├── zlib.rs
│   │       │   ├── lz4.rs
│   │       │   └── zstd.rs
│   │       └── utils/
│   │           ├── mod.rs
│   │           ├── binary_reader.rs
│   │           └── string_pool.rs
│   │
│   ├── stardust-parsers/            # All file format parsers
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs            # Parser trait definitions
│   │       ├── registry.rs          # Parser registry/factory
│   │       ├── p4k/
│   │       │   ├── mod.rs
│   │       │   ├── archive.rs
│   │       │   ├── entry.rs
│   │       │   └── compression.rs
│   │       ├── socpak/
│   │       │   ├── mod.rs
│   │       │   ├── container.rs
│   │       │   └── object.rs
│   │       ├── soc/
│   │       │   ├── mod.rs
│   │       │   └── scene.rs
│   │       ├── dcb/
│   │       │   ├── mod.rs
│   │       │   ├── datacore.rs
│   │       │   ├── records.rs
│   │       │   ├── structs.rs
│   │       │   └── cryxml.rs
│   │       ├── mtl/
│   │       │   ├── mod.rs
│   │       │   └── material.rs
│   │       ├── cgf/
│   │       │   ├── mod.rs
│   │       │   ├── mesh.rs
│   │       │   ├── chunks.rs
│   │       │   └── bones.rs
│   │       ├── cga/
│   │       │   ├── mod.rs
│   │       │   └── animation.rs
│   │       ├── skin/
│   │       │   ├── mod.rs
│   │       │   └── skinned_mesh.rs
│   │       ├── chr/
│   │       │   ├── mod.rs
│   │       │   └── character.rs
│   │       └── dds/
│   │           ├── mod.rs
│   │           ├── header.rs
│   │           ├── formats.rs
│   │           └── combiner.rs
│   │
│   ├── stardust-vfs/                # Virtual File System
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tree.rs
│   │       ├── node.rs
│   │       ├── path.rs
│   │       ├── mount.rs
│   │       └── search.rs
│   │
│   ├── stardust-datacore/           # Game data extraction
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── items/
│   │       │   ├── mod.rs
│   │       │   ├── ships.rs
│   │       │   ├── weapons.rs
│   │       │   ├── armor.rs
│   │       │   ├── components.rs
│   │       │   └── locations.rs
│   │       ├── lookup.rs
│   │       ├── stats.rs
│   │       └── localization.rs
│   │
│   ├── stardust-export/             # Export functionality
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── fbx/
│   │       │   ├── mod.rs
│   │       │   ├── writer.rs
│   │       │   ├── nodes.rs
│   │       │   ├── geometry.rs
│   │       │   ├── materials.rs
│   │       │   └── skeleton.rs
│   │       ├── gltf/
│   │       │   ├── mod.rs
│   │       │   └── exporter.rs
│   │       ├── textures/
│   │       │   ├── mod.rs
│   │       │   ├── converter.rs
│   │       │   └── formats.rs
│   │       └── json/
│   │           ├── mod.rs
│   │           └── serializers.rs
│   │
│   ├── stardust-render/             # 3D preview rendering
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs
│   │       ├── camera.rs
│   │       ├── mesh.rs
│   │       ├── texture.rs
│   │       ├── lighting.rs
│   │       └── shaders/
│   │           ├── mod.rs
│   │           ├── pbr.wgsl
│   │           └── preview.wgsl
│   │
│   └── stardust-gui/                # GUI application
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── app.rs
│           ├── state.rs
│           ├── theme.rs
│           ├── widgets/
│           │   ├── mod.rs
│           │   ├── file_tree.rs
│           │   ├── preview_3d.rs
│           │   ├── texture_view.rs
│           │   ├── data_table.rs
│           │   ├── search_bar.rs
│           │   └── export_dialog.rs
│           └── panels/
│               ├── mod.rs
│               ├── main_panel.rs
│               ├── inspector.rs
│               └── settings.rs
│
└── src/
    └── main.rs                       # Application entry point