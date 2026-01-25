# MacLarian

macOS-focused Larian file format library and toolkit for Baldur's Gate 3 modding.

MacLarian provides pure Rust implementations for reading and writing Larian Studios'
proprietary file formats, with no external binary dependencies (including no reliance
on granny2.dll for GR2 mesh decompression).

## Supported Formats

| Format | Read | Write | Description |
|--------|------|-------|-------------|
| **PAK** (LSPK) | Yes | Yes | Game archive format |
| **LSF** | Yes | Yes | Binary data format |
| **LSX** | Yes | Yes | XML data format |
| **LSJ** | Yes | Yes | JSON data format |
| **LOCA** | Yes | Yes | Localization format |
| **GR2** (Granny2) | Yes | Yes | 3D model format (decompression only) |
| **glTF/GLB** | Yes | Yes | 3D model import/export |
| **DDS** | Yes | Yes | Texture format |
| **GTS/GTP** | Yes | No | Virtual textures |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
maclarian = "0.1"
```

## Quick Start

### Reading a PAK Archive

```rust
use maclarian::pak::PakOperations;

// List contents of a PAK file
let contents = PakOperations::list("path/to/file.pak")?;
for entry in contents.files {
    println!("{}: {} bytes", entry.name, entry.size);
}

// Extract a specific file
let bytes = PakOperations::read_file_bytes("file.pak", "path/inside/pak.lsf")?;
```

### Converting LSF to LSX

```rust
use maclarian::formats::lsf::parse_lsf_bytes;
use maclarian::converter::lsf_lsx::to_lsx_string;

let data = std::fs::read("file.lsf")?;
let doc = parse_lsf_bytes(&data)?;
let xml = to_lsx_string(&doc)?;
```

### Converting GR2 to GLB/glTF

```rust
use maclarian::converter::gr2_gltf::convert_gr2_bytes_to_glb;

let gr2_data = std::fs::read("model.GR2")?;
let glb_data = convert_gr2_bytes_to_glb(&gr2_data)?;
std::fs::write("model.glb", glb_data)?;
```

### Working with Character Dialogs

```rust
use maclarian::dialog::parse_dialog_lsf_bytes;

let data = std::fs::read("dialog.lsf")?;
let dialog = parse_dialog_lsf_bytes(&data)?;

println!("Dialog has {} nodes", dialog.nodes.len());
```

## Prelude

For convenience, commonly used types are re-exported in the prelude:

```rust
use maclarian::prelude::*;
```

## Features

- `cli` - Enable the command-line interface binary
- `audio` - Enable WEM audio file operations
  - Requires vgmstream-cli for actual decoding, which can be installed via [Homebrew](https://brew.sh) with `brew install vgmstream`

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
| `wem` | WEM audio operations (requires `audio` feature) |

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
| `--game-data <path>` | Path to BG3 Data folder (for Textures.pak, Shared.pak) |
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
maclarian extract -s Models.pak -d ./models --bundle --game-data /path/to/Data

# Extract GR2s and convert textures to PNG
maclarian extract -s Models.pak -d ./models --bundle --game-data /path/to/Data --png
```

---

### `create` - Create PAK Archives

Create a PAK archive from a directory.

```bash
maclarian create -s <source_dir> -d <output.pak>
```

**Examples:**

```bash
maclarian create -s ./my_mod -d MyMod.pak
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
| `--game-data <path>` | Path to BG3 Data folder (for texture lookup) |
| `--virtual-textures <path>` | Path to pre-extracted virtual textures |
| `--no-glb` | Skip GLB/glTF conversion (only extract textures) |
| `--no-textures` | Skip texture extraction (only convert model) |
| `--gltf` | Output as glTF instead of GLB (.gltf + .bin files) |
| `--png` | Convert extracted DDS textures to PNG |
| `--keep-dds` | Keep original DDS files after PNG conversion |

**Examples:**

```bash
# Basic bundle (GLB + DDS textures)
maclarian gr2 bundle model.gr2 --game-data ~/BG3/Data

# Bundle as glTF with PNG textures
maclarian gr2 bundle model.gr2 --game-data ~/BG3/Data --gltf --png

# Bundle keeping both DDS and PNG
maclarian gr2 bundle model.gr2 --game-data ~/BG3/Data --png --keep-dds

# Only extract textures, no model conversion
maclarian gr2 bundle model.gr2 --game-data ~/BG3/Data --no-glb
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

Generate an `info.json` file for BaldursModManager.

```bash
maclarian mod info-json --pak <mod.pak> --extracted <mod_dir> [-o info.json]
```

---

### `wem` - Audio Operations

Requires the `audio` feature: `cargo build --features "cli,audio"`

#### `wem inspect` - Inspect WEM Header

Display WEM file header information.

```bash
maclarian wem inspect <file.wem>
```

#### `wem decode` - Decode to WAV

Decode a WEM file to WAV format (requires vgmstream-cli).

```bash
maclarian wem decode <file.wem> [-o output.wav]
```

**Options:**

| Flag | Description |
|------|-------------|
| `-o, --output <file>` | Output WAV file |
| `--silent` | Use silent fallback if vgmstream unavailable |

## License

This project is licensed under the [PolyForm Noncommercial License 1.0.0](../LICENSE).

Some files are dual-licensed under MIT/Apache-2.0 from upstream projects. See the LICENSE file for details.

## Credits

Derived from:
- [LSLib](https://github.com/Norbyte/lslib) by Norbyte (MIT)
- [xiba](https://gitlab.com/saghm/xiba/) by saghm (Apache-2.0)
- [Knit](https://github.com/Legiayayana/Knit) by Legiayayana (MIT)
