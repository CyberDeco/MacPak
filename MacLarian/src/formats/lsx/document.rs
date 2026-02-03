//! LSX document structures
//!
//!

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxDocument {
    pub major: u32,
    pub minor: u32,
    pub revision: u32,
    pub build: u32,
    pub regions: Vec<LsxRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxRegion {
    pub id: String,
    pub nodes: Vec<LsxNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxNode {
    pub id: String,
    pub key: Option<String>,
    pub attributes: Vec<LsxAttribute>,
    pub children: Vec<LsxNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxAttribute {
    pub id: String,
    pub type_name: String,
    pub value: String,
    pub handle: Option<String>,
    pub version: Option<u16>,
}

impl LsxDocument {
    #[must_use]
    pub fn new(major: u32, minor: u32, revision: u32, build: u32) -> Self {
        LsxDocument {
            major,
            minor,
            revision,
            build,
            regions: Vec::new(),
        }
    }

    /// Get version as a string (for LSJ conversion)
    #[must_use]
    pub fn version_string(&self) -> String {
        format!(
            "{}.{}.{}.{}",
            self.major, self.minor, self.revision, self.build
        )
    }
}

impl LsxNode {
    #[must_use]
    pub fn new(id: String) -> Self {
        LsxNode {
            id,
            key: None,
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }
}

impl LsxAttribute {
    #[must_use]
    pub fn new(id: String, type_name: String, value: String) -> Self {
        LsxAttribute {
            id,
            type_name,
            value,
            handle: None,
            version: None,
        }
    }
}
