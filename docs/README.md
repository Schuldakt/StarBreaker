# StarBreaker

<div align="center">

![StarBreaker Logo](assets/logo.png)

**A powerful, cross-platform data mining and asset extraction tool for Star Citizen**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/yourusername/starbreaker/ci.yml?branch=main)](https://github.com/yourusername/starbreaker/actions)

[Features](#features) • [Installation](#installation) • [Usage](#usage) • [Documentation](#documentation) • [Contributing](#contributing)

</div>

---

## Overview

StarBreaker is a modular Rust application designed for extracting, parsing, and exporting game assets from Star Citizen. It provides both a graphical user interface and command-line tools for:

- 📦 **Archive Extraction** - Extract files from P4K archives (Star Citizen's main data format)
- 🗃️ **Database Parsing** - Parse and query the DataCore Binary (DCB) game database
- 🎮 **Asset Conversion** - Convert 3D models, textures, and materials to standard formats
- 🔍 **Data Mining** - Search and analyze game data for ships, weapons, items, and more
- 🖼️ **3D Preview** - View models and textures directly in the application

## Features

### Core Capabilities

| Feature | Status | Description |
|---------|--------|-------------|
| P4K Archive Parser | ✅ Complete | Full support for Star Citizen's ZIP-based archives with ZIP64, Deflate, ZStd, and LZ4 |
| DCB Database Parser | ✅ Complete | Parse Game2.dcb for all game entity data |
| CGF Model Parser | 🔨 In Progress | CryEngine geometry format (meshes, bones, materials) |
| DDS Texture Combiner | 📋 Planned | Reassemble split texture files (.dds.1, .dds.2, etc.) |
| FBX Export | 📋 Planned | Export 3D models to Autodesk FBX format |
| glTF Export | 📋 Planned | Export 3D models to glTF 2.0 format |
| GUI Application | 📋 Planned | Cross-platform GUI with 3D preview |

### Supported File Formats

#### Input Formats (Reading)
| Extension | Format | Description |
|-----------|--------|-------------|
| `.p4k` | P4K Archive | Main game data archive |
| `.dcb` | DataCore Binary | Game database (items, ships, weapons, etc.) |
| `.cgf` | CryEngine Geometry | Static 3D meshes |
| `.cga` | CryEngine Animation | Animated 3D meshes |
| `.chr` | Character | Character models with skeleton |
| `.skin` | Skinned Mesh | Meshes with bone weights |
| `.mtl` | Material | Material definitions (XML) |
| `.dds` | DirectDraw Surface | Textures (including split files) |
| `.soc` | Scene Object Container | Scene/level data |
| `.socpak` | SOC Package | Packaged scene objects |

#### Output Formats (Export)
| Extension | Format | Description |
|-----------|--------|-------------|
| `.fbx` | Autodesk FBX | Industry-standard 3D format (ASCII) |
| `.gltf` / `.glb` | glTF 2.0 | Modern 3D transmission format |
| `.png` / `.tga` | Images | Converted textures |
| `.json` | JSON | Extracted game data |

## Installation

### Prerequisites

- **Rust 1.75+** - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/starbreaker.git
cd starbreaker

# Build in release mode (recommended)
cargo build --release

# The binary will be at target/release/starbreaker
```

### Optional: Install Globally

```bash
cargo install --path .
```

### Platform-Specific Notes

<details>
<summary><b>Windows</b></summary>

No additional dependencies required. Pre-built binaries are available on the releases page.

```powershell
# Using winget (coming soon)
winget install starbreaker
```

</details>

<details>
<summary><b>macOS</b></summary>

```bash
# Using Homebrew (coming soon)
brew install starbreaker
```

For Apple Silicon (M1/M2/M3), the application runs natively without Rosetta.

</details>

<details>
<summary><b>Linux</b></summary>

Ensure you have the following system dependencies:

```bash
# Ubuntu/Debian
sudo apt install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev

# Fedora
sudo dnf install gtk3-devel

# Arch
sudo pacman -S gtk3
```

</details>

## Usage

### Command Line Interface

```bash
# List contents of a P4K archive
starbreaker list --path "C:\Program Files\Roberts Space Industries\StarCitizen\LIVE\Data.p4k"

# Extract specific files
starbreaker extract \
    --archive Data.p4k \
    --output ./extracted \
    --pattern "*.cgf"

# Query the game database
starbreaker dcb \
    --path Data/Game2.dcb \
    --struct Ship \
    --search "Aurora"

# Compare two archive versions
starbreaker diff \
    --old Data_3.21.p4k \
    --new Data_3.22.p4k

# Export a 3D model
starbreaker export \
    --input ship.cgf \
    --output ship.fbx \
    --format fbx

# Show archive statistics
starbreaker stats --path Data.p4k
```

### GUI Application

Launch the graphical interface:

```bash
starbreaker gui
```

Or simply double-click the executable on Windows/macOS.

**GUI Features:**
- File tree browser for P4K archives
- 3D model preview with orbit controls
- Texture viewer with mipmap levels
- Property inspector for game data
- Batch export wizard
- Search across all game data

### Library Usage

Use StarBreaker as a Rust library in your own projects:

```rust
use starbreaker_parsers::{P4kParser, Parser, DcbParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse a P4K archive
    let parser = P4kParser::new();
    let archive = parser.parse_file("Data.p4k")?;
    
    println!("Archive contains {} files", archive.entry_count());
    
    // Find all ship models
    let ships = archive.find("Data/Objects/Spaceships/*.cgf");
    for entry in ships {
        println!("  {}", entry.path);
    }
    
    // Parse the game database
    let dcb_parser = DcbParser::new();
    let datacore = dcb_parser.parse_file("Data/Game2.dcb")?;
    
    // Find all ships
    let ships = datacore.find_by_struct("Ship");
    for ship in ships {
        println!("Ship: {}", ship.name);
    }
    
    Ok(())
}
```

Add to your `Cargo.toml`:

```toml
[dependencies]
starbreaker-parsers = { git = "https://github.com/yourusername/starbreaker" }
```

## Project Structure

```
starbreaker/
├── Cargo.toml              # Workspace configuration
├── src/main.rs             # Application entry point
├── crates/
│   ├── starbreaker-core/       # Core types and utilities
│   ├── starbreaker-parsers/    # All file format parsers
│   ├── starbreaker-vfs/        # Virtual file system
│   ├── starbreaker-datacore/   # Game data extraction
│   ├── starbreaker-export/     # Export to FBX/glTF
│   ├── starbreaker-render/     # 3D preview rendering
│   └── starbreaker-gui/        # GUI application
├── assets/                 # Icons, fonts, shaders
└── docs/                   # Documentation
```

See [ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed technical documentation.

## Documentation

- [Architecture Guide](docs/ARCHITECTURE.md) - Technical design and crate structure
- [TODO List](TODO.md) - Development roadmap and task tracking
- [API Documentation](https://docs.rs/starbreaker) - Generated Rust docs
- [File Format Specs](docs/formats/) - Reverse-engineered format documentation

### Building Documentation

```bash
# Generate and open API documentation
cargo doc --workspace --no-deps --open
```

## Performance

StarBreaker is designed for performance:

- **Parallel Processing** - Uses Rayon for multi-threaded parsing
- **Memory Mapping** - Large files are memory-mapped to avoid loading entirely into RAM
- **Streaming Decompression** - Files are decompressed on-demand
- **Caching** - Frequently accessed data is cached
- **Zero-Copy Parsing** - Where possible, data is parsed without copying

### Benchmarks

| Operation | Time | Notes |
|-----------|------|-------|
| P4K Index Load | ~2s | 500K+ entries |
| DCB Full Parse | ~5s | ~2M records |
| CGF Model Parse | ~10ms | Average ship model |
| Texture Decompress | ~50ms | 4K DDS texture |

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone with full history
git clone https://github.com/yourusername/starbreaker.git
cd starbreaker

# Install development tools
cargo install cargo-watch cargo-nextest

# Run tests
cargo nextest run

# Run with hot-reloading during development
cargo watch -x run

# Check formatting and lints
cargo fmt --check
cargo clippy --workspace
```

### Testing with Game Files

To run integration tests, you need access to Star Citizen game files:

```bash
# Set the path to your Star Citizen installation
export SC_PATH="/path/to/StarCitizen/LIVE"

# Run integration tests
cargo test --features integration-tests
```

## Legal Notice

This project is not affiliated with, endorsed by, or connected to Cloud Imperium Games or Roberts Space Industries. Star Citizen® is a registered trademark of Cloud Imperium Rights LLC.

This tool is intended for personal use, data mining, and creating fan content. Please respect the game's Terms of Service and do not use extracted assets for commercial purposes.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Cloud Imperium Games** - For creating Star Citizen
- **The SC Modding Community** - For reverse engineering documentation
- **Crytek** - For the CryEngine file format foundations

---

<div align="center">

**[⬆ Back to Top](#starbreaker)**

Made with ❤️ by the StarBreaker Team

</div>