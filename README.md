# MacPak - A BG3 Modding Toolkit for macOS

<p align="center">
    <a href="https://github.com/CyberDeco/MacPak/releases/latest">
        <img src="MacPak/src/gui/assets/icons/icon.iconset/icon_512x512.png" alt="MacPak Icon" width="128">
    </a>
</p>

---

<p align="center">
<i>Baldurs Gate 3 modding tools for macOS? In <b>this</b> economy?</i>
</p>

---
<div align="center">
    
[![Custom](https://img.shields.io/badge/MacPak-v0.1.0-7706be?logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48IS0tIFVwbG9hZGVkIHRvOiBTVkcgUmVwbywgd3d3LnN2Z3JlcG8uY29tLCBHZW5lcmF0b3I6IFNWRyBSZXBvIE1peGVyIFRvb2xzIC0tPg0KPHN2ZyB3aWR0aD0iODAwcHgiIGhlaWdodD0iODAwcHgiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4NCjxwYXRoIGQ9Ik0yMSAxMi40OTg0TDE0IDEyLjQ5ODRNMyAxMi40OTg0TDEwIDEyLjQ5ODRNNyA0LjVWMTkuNDk4NE0xNyA0LjVWMTkuNDk4NE02LjIgMTkuNUgxNy44QzE4LjkyMDEgMTkuNSAxOS40ODAyIDE5LjUgMTkuOTA4IDE5LjI4MkMyMC4yODQzIDE5LjA5MDMgMjAuNTkwMyAxOC43ODQzIDIwLjc4MiAxOC40MDhDMjEgMTcuOTgwMiAyMSAxNy40MjAxIDIxIDE2LjNWMTAuOUMyMSA4LjY1OTc5IDIxIDcuNTM5NjggMjAuNTY0IDYuNjg0MDRDMjAuMTgwNSA1LjkzMTM5IDE5LjU2ODYgNS4zMTk0NyAxOC44MTYgNC45MzU5N0MxNy45NjAzIDQuNSAxNi44NDAyIDQuNSAxNC42IDQuNUg5LjRDNy4xNTk3OSA0LjUgNi4wMzk2OCA0LjUgNS4xODQwNCA0LjkzNTk3QzQuNDMxMzkgNS4zMTk0NyAzLjgxOTQ3IDUuOTMxMzkgMy40MzU5NyA2LjY4NDA0QzMgNy41Mzk2OCAzIDguNjU5NzkgMyAxMC45VjE2LjNDMyAxNy40MjAxIDMgMTcuOTgwMiAzLjIxNzk5IDE4LjQwOEMzLjQwOTczIDE4Ljc4NDMgMy43MTU2OSAxOS4wOTAzIDQuMDkyMDIgMTkuMjgyQzQuNTE5ODQgMTkuNSA1LjA3OTg5IDE5LjUgNi4yIDE5LjVaTTEwIDEwLjQ5ODRIMTRWMTQuNDk4NEgxMFYxMC40OTg0WiIgc3Ryb2tlPSIjRkZGRkZGIiBzdHJva2Utd2lkdGg9IjEuNSIgc3Ryb2tlLWxpbmVqb2luPSJyb3VuZCIvPg0KPC9zdmc+DQo=)](https://github.com/CyberDeco/MacPak/releases/latest)
[![Custom](https://img.shields.io/badge/MacLarian-v0.1.0-68fdff?logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz48IS0tIFVwbG9hZGVkIHRvOiBTVkcgUmVwbywgd3d3LnN2Z3JlcG8uY29tLCBHZW5lcmF0b3I6IFNWRyBSZXBvIE1peGVyIFRvb2xzIC0tPgo8c3ZnIGZpbGw9IiNGRkZGRkYiIHdpZHRoPSI4MDBweCIgaGVpZ2h0PSI4MDBweCIgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+PHRpdGxlPmlvbmljb25zLXY1LWg8L3RpdGxlPjxwYXRoIGQ9Ik00OTQuMjYsMjc2LjIyYy0zLjYtNDAuNDEtOS41My00OC4yOC0xMS43Ny01MS4yNC01LjE1LTYuODQtMTMuMzktMTEuMzEtMjIuMTEtMTZsMCwwYTMuNiwzLjYsMCwwLDEtLjkxLTUuNjhBMTUuOTMsMTUuOTMsMCwwLDAsNDY0LDE5MC43NywxNi4yNywxNi4yNywwLDAsMCw0NDcuNjUsMTc2aC0xNS42YTE3LDE3LDAsMCwwLTIsLjEzLDguNSw4LjUsMCwwLDAtMS40MS0uNDdsMCwwYy05LjI0LTE5LjUzLTIxLjg5LTQ2LjI3LTQ4LjExLTU5LjMyQzM0MS42NCw5NywyNzAsOTYsMjU2LDk2cy04NS42NCwxLTEyNC40OCwyMC4zMWMtMjYuMjIsMTMuMDUtMzguODcsMzkuNzktNDguMTEsNTkuMzJsLS4wOC4xNmE2LjUyLDYuNTIsMCwwLDAtMS4zNS4zNCwxNywxNywwLDAsMC0yLS4xM0g2NC4zNUExNi4yNywxNi4yNywwLDAsMCw0OCwxOTAuNzdhMTUuOTMsMTUuOTMsMCwwLDAsNC41OSwxMi40NywzLjYsMy42LDAsMCwxLS45MSw1LjY4bDAsMGMtOC43Miw0LjcyLTE3LDkuMTktMjIuMTEsMTYtMi4yNCwzLTguMTYsMTAuODMtMTEuNzcsNTEuMjQtMiwyMi43NC0yLjMsNDYuMjgtLjczLDYxLjQ0LDMuMjksMzEuNSw5LjQ2LDUwLjU0LDkuNzIsNTEuMzNhMTYsMTYsMCwwLDAsMTMuMiwxMC44N2gwVjQwMGExNiwxNiwwLDAsMCwxNiwxNmg1NmExNiwxNiwwLDAsMCwxNi0xNmgwYzguNjEsMCwxNC42LTEuNTQsMjAuOTUtMy4xOGExNTguODMsMTU4LjgzLDAsMCwxLDI4LTQuOTFDMjA3LjQ1LDM4OSwyMzcuNzksMzg4LDI1NiwzODhjMTcuODQsMCw0OS41MiwxLDgwLjA4LDMuOTFhMTU5LjE2LDE1OS4xNiwwLDAsMSwyOC4xMSw0LjkzYzYuMDgsMS41NiwxMS44NSwzLDE5Ljg0LDMuMTVoMGExNiwxNiwwLDAsMCwxNiwxNmg1NmExNiwxNiwwLDAsMCwxNi0xNnYtLjEyaDBBMTYsMTYsMCwwLDAsNDg1LjI3LDM4OWMuMjYtLjc5LDYuNDMtMTkuODMsOS43Mi01MS4zM0M0OTYuNTYsMzIyLjUsNDk2LjI4LDI5OSw0OTQuMjYsMjc2LjIyWk0xMTIuMzMsMTg5LjMxYzgtMTcsMTcuMTUtMzYuMjQsMzMuNDQtNDQuMzUsMjMuNTQtMTEuNzIsNzIuMzMtMTcsMTEwLjIzLTE3czg2LjY5LDUuMjQsMTEwLjIzLDE3YzE2LjI5LDguMTEsMjUuNCwyNy4zNiwzMy40NCw0NC4zNWwxLDIuMTdhOCw4LDAsMCwxLTcuNDQsMTEuNDJDMzYwLDIwMiwyOTAsMTk5LjEyLDI1NiwxOTkuMTJzLTEwNCwyLjk1LTEzNy4yOCwzLjg1YTgsOCwwLDAsMS03LjQ0LTExLjQyQzExMS42MywxOTAuODEsMTEyLDE5MC4wNiwxMTIuMzMsMTg5LjMxWm0xMS45Myw3OS42M0E0MjcuMTcsNDI3LjE3LDAsMCwxLDcyLjQyLDI3MmMtMTAuNiwwLTIxLjUzLTMtMjMuNTYtMTIuNDQtMS4zOS02LjM1LTEuMjQtOS45Mi0uNDktMTMuNTFDNDksMjQzLDUwLDI0MC43OCw1NSwyNDBjMTMtMiwyMC4yNy41MSw0MS41NSw2Ljc4LDE0LjExLDQuMTUsMjQuMjksOS42OCwzMC4wOSwxNC4wNkMxMjkuNTUsMjYzLDEyOCwyNjguNjQsMTI0LjI2LDI2OC45NFptMjIxLjM4LDgyYy0xMy4xNiwxLjUtMzkuNDguOTUtODkuMzQuOTVzLTc2LjE3LjU1LTg5LjMzLS45NWMtMTMuNTgtMS41MS0zMC44OS0xNC4zNS0xOS4wNy0yNS43OSw3Ljg3LTcuNTQsMjYuMjMtMTMuMTgsNTAuNjgtMTYuMzVTMjMzLjM4LDMwNCwyNTYuMiwzMDRzMzIuMTIsMSw1Ny42Miw0LjgxLDQ0Ljc3LDkuNTIsNTAuNjgsMTYuMzVDMzc1LjI4LDMzNy40LDM1OS4yMSwzNDkuMzUsMzQ1LjY0LDM1MVptMTE3LjUtOTEuMzljLTIsOS40OC0xMywxMi40NC0yMy41NiwxMi40NGE0NTUuOTEsNDU1LjkxLDAsMCwxLTUyLjg0LTMuMDZjLTMuMDYtLjI5LTQuNDgtNS42Ni0xLjM4LTguMSw1LjcxLTQuNDksMTYtOS45MSwzMC4wOS0xNC4wNiwyMS4yOC02LjI3LDMzLjU1LTguNzgsNDQuMDktNi42OSwyLjU3LjUxLDMuOTMsMy4yNyw0LjA5LDVBNDAuNjQsNDAuNjQsMCwwLDEsNDYzLjE0LDI1OS41NloiLz48L3N2Zz4K)](https://github.com/CyberDeco/MacPak/tree/main/MacLarian)

</div>

<ins>**MacPak**</ins> (**[M]ac**OS + .**[P]ak** files): a self-contained BG3 modding toolkit app designed specifically for macOS modders.

<ins>**MacLarian**</ins> (**[M]ac**OS + **Larian**): the, ahem, *engine* behind MacPak. MacLarian is also available as a standalone Rust library + CLI on [crates.io](https://crates.io/crates/maclarian).

---

## Features

| MacPak Tab | General Purpose |
|:----------:|--------------|
| **Browser** | In-app file browser and previewer for text, image, and model files |
| **Editor** | In-app, multi-tab text file editor and converter (TXT, XML, LOCA, LSF, LSX, LSJ, and JSON) |
| **PAK Ops** | Browse, extract, and create .pak archives |
| **Models** | GR2 ↔ glTF/GLB extraction, conversion, and texture matching |
| **Textures** | Virtual texture (GTS/GTP) extraction, DDS ↔ PNG conversion |
| **Dye Lab** | Import/export clothing and armor dye mods and create custom color palettes |
| **Search** | Search across PAK contents (file name + contents both supported) |
| **Dialogue** | Search and view character dialogue trees and play linked audio |

> [!IMPORTANT]
> [Full documentation is in the wiki →](https://github.com/CyberDeco/MacPak/wiki) (Coming Soon)

---

## How to Install

1. **Download the MacPak macOS app from [Releases](https://github.com/CyberDeco/MacPak/releases/latest)** (Coming Soon)
2. That's it.

<details>
  <summary><h3>Standalone MacLarian Install</h3></summary>
To install the MacLarian library as a dependency crate, add this to your `Cargo.toml`:

```toml
[dependencies]
maclarian = "0.1.0"
```

To use MacLarian as a CLI tool (need to have Rust installed first):

```
cargo install maclarian

# Extract entire PAK
maclarian pak extract -s Shared.pak -d ./extracted

# Extract only LSF files
maclarian pak extract -s Shared.pak -d ./extracted --filter "*.lsf"
```

See the [MacLarian README](MacLarian/README.md) and [CLI section of the wiki](https://github.com/CyberDeco/MacPak/wiki/MacLarian-CLI-Commands) for more examples and/or browse [docs.rs](https://docs.rs/maclarian) for the library API.
</details>

---
                                                                                                                    
## Technical Notes

| OS | Compatibility |
|----------|:--------------:|
| **macOS (Apple Silicon)** | Yes (Built with M2 Max) |
| **macOS (Intel)** | Currently Testing (Intel iMac) | 
| **Windows 10 (Boot Camp)** | Currently Testing (Intel iMac) | 
| **Windows 11** | Currently Testing | 
| **Linux** | Unknown | 

> [!CAUTION]
> In case it's not abundantly clear, even though Rust is largely OS-agnostic, this is a macOS-focused project. Any cross-platform support (Windows, Linux) is incidental and will not be actively maintained.

MacPak is self-contained, so there's no need to download/install/build any external dependencies. Audio playback in the Dialogue tab of MacPak *technically* requires [vgmstream-cli](https://github.com/vgmstream/vgmstream), but the macOS binary for that is already included within the MacPak app.

---

## Credits

**Core functionality is derived from (and wouldn't be possible without):**
- [LSLib](https://github.com/Norbyte/lslib)
  - PAK handling
  - LSF/LSX/LSJ handling
    - *LSLib metadata is still used in MacPak's LSF/LSX/LSJ output as a nod to the GOAT.*
  - GR2/glTF conversion
  - Virtual texture handling
  - Loca file handling
- [xiba](https://gitlab.com/saghm/xiba/)
  - PAK handling
- [Knit](https://github.com/neptuwunium/Knit)
  - GR2 decompression
  - ***Huge*** shoutout to [neptuwunium](https://github.com/arves100/opengr2/issues/8) for their clean room reverse-engineering.
- [BG3 Dialog Reader](https://github.com/angaityel/bg3-dialog-reader)
  - Character dialogue handling
- [Padme's BG3 Tutorials Templates and Modding Resources](https://www.nexusmods.com/baldursgate3/mods/132?tab=files)
  - Dye mod templates