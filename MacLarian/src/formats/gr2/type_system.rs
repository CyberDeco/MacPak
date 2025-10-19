//! Granny2 Type System
//!
//! Granny uses a reflection-based type system where types are defined at runtime.
//! This allows different file versions to have different structure layouts while
//! remaining compatible.
//!
//! ## Type System Overview
//!
//! Every GR2 file contains a type definition table that describes all structures
//! used in the file. Each type has:
//! - A type tag (identifier)
//! - Field definitions (name, type, offset)
//! - Size information
//!
//! ## Member Types
//!
//! Granny supports several member types:
//! - Inline: Embedded struct data
//! - Reference: Pointer to single item (section + offset)
//! - ReferenceToArray: Pointer to array (count + data)
//! - ArrayOfReferences: Array of pointers
//! - String: Null-terminated C string
//! - Primitives: uint32, float, etc.

use crate::error::{Error, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

/// Granny member type enumeration
///
/// Based on LSLib's MemberType enum. These describe how field data
/// is stored and referenced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemberType {
    /// Embedded struct data (no pointer indirection)
    Inline = 0,

    /// Pointer to a single struct instance
    Reference = 1,

    /// Pointer to an array of structs (count + data pointer)
    ReferenceToArray = 2,

    /// Array of pointers to structs
    ArrayOfReferences = 3,

    /// Polymorphic reference (type ID + pointer)
    VariantReference = 4,

    /// Null-terminated C string reference
    String = 7,

    /// 3D transformation matrix (special type)
    Transform = 8,

    // Primitive types (not explicitly in LSLib enum, but used)
    /// 32-bit unsigned integer
    UInt32 = 10,

    /// 32-bit float
    Float32 = 11,

    /// 8-bit unsigned integer
    UInt8 = 12,

    /// 16-bit unsigned integer
    UInt16 = 13,
}

impl MemberType {
    /// Parse member type from u32 value
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Inline),
            1 => Some(Self::Reference),
            2 => Some(Self::ReferenceToArray),
            3 => Some(Self::ArrayOfReferences),
            4 => Some(Self::VariantReference),
            7 => Some(Self::String),
            8 => Some(Self::Transform),
            10 => Some(Self::UInt32),
            11 => Some(Self::Float32),
            12 => Some(Self::UInt8),
            13 => Some(Self::UInt16),
            _ => None,
        }
    }

    /// Check if this type is a reference (requires pointer resolution)
    pub fn is_reference(&self) -> bool {
        matches!(
            self,
            Self::Reference
                | Self::ReferenceToArray
                | Self::ArrayOfReferences
                | Self::VariantReference
                | Self::String
        )
    }

    /// Check if this type is inline (no pointer indirection)
    pub fn is_inline(&self) -> bool {
        matches!(self, Self::Inline)
    }
}

/// Type definition for a Granny struct
///
/// Describes the layout of a structure, including all its fields.
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    /// Type tag (identifier)
    pub type_tag: u32,

    /// Field definitions
    pub fields: Vec<FieldDefinition>,

    /// Total size of this type in bytes (if known)
    pub size: Option<u32>,

    /// Type name (if available)
    pub name: Option<String>,
}

impl TypeDefinition {
    /// Create a new type definition
    pub fn new(type_tag: u32) -> Self {
        Self {
            type_tag,
            fields: Vec::new(),
            size: None,
            name: None,
        }
    }

    /// Add a field to this type
    pub fn add_field(&mut self, field: FieldDefinition) {
        self.fields.push(field);
    }

    /// Get a field by name
    pub fn get_field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields.iter().find(|f| f.name == name)
    }

    /// Get field count
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}

/// Field definition within a type
///
/// Describes a single field: its name, type, and location.
#[derive(Debug, Clone)]
pub struct FieldDefinition {
    /// Field name (e.g., "Meshes", "Name", "VertexCount")
    pub name: String,

    /// Member type (how the data is stored)
    pub member_type: MemberType,

    /// Byte offset within parent struct
    pub offset: u32,

    /// Type tag of the field type (for complex types)
    pub type_tag: Option<u32>,

    /// Array size (if fixed-size array)
    pub array_size: Option<u32>,
}

impl FieldDefinition {
    /// Create a new field definition
    pub fn new(name: String, member_type: MemberType, offset: u32) -> Self {
        Self {
            name,
            member_type,
            offset,
            type_tag: None,
            array_size: None,
        }
    }

    /// Set the type tag for this field
    pub fn with_type_tag(mut self, type_tag: u32) -> Self {
        self.type_tag = Some(type_tag);
        self
    }

    /// Set the array size for this field
    pub fn with_array_size(mut self, size: u32) -> Self {
        self.array_size = Some(size);
        self
    }
}

/// Type cache for quick lookups
///
/// Maps type tags to their definitions for fast access during deserialization.
#[derive(Debug, Clone)]
pub struct TypeCache {
    types: HashMap<u32, TypeDefinition>,
}

impl TypeCache {
    /// Create an empty type cache
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Add a type definition to the cache
    pub fn insert(&mut self, type_def: TypeDefinition) {
        self.types.insert(type_def.type_tag, type_def);
    }

    /// Get a type definition by tag
    pub fn get(&self, type_tag: u32) -> Option<&TypeDefinition> {
        self.types.get(&type_tag)
    }

    /// Check if a type exists
    pub fn contains(&self, type_tag: u32) -> bool {
        self.types.contains_key(&type_tag)
    }

    /// Get the number of types in the cache
    pub fn len(&self) -> usize {
        self.types.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    /// Iterate over all types
    pub fn types(&self) -> impl Iterator<Item = &TypeDefinition> {
        self.types.values()
    }
}

impl Default for TypeCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse type definitions from section data
///
/// The type table is typically at the beginning of the root section.
/// It contains type definitions for all structures used in the file.
///
/// # Format
///
/// The exact binary format varies, but generally:
/// - Type count (u32)
/// - Array of type descriptors
/// - Each descriptor contains field count and field definitions
///
/// # Arguments
///
/// * `data` - Section data containing type definitions
/// * `offset` - Starting offset in the section (usually 0)
///
/// # Returns
///
/// A TypeCache containing all parsed type definitions
pub fn parse_type_definitions(data: &[u8], offset: usize) -> Result<TypeCache> {
    // TODO: Implement full type parsing
    // For now, we'll create a minimal implementation that we can expand

    tracing::debug!("Parsing type definitions from offset 0x{:x}", offset);
    tracing::debug!("Data size: {} bytes", data.len());

    // Create empty cache
    let mut cache = TypeCache::new();

    // Try to parse basic structure from the data
    if data.len() < offset + 8 {
        return Err(Error::Gr2ParseError(
            "Section too small for type definitions".to_string(),
        ));
    }

    let mut cursor = Cursor::new(&data[offset..]);

    // Read first few values to understand the structure
    let first_u32 = cursor.read_u32::<LittleEndian>()?;
    let second_u32 = cursor.read_u32::<LittleEndian>()?;

    tracing::debug!("First u32: 0x{:08x} ({})", first_u32, first_u32);
    tracing::debug!("Second u32: 0x{:08x} ({})", second_u32, second_u32);

    // In BG3 files, we've observed:
    // Offset 0x00: 0x0000f4f0 (62704) - possibly an offset or type table pointer
    // Offset 0x04: 0x00000000 - null/reserved
    // Offset 0x08: 0x00000000 - null/reserved
    // Offset 0x0C: 0x0000039c (924) - possibly a size or count

    // For now, create a placeholder type definition for the root type (tag 4)
    // This will allow us to continue development while we work on full type parsing
    let root_type = create_placeholder_root_type();
    cache.insert(root_type);

    tracing::info!("Type cache initialized with {} types", cache.len());

    Ok(cache)
}

/// Create a placeholder type definition for GrannyFileInfo (root type)
///
/// This is a temporary implementation based on known structure.
/// Will be replaced with proper type parsing.
fn create_placeholder_root_type() -> TypeDefinition {
    let mut root_type = TypeDefinition::new(4); // Type tag 4 = GrannyFileInfo
    root_type.name = Some("GrannyFileInfo".to_string());

    // Add known fields from LSLib research
    // These are placeholders - actual offsets will be parsed from type data

    // ArtToolInfo* (reference)
    root_type.add_field(
        FieldDefinition::new("ArtToolInfo".to_string(), MemberType::Reference, 0x00)
            .with_type_tag(100), // Placeholder type tag
    );

    // ExporterInfo* (reference)
    root_type.add_field(
        FieldDefinition::new("ExporterInfo".to_string(), MemberType::Reference, 0x04)
            .with_type_tag(101), // Placeholder type tag
    );

    // Meshes[] (reference to array)
    root_type.add_field(
        FieldDefinition::new("Meshes".to_string(), MemberType::ReferenceToArray, 0x08)
            .with_type_tag(200), // Placeholder type tag for Mesh
    );

    // Skeletons[] (reference to array)
    root_type.add_field(
        FieldDefinition::new("Skeletons".to_string(), MemberType::ReferenceToArray, 0x10)
            .with_type_tag(201), // Placeholder type tag for Skeleton
    );

    root_type
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_member_type_parsing() {
        assert_eq!(MemberType::from_u32(0), Some(MemberType::Inline));
        assert_eq!(MemberType::from_u32(1), Some(MemberType::Reference));
        assert_eq!(MemberType::from_u32(2), Some(MemberType::ReferenceToArray));
        assert_eq!(MemberType::from_u32(7), Some(MemberType::String));
        assert_eq!(MemberType::from_u32(999), None);
    }

    #[test]
    fn test_member_type_is_reference() {
        assert!(MemberType::Reference.is_reference());
        assert!(MemberType::ReferenceToArray.is_reference());
        assert!(MemberType::String.is_reference());
        assert!(!MemberType::Inline.is_reference());
        assert!(!MemberType::UInt32.is_reference());
    }

    #[test]
    fn test_type_definition() {
        let mut type_def = TypeDefinition::new(42);
        assert_eq!(type_def.type_tag, 42);
        assert_eq!(type_def.field_count(), 0);

        type_def.add_field(FieldDefinition::new(
            "TestField".to_string(),
            MemberType::UInt32,
            0x10,
        ));

        assert_eq!(type_def.field_count(), 1);
        assert!(type_def.get_field("TestField").is_some());
        assert!(type_def.get_field("NonExistent").is_none());
    }

    #[test]
    fn test_type_cache() {
        let mut cache = TypeCache::new();
        assert!(cache.is_empty());

        let type_def = TypeDefinition::new(1);
        cache.insert(type_def);

        assert_eq!(cache.len(), 1);
        assert!(cache.contains(1));
        assert!(!cache.contains(2));
        assert!(cache.get(1).is_some());
    }

    #[test]
    fn test_placeholder_root_type() {
        let root = create_placeholder_root_type();
        assert_eq!(root.type_tag, 4);
        assert_eq!(root.name, Some("GrannyFileInfo".to_string()));
        assert!(root.get_field("Meshes").is_some());
        assert!(root.get_field("Skeletons").is_some());
    }
}
