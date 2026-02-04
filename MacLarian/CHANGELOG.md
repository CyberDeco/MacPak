# Changelog

## [0.1.1] - 2026-02-03

- Added support for missing GR2 metadata via glTF extensions: MeshProxy, Rigid, Cloth, Spring, Occluder, LOD, LSMVersion, Flags, and LodDistance

## [0.1.0] - 2026-02-03

Initial release.

### Added

#### PAK Archives
- Extract, create, and list .pak files
- LZ4 and Zlib compression support
- Batch operations with glob patterns
- Progress callbacks for UI integration

#### Document Formats
- LSF (binary) reading and writing
- LSX (XML) reading and writing
- LSJ (JSON) reading and writing
- Bidirectional conversion between all three formats

#### 3D Mesh Support
- GR2 (Granny2) file parsing with BitKnit decompression
- Export to glTF 2.0 and GLB formats
- Import from glTF/GLB back to GR2
- Skeleton and skinned mesh support

#### Virtual Textures
- GTS metadata parsing
- GTP tile extraction to DDS
- Virtual texture creation from DDS sources
- BC1-BC7 compression support

#### Textures
- DDS to PNG conversion
- PNG to DDS conversion (BC1, BC3, RGBA)

#### Localization
- LOCA file parsing
- LOCA to XML conversion
- Text search within LOCA files

#### Mod Utilities
- Mod structure validation
- meta.lsx generation
- info.json generation for mod managers
- Mod conflict detection
- Distribution packaging (zip, 7z)

#### CLI
- Full command-line interface with all features above
- Progress bars and colored output
- Quiet mode for scripting

### Known Limitations

- Virtual texture injection requires BG3 Script Extender (Windows-only)
- GR2 output is uncompressed (compression not yet implemented)
