//! Type definitions for GR2 struct serialization

use std::collections::HashMap;

use super::constants::MEMBER_NONE;
use super::section::Section;

/// A member definition for struct serialization.
pub struct MemberDef {
    pub name: &'static str,
    pub member_type: u32,
    pub array_size: u32,
    /// Optional pointer to nested type definition (section 4 offset).
    pub def_type_offset: Option<u32>,
}

impl MemberDef {
    pub fn new(name: &'static str, member_type: u32) -> Self {
        Self {
            name,
            member_type,
            array_size: 0,
            def_type_offset: None,
        }
    }

    pub fn array(name: &'static str, member_type: u32, size: u32) -> Self {
        Self {
            name,
            member_type,
            array_size: size,
            def_type_offset: None,
        }
    }

    pub fn with_def(name: &'static str, member_type: u32, def_offset: u32) -> Self {
        Self {
            name,
            member_type,
            array_size: 0,
            def_type_offset: Some(def_offset),
        }
    }
}

/// Write a type definition to a section.
pub fn write_type_def(
    section: &mut Section,
    members: &[MemberDef],
    string_offsets: &HashMap<String, u32>,
) {
    // Member size in 64-bit mode: 44 bytes
    // [type:4][name_ptr:8][def_ptr:8][array_size:4][extra:12][unknown:8]
    for member in members {
        section.write_u32(member.member_type);
        if let Some(&offset) = string_offsets.get(member.name) {
            section.write_ptr(0, offset); // Name string in section 0
        } else {
            section.write_null_ptr();
        }
        if let Some(def_offset) = member.def_type_offset {
            section.write_ptr(4, def_offset); // Definition pointer to type in section 4
        } else {
            section.write_null_ptr(); // No nested type definition
        }
        section.write_u32(member.array_size);
        section.write_u32(0); // Extra[0]
        section.write_u32(0); // Extra[1]
        section.write_u32(0); // Extra[2]
        section.write_u64(0); // Unknown
    }
    // End marker
    section.write_u32(MEMBER_NONE);
    section.write_null_ptr();
    section.write_null_ptr();
    section.write_u32(0);
    section.write_u32(0);
    section.write_u32(0);
    section.write_u32(0);
    section.write_u64(0);
}
