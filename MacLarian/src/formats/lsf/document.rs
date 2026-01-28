//! LSF document structure definitions
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

use crate::error::{Error, Result};

/// LSF Metadata format (from header field)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LsfMetadataFormat {
    #[default]
    None = 0,
    KeysAndAdjacency = 1,
    None2 = 2, // Uses lsf_adjacency instead of lsf_keys_adjacency
}

impl From<u32> for LsfMetadataFormat {
    fn from(value: u32) -> Self {
        match value {
            1 => LsfMetadataFormat::KeysAndAdjacency,
            2 => LsfMetadataFormat::None2,
            _ => LsfMetadataFormat::None,
        }
    }
}

#[derive(Debug)]
pub struct LsfDocument {
    pub engine_version: u64,
    pub names: Vec<Vec<String>>,
    pub nodes: Vec<LsfNode>,
    pub attributes: Vec<LsfAttribute>,
    pub values: Vec<u8>,
    pub node_keys: Vec<Option<String>>,
    pub has_keys_section: bool,
    pub metadata_format: LsfMetadataFormat,
}

#[derive(Debug, Clone)]
pub struct LsfNode {
    pub name_index_outer: usize,
    pub name_index_inner: usize,
    pub parent_index: i32,
    pub first_attribute_index: i32,
}

#[derive(Debug, Clone)]
pub struct LsfAttribute {
    pub name_index_outer: usize,
    pub name_index_inner: usize,
    pub type_info: u32,
    pub next_index: i32,
    pub offset: usize,
}

impl LsfDocument {
    #[must_use]
    pub fn new() -> Self {
        LsfDocument {
            engine_version: 0,
            names: Vec::new(),
            nodes: Vec::new(),
            attributes: Vec::new(),
            values: Vec::new(),
            node_keys: Vec::new(),
            has_keys_section: false,
            metadata_format: LsfMetadataFormat::None,
        }
    }

    /// Get a name from the names table by indices.
    ///
    /// # Errors
    /// Returns an error if the indices are out of bounds.
    pub fn get_name(&self, outer: usize, inner: usize) -> Result<&str> {
        // 65535 (0xFFFF) is a sentinel value meaning "no name" or null
        if outer == 65535 || inner == 65535 {
            return Ok("");
        }
        self.names
            .get(outer)
            .and_then(|list| list.get(inner))
            .map(std::string::String::as_str)
            .ok_or_else(|| {
                let bucket_size = self.names.get(outer).map_or(0, std::vec::Vec::len);
                Error::InvalidStringIndex(format!(
                    "outer={outer}, inner={inner} (bucket has {bucket_size} strings, total {} buckets)",
                    self.names.len()
                ))
            })
    }

    /// Find name indices in the names table
    #[must_use]
    pub fn find_name_indices(&self, name: &str) -> Option<(usize, usize)> {
        for (outer, name_list) in self.names.iter().enumerate() {
            for (inner, list_name) in name_list.iter().enumerate() {
                if list_name == name {
                    return Some((outer, inner));
                }
            }
        }
        None
    }
}

impl Default for LsfDocument {
    fn default() -> Self {
        Self::new()
    }
}

// Query helpers for direct LSF access (avoids XML conversion overhead)
impl LsfDocument {
    /// Get the name of a node
    #[must_use]
    pub fn node_name(&self, node_idx: usize) -> Option<&str> {
        let node = self.nodes.get(node_idx)?;
        self.get_name(node.name_index_outer, node.name_index_inner)
            .ok()
    }

    /// Get indices of all children of a node
    #[must_use]
    pub fn children_of(&self, parent_idx: usize) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.parent_index == parent_idx as i32)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Get indices of root nodes (`parent_index` == -1)
    #[must_use]
    pub fn root_nodes(&self) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.parent_index == -1)
            .map(|(idx, _)| idx)
            .collect()
    }

    /// Iterate over attributes of a node, yielding (name, `type_id`, `value_offset`, `value_length`)
    #[must_use]
    pub fn attributes_of(&self, node_idx: usize) -> Vec<(usize, &str, u32, usize, usize)> {
        let Some(node) = self.nodes.get(node_idx) else {
            return Vec::new();
        };

        if node.first_attribute_index < 0 {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut attr_idx = node.first_attribute_index as usize;

        loop {
            let Some(attr) = self.attributes.get(attr_idx) else {
                break;
            };

            let name = self
                .get_name(attr.name_index_outer, attr.name_index_inner)
                .unwrap_or("");
            let type_id = attr.type_info & 0x3F;
            let value_length = (attr.type_info >> 6) as usize;

            result.push((attr_idx, name, type_id, attr.offset, value_length));

            if attr.next_index < 0 {
                break;
            }
            attr_idx = attr.next_index as usize;
        }

        result
    }

    /// Get a `FixedString` attribute value directly (`type_id` 22)
    #[must_use]
    pub fn get_fixed_string_attr(&self, node_idx: usize, attr_name: &str) -> Option<String> {
        for (_, name, type_id, offset, length) in self.attributes_of(node_idx) {
            if name == attr_name && type_id == 22 {
                // FixedString: null-terminated string (NOT length-prefixed)
                if offset + length > self.values.len() {
                    return None;
                }
                let bytes = &self.values[offset..offset + length];
                // Find null terminator
                let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
                return String::from_utf8(bytes[..end].to_vec()).ok();
            }
        }
        None
    }

    /// Get a float attribute value directly (`type_id` 6)
    #[must_use]
    pub fn get_float_attr(&self, node_idx: usize, attr_name: &str) -> Option<f32> {
        for (_, name, type_id, offset, _length) in self.attributes_of(node_idx) {
            if name == attr_name && type_id == 6 {
                if offset + 4 > self.values.len() {
                    return None;
                }
                let bytes = [
                    self.values[offset],
                    self.values[offset + 1],
                    self.values[offset + 2],
                    self.values[offset + 3],
                ];
                return Some(f32::from_le_bytes(bytes));
            }
        }
        None
    }

    /// Find child nodes with a specific name
    #[must_use]
    pub fn find_children_by_name(&self, parent_idx: usize, name: &str) -> Vec<usize> {
        self.children_of(parent_idx)
            .into_iter()
            .filter(|&idx| self.node_name(idx) == Some(name))
            .collect()
    }
}
