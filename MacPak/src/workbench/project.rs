//! Project manifest types for macpak.toml

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_compression() -> String {
    "lz4".to_string()
}

fn default_true() -> bool {
    true
}

fn default_output_dir() -> String {
    "build".to_string()
}

/// The full project manifest (macpak.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub project: ProjectMeta,
    #[serde(default)]
    pub build: BuildSettings,
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

/// Mod project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMeta {
    pub name: String,
    pub folder: String,
    pub author: String,
    pub description: String,
    pub uuid: String,
    pub version: String,
    pub recipe: String,
}

/// Build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSettings {
    #[serde(default = "default_compression")]
    pub compression: String,
    #[serde(default)]
    pub priority: u8,
    #[serde(default = "default_true")]
    pub generate_info_json: bool,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

impl Default for BuildSettings {
    fn default() -> Self {
        Self {
            compression: default_compression(),
            priority: 0,
            generate_info_json: true,
            output_dir: default_output_dir(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_roundtrip() {
        let manifest = ProjectManifest {
            project: ProjectMeta {
                name: "My Cool Armor".to_string(),
                folder: "MyCoolArmor".to_string(),
                author: "ModderName".to_string(),
                description: "A set of custom armor".to_string(),
                uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
                version: "1.0.0.0".to_string(),
                recipe: "equipment".to_string(),
            },
            build: BuildSettings::default(),
            variables: HashMap::from([("item_type".to_string(), "Armor".to_string())]),
        };

        let toml_str = toml::to_string_pretty(&manifest).unwrap();
        let parsed: ProjectManifest = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.project.name, "My Cool Armor");
        assert_eq!(parsed.project.folder, "MyCoolArmor");
        assert_eq!(parsed.build.compression, "lz4");
        assert_eq!(parsed.variables.get("item_type").unwrap(), "Armor");
    }
}
