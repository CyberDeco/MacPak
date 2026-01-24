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

- `audio` - Enable WEM audio file path detection
  - Requires vgmstream-cli for actual decoding, which can be installed via [Homebrew](https://brew.sh) with `brew install vgmstream`

## License

This project is licensed under the [PolyForm Noncommercial License 1.0.0](../LICENSE).

Some files are dual-licensed under MIT/Apache-2.0 from upstream projects. See the LICENSE file for details.

## Credits

Derived from:
- [LSLib](https://github.com/Norbyte/lslib) by Norbyte (MIT)
- [xiba](https://gitlab.com/saghm/xiba/) by saghm (Apache-2.0)
- [Knit](https://github.com/Legiayayana/Knit) by Legiayayana (MIT)
