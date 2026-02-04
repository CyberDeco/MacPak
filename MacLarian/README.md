# MacLarian

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/maclarian.svg?color=68fdff)](https://crates.io/crates/maclarian)
[![Documentation](https://docs.rs/maclarian/badge.svg)](https://docs.rs/maclarian)
![Rust](https://img.shields.io/badge/rust-1.85+-orange?logo=rust)

</div>

A pure-Rust implementation of a ***[M]ac***OS-focused ***Larian*** file format library and toolkit for Baldur's Gate 3 file handling and modding.

> [!NOTE]
> This crate is in active development (0.x). The API may change between releases.

## Supported Formats

<div align="center">
    
| Format | Read | Write | Description |
|--------|:----:|:-----:|-----------|
| **PAK** | Yes | Yes | Extract, create, and list game asset packages |
| **LSF/LSX/LSJ** | Yes | Yes | Binary, XML, and JSON document formats |
| **LOCA** | Yes | Yes | Localization (language) files |
| **GR2** | Yes | Yes | Granny2 mesh files for BG3 |
| **glTF/GLB** | Yes | Yes | 3D model import/export for Blender |
| **DDS/PNG** | Yes | Yes | Texture conversion |
| **GTS/GTP** | Yes | Yes | GTS/GTP streaming virtual texture extraction/creation |

</div>

> [!CAUTION]
> Creating custom virtual textures (.gts/.gtp files) is not recommended for macOS because they need to be injected into the game using the [BG3 Script Extender](https://github.com/Norbyte/bg3se/blob/main/Docs/VirtualTextures.md), which is Windows-only. [BG3SE-macOS](https://github.com/tdimino/bg3se-macos) is a macOS port of the original Windows version, but it's in active development and custom virtual textures may not be fully supported yet.

## Platform Support

<div align="center">
    
| OS | Compatibility | 
|----------|:--------------:|
| **macOS (Apple Silicon)** | Yes (Built with M2 Max) |
| **macOS (Intel)** | Currently Testing (Intel iMac) | 
| **Windows 10 (Boot Camp)** | Currently Testing (Intel iMac) | 
| **Windows 11** | Currently Testing | 
| **Linux** | Unknown | 

</div>

The following are assumed to be the default BG3 install locations:
  - macOS: `~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/Baldur's Gate 3.app/Contents/Data`
  - Windows: `C:\Program Files (x86)\Steam\steamapps\common\Baldurs Gate 3\Data`
  - Linux: `~/.steam/steam/steamapps/common/Baldurs Gate 3/Data`

**Note:** MacLarian should work on all platforms. However, automatic BG3 game installation detection (`GameDataResolver::auto_detect()`) is limited to the paths above. If BG3 is installed in a different location (specifically the `\Data` subfolder), use `--bg3-path <path>` to specify the BG3 install folder manually.

## MacLarian Library

*To install the MacLarian library as a dependency crate, add this to your `Cargo.toml`:*

```toml
[dependencies]
maclarian = "0.1"
```

*If you don't want the CLI and only want the library, add this to your `Cargo.toml`:*

```toml
[dependencies]
maclarian = { version = "0.1", default-features = false }
```

<details>
  <summary><b>Working with PAK Files</b></summary>

```rust
use maclarian::pak::PakOperations;

// List contents of a PAK file
let files = PakOperations::list("Shared.pak")?;
println!("Found {} files", files.len());
for path in &files {
    println!("{path}");
}

// Extract an entire PAK file
PakOperations::extract("Shared.pak", "output/")?;

// Read a specific file from a PAK without extracting
let data = PakOperations::read_file_bytes("Shared.pak", "Public/Shared/meta.lsx")?;
```
</details>

<details>
  <summary><b>Converting LSF to LSX</b></summary>

```rust
use maclarian::converter::convert_lsf_to_lsx;

// Convert LSF (binary) to LSX (XML) file
convert_lsf_to_lsx("meta.lsf", "meta.lsx")?;
```
</details>

<details>
<summary><b>Converting GR2 to GLB/glTF</b></summary>

```rust
use maclarian::converter::gr2_gltf::convert_gr2_bytes_to_glb;

let gr2_data = std::fs::read("model.GR2")?;
let glb_data = convert_gr2_bytes_to_glb(&gr2_data)?;
std::fs::write("model.glb", glb_data)?;
```
</details>

<details>
  <summary><b>Using the Prelude</b></summary>
The prelude provides convenient access to commonly used types:

```rust
use maclarian::prelude::*;

// Now you have access to:
// - PakOperations, LsfDocument, LsxDocument, LsjDocument
// - VirtualTextureExtractor, GtsFile, GtpFile
// - Error, Result, and more
```
</details>
<br>

> [!IMPORTANT]
> [Documentation of the full API is in the MacLarian docs.rs →](https://docs.rs/maclarian)

---

## MacLarian CLI

*To use MacLarian as a CLI tool (assuming Rust is already installed):*

```
cargo install maclarian
```

*Run with:*

```bash
maclarian <command>
```

### Commands Overview

<div align="center">

| Command | Description |
|---------|-------------|
| `pak` | List, extract, and create PAK files |
| `convert` | Convert between file formats (LSF↔LSX↔LSJ, LOCA↔XML, DDS↔PNG) |
| `gr2` | Pair and extract textures for GR2 files, convert GR2↔glTF/GLB |
| `vt` | Virtual texture (GTS/GTP) extraction and creation |
| `mods` | Mod utilities (validation, info.json) |
| `loca` | Search within LOCA localization files |

</div>

> [!IMPORTANT]
> [The full list of CLI commands is in the wiki →](https://github.com/CyberDeco/MacPak/wiki/MacLarian-CLI-Commands)


## License

This project is licensed under the [PolyForm Noncommercial License 1.0.0](LICENSE).

Some files are dual-licensed under MIT/Apache-2.0 from upstream projects. See the LICENSE file for details.

## Credits

Most of the functionality in MacLarian is derived from:
- [LSLib](https://github.com/Norbyte/lslib) by Norbyte (MIT)
- [xiba](https://gitlab.com/saghm/xiba/) by saghm (Apache-2.0)
- [Knit](https://github.com/Legiayayana/Knit) by Legiayayana (MIT)
