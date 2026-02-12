# Changelog

## [0.1.3] - 2026-02-11

### Bug Fixes

#### GR2 <> glTF conversion:
- Joint ordering: bones now export (to glTF) in depth-first order for Blender
- Bone coordinate conversion: bone transforms and inverse bind matrices now have X-axis reflection applied to matching the mesh vertex coordinate conversion
- BoneBindings:
  - Vertex `bone_indices` are now correctly resolved through the per-mesh BoneBindings array to skeleton-global joint indices
  - Fixed bug where bone binding reads after the first entry were misaligned
- Fixed QTangent bug where truncation-toward-zero when quantizing to i16 would cause a sign flip

## [0.1.2] - 2026-02-04

- Added support for alternate GR2 magic signature (LE64v2) for mods that were exported from LSLib with Divinity: Original Sin 2 DE settings

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
