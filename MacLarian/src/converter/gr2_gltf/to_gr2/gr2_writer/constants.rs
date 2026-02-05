//! GR2 format constants

/// Game tag for BG3/DOS2
pub const TAG_BG3: u32 = 0xE57F0039;

/// GR2 format version
pub const VERSION: u32 = 7;

/// Section count (7 sections for BG3 GR2 files)
/// Section 0: Main (root object, strings, misc data)
/// Section 1: `TrackGroups` (animations) - empty for static meshes
/// Section 2: Skeleton data
/// Section 3: Mesh structs
/// Section 4: Type definitions
/// Section 5: Vertex data
/// Section 6: Index data
pub const NUM_SECTIONS: u32 = 7;

// Member types
pub const MEMBER_NONE: u32 = 0;
pub const MEMBER_REFERENCE: u32 = 2;
pub const MEMBER_REF_TO_ARRAY: u32 = 3;
pub const MEMBER_ARRAY_OF_REFS: u32 = 4;
pub const MEMBER_STRING: u32 = 8;
pub const MEMBER_TRANSFORM: u32 = 9;
pub const MEMBER_REAL32: u32 = 10;
pub const MEMBER_UINT8: u32 = 12;
pub const MEMBER_BINORMAL_INT16: u32 = 17;
pub const MEMBER_INT32: u32 = 19;
pub const MEMBER_UINT32: u32 = 20;
pub const MEMBER_REAL16: u32 = 21;
