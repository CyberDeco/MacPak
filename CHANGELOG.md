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
  - [x] Pak operations tab
  - [x] Tab for gr2 <> glTF/GLB single/batch conversion
  - [x] Tab for extracting and stitching single/batch virtual textures
    - [ ] Future: work in reverse for custom virtual textures (needs BG3SE for testing)
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