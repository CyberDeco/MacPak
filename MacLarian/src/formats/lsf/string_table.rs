//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT), 2023 saghm (xiba, Apache-2.0)
//!
//! SPDX-License-Identifier: MIT AND Apache-2.0
//!
//! String table management for LSF files

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StringTable {
    strings: Vec<String>,
    indices: HashMap<String, i32>,
}

impl StringTable {
    #[must_use] 
    pub fn new() -> Self {
        StringTable {
            strings: Vec::new(),
            indices: HashMap::new(),
        }
    }
    
    /// Add a string to the table, returns its index
    pub fn add(&mut self, s: &str) -> i32 {
        if let Some(&idx) = self.indices.get(s) {
            return idx;
        }
        
        let idx = self.strings.len() as i32;
        self.strings.push(s.to_string());
        self.indices.insert(s.to_string(), idx);
        idx
    }
    
    /// Get string by index
    #[must_use] 
    pub fn get(&self, idx: i32) -> Option<&str> {
        self.strings.get(idx as usize).map(std::string::String::as_str)
    }
    
    /// Get index of a string
    #[must_use] 
    pub fn index_of(&self, s: &str) -> Option<i32> {
        self.indices.get(s).copied()
    }
    
    /// Convert to bytes for serialization
    #[must_use] 
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for s in &self.strings {
            bytes.extend_from_slice(s.as_bytes());
            bytes.push(0); // Null terminator
        }
        bytes
    }
    
    /// Number of strings in table
    #[must_use] 
    pub fn len(&self) -> usize {
        self.strings.len()
    }
    
    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
}