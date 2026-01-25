//! LSJ document structures
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

#![allow(clippy::cast_possible_truncation)]

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use indexmap::IndexMap;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsjDocument {
    pub save: LsjSave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsjSave {
    pub header: LsjHeader,
    pub regions: HashMap<String, LsjNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsjHeader {
    pub version: String, // "4.0.0.100" format
}

/// LSJ Node - represents both regions and nodes
/// In JSON, this is an object with mixed attributes and child arrays
#[derive(Debug, Clone)]
pub struct LsjNode {
    pub attributes: IndexMap<String, LsjAttribute>,
    pub children: IndexMap<String, Vec<LsjNode>>,
}

impl Serialize for LsjNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        
        let mut map = serializer.serialize_map(Some(self.attributes.len() + self.children.len()))?;
        
        // Serialize attributes first
        for (key, value) in &self.attributes {
            map.serialize_entry(key, value)?;
        }
        
        // Then serialize children
        for (key, children) in &self.children {
            map.serialize_entry(key, children)?;
        }
        
        map.end()
    }
}

impl<'de> Deserialize<'de> for LsjNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Use IndexMap to preserve order when reading
        let map: IndexMap<String, Value> = IndexMap::deserialize(deserializer)?;
        let mut attributes = IndexMap::new();
        let mut children = IndexMap::new();
        
        for (key, value) in map {
            // Check if this is an attribute (object with "type" field) or children (array)
            if value.is_array() {
                // This is a child node array
                let child_array: Vec<LsjNode> = serde_json::from_value(value)
                    .map_err(serde::de::Error::custom)?;
                children.insert(key, child_array);
            } else if value.is_object() {
                // This is an attribute
                let attr: LsjAttribute = serde_json::from_value(value)
                    .map_err(serde::de::Error::custom)?;
                attributes.insert(key, attr);
            }
        }
        
        Ok(LsjNode {
            attributes,
            children,
        })
    }
}

/// LSJ Attribute - can be simple value, `TranslatedString`, or `TranslatedFSString`
#[derive(Debug, Clone)]
pub enum LsjAttribute {
    Simple {
        type_name: String,
        value: Value,
    },
    TranslatedString {
        type_name: String,
        value: Option<String>,
        handle: String,
        version: Option<u16>,
    },
    TranslatedFSString {
        type_name: String,
        value: Option<String>,
        handle: String,
        arguments: Vec<TranslatedFSStringArgument>,
    },
}

impl Serialize for LsjAttribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;
        
        match self {
            LsjAttribute::Simple { type_name, value } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", type_name)?;
                map.serialize_entry("value", value)?;
                map.end()
            }
            LsjAttribute::TranslatedString { type_name, value, handle, version } => {
                let mut size = 2; // type + handle
                if value.is_some() { size += 1; }
                if version.is_some() && *version != Some(0) { size += 1; }
                
                let mut map = serializer.serialize_map(Some(size))?;
                map.serialize_entry("type", type_name)?;
                
                if let Some(v) = value {
                    map.serialize_entry("value", v)?;
                }
                
                if let Some(ver) = version
                    && *ver != 0 {
                        map.serialize_entry("version", ver)?;
                    }
                
                map.serialize_entry("handle", handle)?;
                map.end()
            }
            LsjAttribute::TranslatedFSString { type_name, value, handle, arguments } => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("type", type_name)?;
                map.serialize_entry("value", value)?;
                map.serialize_entry("handle", handle)?;
                map.serialize_entry("arguments", arguments)?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for LsjAttribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, Value> = HashMap::deserialize(deserializer)?;
        
        let type_name = map.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| serde::de::Error::missing_field("type"))?
            .to_string();
        
        // Check if this is a TranslatedString or TranslatedFSString
        if type_name == "TranslatedString" || type_name == "28" {
            let value = map.get("value").and_then(|v| v.as_str()).map(std::string::ToString::to_string);
            let handle = map.get("handle")
                .and_then(|v| v.as_str())
                .ok_or_else(|| serde::de::Error::missing_field("handle"))?
                .to_string();
            let version = map.get("version")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as u16);
            
            Ok(LsjAttribute::TranslatedString {
                type_name,
                value,
                handle,
                version,
            })
        } else if type_name == "TranslatedFSString" || type_name == "33" {
            let value = map.get("value").and_then(|v| v.as_str()).map(std::string::ToString::to_string);
            let handle = map.get("handle")
                .and_then(|v| v.as_str())
                .ok_or_else(|| serde::de::Error::missing_field("handle"))?
                .to_string();
            let arguments = map.get("arguments")
                .map(|v| serde_json::from_value(v.clone()).unwrap_or_default())
                .unwrap_or_default();
            
            Ok(LsjAttribute::TranslatedFSString {
                type_name,
                value,
                handle,
                arguments,
            })
        } else {
            // Simple attribute
            let value = map.get("value")
                .ok_or_else(|| serde::de::Error::missing_field("value"))?
                .clone();
            
            Ok(LsjAttribute::Simple {
                type_name,
                value,
            })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedFSStringArgument {
    pub key: String,
    pub string: TranslatedFSStringValue,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslatedFSStringValue {
    pub value: Option<String>,
    pub handle: String,
    pub arguments: Vec<TranslatedFSStringArgument>,
}

impl LsjDocument {
    #[must_use] 
    pub fn new(major: u32, minor: u32, revision: u32, build: u32) -> Self {
        LsjDocument {
            save: LsjSave {
                header: LsjHeader {
                    version: format!("{major}.{minor}.{revision}.{build}"),
                },
                regions: HashMap::new(),
            },
        }
    }
    
    /// Parse version string to components
    #[must_use] 
    pub fn parse_version(&self) -> (u32, u32, u32, u32) {
        let parts: Vec<&str> = self.save.header.version.split('.').collect();
        (
            parts.first().and_then(|s| s.parse().ok()).unwrap_or(4),
            parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0),
        )
    }
}

impl LsjNode {
    #[must_use] 
    pub fn new() -> Self {
        LsjNode {
            attributes: IndexMap::new(),
            children: IndexMap::new(),
        }
    }
}

impl Default for LsjNode {
    fn default() -> Self {
        Self::new()
    }
}