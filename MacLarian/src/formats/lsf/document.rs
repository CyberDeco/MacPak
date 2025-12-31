//! LSF document structure definitions

use crate::error::{Error, Result};

/// LSF Metadata format (from header field)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LsfMetadataFormat {
    #[default]
    None = 0,
    KeysAndAdjacency = 1,
    None2 = 2,  // Uses lsf_adjacency instead of lsf_keys_adjacency
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
    
    pub fn get_name(&self, outer: usize, inner: usize) -> Result<&str> {
        // 65535 (0xFFFF) is a sentinel value meaning "no name" or null
        if outer == 65535 || inner == 65535 {
            return Ok("");
        }
        self.names
            .get(outer)
            .and_then(|list| list.get(inner))
            .map(|s| s.as_str())
            .ok_or_else(|| {
                let bucket_size = self.names.get(outer).map(|l| l.len()).unwrap_or(0);
                Error::InvalidStringIndex(format!(
                    "outer={}, inner={} (bucket has {} strings, total {} buckets)",
                    outer, inner, bucket_size, self.names.len()
                ))
            })
    }
    
    /// Find name indices in the names table
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