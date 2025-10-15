//! LSF document structure definitions

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct LsfDocument {
    pub engine_version: u64,
    pub names: Vec<Vec<String>>,
    pub nodes: Vec<LsfNode>,
    pub attributes: Vec<LsfAttribute>,
    pub values: Vec<u8>,
    pub node_keys: Vec<Option<String>>,
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
        }
    }
    
    pub fn get_name(&self, outer: usize, inner: usize) -> Result<&str> {
        self.names
            .get(outer)
            .and_then(|list| list.get(inner))
            .map(|s| s.as_str())
            .ok_or_else(|| Error::InvalidStringIndex(inner as i32))
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