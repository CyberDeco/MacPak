# Changelog

## [Unreleased]

### Added

- [x] Working: pak operations (list contents, unpack single/batch, verify structure, and pack single/batch)
- [x] Working: .lsf <> .lsx <> .lsj conversion
- [x] Working: inline text file editor with multiple tabbed support and meta.lsx generator
- [x] Working: various QOL things - UUID/TranslatedString generator, color picker/saver, etc.
- [x] Temporarily working: DDS file previews (it's a little jank because it's a stopgap fix)
- [x] Working: convert .gr2 files <> .glb or .glTF (same as .glb but .bin is its own file)
- [x] Working: convert .dds (BC1/BC2/BC3) <> .png
- [x] Working: decompress and stitch virtual textures
  - Attempted: reverse-engineering compression for .gr2 files, but got to be too annoying for just a QOL implementation
- [x] Working: local Floem GUI (not yet packaged for production)
  - [x] File tree/preview/browser tab
  - [x] Text editor tab + meta.lsx generator
    - [ ] BUG: Content doesn't populate after large file warning
    - [ ] BUG: I think some LSF files from BaldursBasket aren't populating because the conversion is failing?
  - [x] Pak operations tab
    - [ ] BUG: some files show as failed in the list (should never show as failed if just listing)
    - [ ] BUG: investigate why some files are failing to unpak
  - [x] Tab for gr2 <> glTF/GLB single/batch conversion
  - [x] Tab for extracting and stitching single/batch virtual textures
    - [ ] Future: work in reverse for custom virtual textures (needs BG3SE for testing)
  - [x] Tab for creating custom dyes
  - [x] Tab incorporating [BG3 Dialog Reader](https://github.com/angaityel/bg3-dialog-reader)
    - [x] Loads all dialogues from Gustav.pak and Shared.pak - very quickly
    - [x] Can open/expand nodes no problem
    - [ ] BUG: need to fix width for left panel
    - [ ] BUG: not all (*snaps* yes)
    - [ ] FEATURE: implement per-act subfolders (might be more of a hassle than it's worth)
    - [x] FEATURE: play audio (right-click)
      - [x] WORKING: using vgmstream (because damn was reverse-engineering not worth it) - uses brew installed vgmstream as fallback but will probably bundle [vgmstream-cli](https://vgmstream.org) within app when ready for launch
- [x] Working: Piece together x-ref library for GR2 files to their base/normal/physical virtual textures and/or cloth msk
    - [ ] Get tooltip/menu little icons
    - [ ] Option to automatically retrieve companion texture files when working with GR2
    - [ ] Combine GR2 and VTEX tabs into one
- [ ] Currently: incorporate bevy for 3D model and texture rendering (just previews)
  - [x] .glb/.gltf
  - [x] .gr2 (via temp .glb conversion)
  - [ ] .dds - can already load 512 x 512 previews in the GUI
  - [ ] marry texture(s) to GR2/glTF (paths database built)
- [ ] Currently: dye lab
    - [x] Mimic Official Toolkit's dye menu... thing... and improve it + use built-in macOS Color Picker
    - [ ] Preview on a GR2 + mesh ?
    - [x] Auto-populate mod structure based on Padme4000's template
     
- [ ] GENERAL: Parallelization with Rayon
    - [x] Dialogue backend (Voice Meta, Flag Cache, Localization, 

 New Module

  - MacLarian/src/formats/voice_meta.rs - Moved voice meta loading from MacPak to MacLarian with parallel soundbank parsing

  Parallelized Locations

  | Target         | File                                                | Pattern                                           |
  |----------------|-----------------------------------------------------|---------------------------------------------------|
  | Voice Meta     | MacLarian/formats/voice_meta.rs                     | par_iter on soundbank files, collect → merge      |
  | GR2 Batch      | MacPak/gui/tabs/gr2/conversion.rs                   | par_iter().map().collect() + atomics              |
  | GTS Extraction | MacPak/gui/tabs/virtual_textures/extraction.rs      | par_iter on GTS files (outer loop only)           |
  | Flag Cache     | MacLarian/formats/dialog/flags.rs                   | par_iter on flag parsing, collect → merge         |
  | Localization   | MacPak/gui/tabs/dialogue/operations/localization.rs | Batch read + par_iter on loca parsing             |
  | PAK Extract    | MacPak/gui/tabs/pak_ops/operations.rs:856           | par_iter + AtomicUsize counters                   |
  | PAK Create     | MacPak/gui/tabs/pak_ops/operations.rs:999           | par_iter + AtomicUsize counters                   |
  | Audio Index    | MacLarian/formats/wem/cache.rs                      | walkdir + par_iter for dir scan; par_iter for PAK |

- [ ] GENERAL: Replacing writing temp lsx files with new extract_as_bytes (for both .pak and .lsf)

### Known Bugs

## GUI

- Color selection in the Dye Lab tab only works for macOS since it relies on the Color Picker app.

### Known Patches

- Uses muda to control the menu bar (in a really hacky way) until Floem fully incorporates it.
     
### To Do

- [ ] preview with cloth on
- [ ] debug why basket gr2 can't be converted to glb

### Added
- New features go here as you work

### Changed
- Modifications to existing features

### Fixed
- Bug fixes

## [0.1.0-alpha.1] - 2025-01-15
### Added
- New features go here as you work

### Changed
- Modifications to existing features

### Fixed
- Bug fixes