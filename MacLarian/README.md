# MacLarian

![Rust](https://img.shields.io/badge/rust-1.85+-orange?logo=rust)
[![Crates.io](https://img.shields.io/crates/v/maclarian.svg)](https://crates.io/crates/maclarian)
[![Documentation](https://docs.rs/maclarian/badge.svg)](https://docs.rs/maclarian)
[![License](https://img.shields.io/badge/license-PolyForm%20Noncommercial-blue)](LICENSE)

A pure-Rust implementation of a ***[M]ac***OS-focused ***Larian*** file format library and toolkit for Baldur's Gate 3 file handling and modding.

> **Note:** This crate is in active development (0.x). The API may change between releases.

## Supported Formats

| Format | Read | Write | Description |
|--------|:----:|:-----:|-----------|
| **PAK** | Yes | Yes | Extract, create, and list game asset packages |
| **LSF/LSX/LSJ** | Yes | Yes | Binary, XML, and JSON document formats |
| **LOCA** | Yes | Yes | Localization (language) files |
| **GR2** | Yes | Yes | Granny2 mesh files for BG3 |
| **glTF/GLB** | Yes | Yes | 3D model import/export for Blender |
| **DDS/PNG** | Yes | Yes | Texture conversion |
| **GTS/GTP** | Yes | Yes | GTS/GTP streaming virtual texture extraction/creation |

*Note: creating custom virtual textures (.gts/.gtp files) is not recommended for macOS because they need to be injected into the game using the [BG3 Script Extender](https://github.com/Norbyte/bg3se/blob/main/Docs/VirtualTextures.md), which is Windows-only. [BG3SE-macOS](https://github.com/tdimino/bg3se-macos) is a macOS port of the original Windows version, but it's in active development and custom virtual textures may not be fully supported yet.*

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
maclarian = "0.1"
```

## Requirements

- **Rust 1.85+** (uses 2024 edition features)

## Platform Support

| Feature | macOS | Windows | Linux |
|---------|-------|---------|-------|
| Core library (format parsing) | Full | Full | Full |
| Game data auto-detection | Full | Limited | None |
| CLI tools | Full | Full | Full |

**Note:** The core library for reading/writing PAK, LSF, LSX, GR2, and other formats works on all platforms. However, automatic BG3 game installation detection (`GameDataResolver::auto_detect()`) currently only works reliably on macOS. Windows and Linux users should use `--bg3-path <path>` to specify the BG3 install folder manually.

## Quick Start

### Working with PAK Archives

```rust
use maclarian::pak::PakOperations;

// List contents of a PAK file
let contents = PakOperations::list("Shared.pak")?;
println!("Found {} files", files.len());
for entry in contents.files {
    println!("{}: {} bytes", entry.name, entry.size);
}

// Extract an entire PAK file
PakOperations::extract("Shared.pak", "output/")?;

// Extract a specific file from a PAK file
let data = PakOperations::read_file_bytes("Shared.pak", "Public/Shared/meta.lsx")?;
```

### Converting LSF to LSX

```rust
use maclarian::converter::convert_lsf_to_lsx;

// Convert LSF (binary) to LSX (XML) file
convert_lsf_to_lsx("meta.lsf", "meta.lsx")?;
```

### Converting GR2 to GLB/glTF

```rust
use maclarian::converter::gr2_gltf::convert_gr2_bytes_to_glb;

let gr2_data = std::fs::read("model.GR2")?;
let glb_data = convert_gr2_bytes_to_glb(&gr2_data)?;
std::fs::write("model.glb", glb_data)?;
```

### Using the Prelude

The prelude provides convenient access to commonly used types:

```rust
use maclarian::prelude::*;

// Now you have access to:
// - PakOperations, SearchIndex, FileType
// - LsfDocument, LsxDocument, LsjDocument
// - VirtualTextureExtractor, GtsFile, GtpFile
// - Error, Result, and more
```

## Features

- `cli` - Enables the `maclarian` command-line binary

## CLI Usage

Build with CLI support:

```bash
cargo build --release --features cli
```

Run with:

```bash
cargo run --features cli -- <command>
# or after building:
./target/release/maclarian <command>
```

### Commands Overview

| Command | Description |
|---------|-------------|
| `extract` | Extract files from PAK archives |
| `create` | Create PAK archives from directories |
| `list` | List contents of PAK archives |
| `convert` | Convert between file formats (LSF/LSX/LSJ) |
| `gr2` | GR2 model file operations |
| `vt` | Virtual texture (GTS/GTP) operations |
| `mod` | Mod utilities (validation, info.json) |
| `search` | Search PAK file contents |
| `index` | Build and manage search indexes |
| `pak` | PAK batch operations and info |
| `loca` | LOCA localization file operations |
| `texture` | Texture conversion (DDS/PNG) |

---

### `extract` - Extract PAK Archives

Extract files from a PAK archive with optional GR2 processing.

```bash
maclarian extract -s <source.pak> -d <destination>
```

**Options:**

| Flag | Description |
|------|-------------|
| `-s, --source` | Source PAK file |
| `-d, --destination` | Output directory |
| `--filter <pattern>` | Only extract files matching glob pattern (e.g., `*.lsf`) |
| `--file <path>` | Extract a single file by internal path |
| `-q, --quiet` | Suppress progress bar |

**GR2 Processing Options:**

| Flag | Description |
|------|-------------|
| `--bundle` | Enable all GR2 processing (GLB conversion + texture extraction) |
| `--convert-gr2` | Convert extracted GR2 files to GLB |
| `--extract-textures` | Extract DDS textures associated with GR2 files |
| `--extract-virtual-textures` | Extract virtual textures (GTex) for GR2 files |
| `--bg3-path <path>` | Path to BG3 install folder (for Textures.pak, Shared.pak) |
| `--virtual-textures <path>` | Path to pre-extracted virtual textures (GTS/GTP) |
| `--delete-gr2` | Delete original GR2 files after GLB conversion |
| `--png` | Convert extracted DDS textures to PNG |
| `--keep-dds` | Keep original DDS files after PNG conversion |

**Examples:**

```bash
# Extract entire PAK
maclarian extract -s Shared.pak -d ./extracted

# Extract only LSF files
maclarian extract -s Shared.pak -d ./extracted --filter "*.lsf"

# Extract with full GR2 processing
maclarian extract -s Models.pak -d ./models --bundle --bg3-path /path/to/Data

# Extract GR2s and convert textures to PNG
maclarian extract -s Models.pak -d ./models --bundle --bg3-path /path/to/Data --png
```

---

### `create` - Create PAK Archives

Create a PAK archive from a directory.

```bash
maclarian create -s <source_dir> -d <output.pak> [-c <compression>]
```

**Options:**

| Flag | Description |
|------|-------------|
| `-s, --source` | Source directory |
| `-d, --destination` | Output PAK file |
| `-c, --compression` | Compression method: `lz4` (default), `zlib`, `none` |

**Examples:**

```bash
# Create with default LZ4 compression
maclarian create -s ./my_mod -d MyMod.pak

# Create with Zlib compression
maclarian create -s ./my_mod -d MyMod.pak -c zlib

# Create uncompressed PAK
maclarian create -s ./my_mod -d MyMod.pak -c none
```

---

### `list` - List PAK Contents

List files inside a PAK archive.

```bash
maclarian list -s <source.pak>
```

**Options:**

| Flag | Description |
|------|-------------|
| `-s, --source` | PAK file to list |
| `-l, --detailed` | Show detailed info (sizes, compression ratio) |
| `--filter <pattern>` | Only list files matching glob pattern |
| `-c, --count` | Only show count of matching files |

**Examples:**

```bash
# List all files
maclarian list -s Shared.pak

# Count GR2 files
maclarian list -s Models.pak --filter "*.gr2" --count

# Detailed listing of LSF files
maclarian list -s Shared.pak --filter "*.lsf" -l
```

---

### `convert` - Format Conversion

Convert between LSF, LSX, and LSJ formats.

```bash
maclarian convert -s <source> -d <destination>
```

**Options:**

| Flag | Description |
|------|-------------|
| `-s, --source` | Source file |
| `-d, --destination` | Destination file |
| `-i, --input-format` | Input format (auto-detected from extension) |
| `-o, --output-format` | Output format (auto-detected from extension) |

**Examples:**

```bash
# LSF to LSX
maclarian convert -s meta.lsf -d meta.lsx

# LSX to LSF
maclarian convert -s meta.lsx -d meta.lsf

# LSF to JSON
maclarian convert -s data.lsf -d data.json
```

---

### `gr2` - GR2 Model Operations

Subcommands for working with GR2 (Granny2) model files.

#### `gr2 inspect` - Inspect GR2 Structure

Display detailed information about a GR2 file.

```bash
maclarian gr2 inspect <file.gr2>
```

#### `gr2 decompress` - Decompress GR2

Decompress all BitKnit-compressed sections in a GR2 file.

```bash
maclarian gr2 decompress <file.gr2> [-o output.gr2]
```

#### `gr2 to-glb` - Convert to GLB

Convert a GR2 file to binary glTF (GLB) format.

```bash
maclarian gr2 to-glb <file.gr2> [-o output.glb]
```

#### `gr2 from-gltf` - Convert from glTF

Convert a glTF/GLB file to GR2 format.

```bash
maclarian gr2 from-gltf <file.glb> [-o output.gr2]
```

#### `gr2 bundle` - Bundle with Textures

Convert GR2 to GLB/glTF and extract associated textures into a subdirectory.

```bash
maclarian gr2 bundle <file.gr2> [options]
```

**Options:**

| Flag | Description |
|------|-------------|
| `-o, --output <dir>` | Output directory (defaults to GR2 location) |
| `--bg3-path <path>` | Path to BG3 install folder (optional for macOS, required for Windows/Linux) |
| `--virtual-textures <path>` | Path to pre-extracted virtual textures |
| `--no-glb` | Skip GLB/glTF conversion (only extract textures) |
| `--no-textures` | Skip texture extraction (only convert model) |
| `--gltf` | Output as glTF instead of GLB (.gltf + .bin files) |
| `--png` | Convert extracted DDS textures to PNG |
| `--keep-dds` | Keep original DDS files after PNG conversion |

**Examples:**

```bash
# Basic bundle (GLB + DDS textures)
maclarian gr2 bundle model.gr2

# Bundle as glTF with PNG textures
maclarian gr2 bundle model.gr2 --gltf --png

# Bundle keeping both DDS and PNG
maclarian gr2 bundle model.gr2 --png --keep-dds

# Only extract textures, no model conversion
maclarian gr2 bundle model.gr2 --no-glb
```

#### `gr2 extract` - Extract to JSON

Extract mesh information to a JSON file.

```bash
maclarian gr2 extract <file.gr2> -o output.json
```

---

### `vt` - Virtual Texture Operations

Subcommands for working with virtual texture files (GTS/GTP).

#### `vt list` - List Textures

List all textures in a GTS file.

```bash
maclarian vt list <file.gts>
```

#### `vt extract` - Extract Textures

Extract textures from GTS/GTP files to DDS format.

```bash
maclarian vt extract <file.gts> -o <output_dir>
```

**Options:**

| Flag | Description |
|------|-------------|
| `-o, --output <dir>` | Output directory for DDS files |
| `--gtp-dir <path>` | Directory containing GTP files (defaults to GTS directory) |
| `-t, --texture <name>` | Extract only this texture by name |
| `-l, --layer <n>` | Layer index to extract |
| `-a, --all-layers` | Extract all layers (creates `_0`, `_1`, `_2` suffix files) |

**Examples:**

```bash
# Extract all textures
maclarian vt extract VirtualTextures.gts -o ./textures

# Extract specific texture
maclarian vt extract VirtualTextures.gts -o ./textures -t "my_texture"

# Extract all layers
maclarian vt extract VirtualTextures.gts -o ./textures --all-layers
```

#### `vt gtp-info` - GTP Page Info

Display information about a GTP page file.

```bash
maclarian vt gtp-info <file.gtp> [--gts <file.gts>]
```

---

### `mod` - Mod Utilities

Utilities for mod development.

#### `mod validate` - Validate Mod Structure

Check a mod directory for common issues.

```bash
maclarian mod validate -s <mod_directory>
```

#### `mod info-json` - Generate info.json

Generate an `info.json` file for [Baldur's Gate 3 Mod Manager](https://github.com/mkinfrared/baldurs-gate3-mod-manager).

```bash
maclarian mod info-json --pak <mod.pak> --extracted <mod_dir> [-o info.json]
```

---

### `search` - Search PAK Contents

Search for files within PAK archives.

#### `search filename` - Search by Filename

```bash
maclarian search filename <pak> <query> [-t <type>]
```

#### `search path` - Search by Path

```bash
maclarian search path <pak> <query> [-t <type>]
```

#### `search uuid` - Search by UUID

```bash
maclarian search uuid <pak> <uuid>
```

#### `search content` - Full-text Search

Search within file contents (slower, builds index first).

```bash
maclarian search content <pak> <query> [-l <limit>]
```

**Examples:**

```bash
# Find files containing "Barbarian" in filename
maclarian search filename Shared.pak "Barbarian"

# Find LSX files in a specific path
maclarian search path Shared.pak "Creatures" -t lsx

# Search file contents
maclarian search content Shared.pak "character ability" -l 20
```

---

### `index` - Search Index Management

Build and manage persistent search indexes for faster repeated searches.

#### `index build` - Build Index

```bash
maclarian index build <pak...> -o <output_dir> [--fulltext]
```

#### `index stats` - Show Index Statistics

```bash
maclarian index stats <index_dir>
```

**Examples:**

```bash
# Build index with full-text support
maclarian index build Shared.pak Gustav.pak -o ./my_index --fulltext

# Check index statistics
maclarian index stats ./my_index
```

---

### `pak` - PAK Utilities

Advanced PAK operations including batch processing and analysis.

#### `pak info` - PAK Statistics

Show detailed statistics about a PAK file.

```bash
maclarian pak info <pak>
```

Shows: file counts by type, compression ratio, largest files.

#### `pak find` - Find PAK Files

Find all PAK files in a directory.

```bash
maclarian pak find <directory>
```

#### `pak batch-extract` - Batch Extract

Extract multiple PAK files in parallel.

```bash
maclarian pak batch-extract -s <source_dir> -d <dest_dir>
```

#### `pak batch-create` - Batch Create

Create multiple PAK files from folders.

```bash
maclarian pak batch-create -s <source_dir> -d <dest_dir>
```

**Examples:**

```bash
# Show PAK statistics
maclarian pak info Shared.pak

# Find all PAKs in BG3 Data folder
maclarian pak find ~/BG3/Data

# Extract all PAKs in a directory
maclarian pak batch-extract -s ./paks -d ./extracted
```

---

### `loca` - Localization Operations

Work with LOCA localization files.

#### `loca list` - List Entries

```bash
maclarian loca list <file.loca> [-l <limit>]
```

#### `loca get` - Get Entry by Handle

```bash
maclarian loca get <file.loca> <handle>
```

#### `loca search` - Search Text

```bash
maclarian loca search <file.loca> <query> [-l <limit>]
```

#### `loca export` - Export to XML

```bash
maclarian loca export <file.loca> -o <output.xml>
```

**Examples:**

```bash
# List first 20 entries
maclarian loca list english.loca -l 20

# Search for specific text
maclarian loca search english.loca "Hello traveler"

# Export to XML for editing
maclarian loca export english.loca -o english.xml
```

---

### `texture` - Texture Operations

Convert between DDS and PNG texture formats.

#### `texture info` - Show DDS Info

```bash
maclarian texture info <file.dds>
```

Shows: dimensions, format, mip levels.

#### `texture convert` - Convert Texture

```bash
maclarian texture convert <input> <output> [-f <format>]
```

Formats for DDS output: `bc1`, `bc2`, `bc3` (default), `rgba`

#### `texture batch-convert` - Batch Convert

```bash
maclarian texture batch-convert -d <dir> -o <output_dir> -t <png|dds> [--dds-format <fmt>]
```

**Examples:**

```bash
# Show DDS info
maclarian texture info albedo.dds

# Convert DDS to PNG
maclarian texture convert albedo.dds albedo.png

# Convert PNG to DDS (BC3 compression)
maclarian texture convert albedo.png albedo.dds -f bc3

# Batch convert all DDS to PNG
maclarian texture batch-convert -d ./textures -o ./png_out -t png
```

## License

This project is licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE).

Some files are dual-licensed under MIT/Apache-2.0 from upstream projects. See the LICENSE file for details.

## Credits

Most of the functionality in MacLarian is derived from:
- [LSLib](https://github.com/Norbyte/lslib) by Norbyte (MIT)
- [xiba](https://gitlab.com/saghm/xiba/) by saghm (Apache-2.0)
- [Knit](https://github.com/Legiayayana/Knit) by Legiayayana (MIT)
