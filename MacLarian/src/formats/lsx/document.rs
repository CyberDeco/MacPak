//! LSX document structures
//!
//!

use serde::{Deserialize, Serialize};

/// An LSX (Larian Save XML) document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxDocument {
    /// Major version number.
    pub major: u32,
    /// Minor version number.
    pub minor: u32,
    /// Revision number.
    pub revision: u32,
    /// Build number.
    pub build: u32,
    /// Document regions containing the data.
    pub regions: Vec<LsxRegion>,
}

/// A region in an LSX document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxRegion {
    /// Region identifier.
    pub id: String,
    /// Root nodes in this region.
    pub nodes: Vec<LsxNode>,
}

/// A node in an LSX document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxNode {
    /// Node identifier/type.
    pub id: String,
    /// Optional key for this node.
    pub key: Option<String>,
    /// Attributes on this node.
    pub attributes: Vec<LsxAttribute>,
    /// Child nodes.
    pub children: Vec<LsxNode>,
}

/// An attribute on an LSX node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LsxAttribute {
    /// Attribute identifier/name.
    pub id: String,
    /// Type name (e.g., "`FixedString`", "`int32`").
    pub type_name: String,
    /// String representation of the value.
    pub value: String,
    /// Localization handle for translated strings.
    pub handle: Option<String>,
    /// Version number for translated strings.
    pub version: Option<u16>,
}

impl LsxDocument {
    /// Creates a new LSX document with the specified version.
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
    /// Creates a new LSX node with the given ID.
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
    /// Creates a new LSX attribute with the given ID, type, and value.
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
