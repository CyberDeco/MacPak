//! Mod metadata parsing from meta.lsx files

/// Full mod metadata extracted from meta.lsx
#[derive(Clone, Debug, Default)]
pub struct ModMetadata {
    pub name: String,
    pub folder: String,
    pub uuid: String,
    pub author: String,
    pub description: String,
    pub version64: Option<i64>,
}

impl ModMetadata {
    /// Convert Version64 to version string (e.g., "1.0.0.0")
    /// Version64 encoding: major << 55 | minor << 47 | revision << 31 | build
    pub fn version_string(&self) -> Option<String> {
        self.version64.map(|v| {
            let major = (v >> 55) & 0x7F;
            let minor = (v >> 47) & 0xFF;
            let revision = (v >> 31) & 0xFFFF;
            let build = v & 0x7FFFFFFF;
            format!("{}.{}.{}.{}", major, minor, revision, build)
        })
    }

    /// Check if this metadata is valid (has at minimum a UUID)
    pub fn is_valid(&self) -> bool {
        !self.uuid.is_empty()
    }
}

/// Extract an XML attribute value from a line
fn extract_xml_attribute(line: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr_name);
    if let Some(start) = line.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = line[value_start..].find('"') {
            return Some(line[value_start..value_start + end].to_string());
        }
    }
    None
}

/// Parse meta.lsx content to extract full mod metadata
///
/// This function parses the XML-format meta.lsx file used by BG3 mods
/// to define their metadata (name, UUID, author, etc.)
pub fn parse_meta_lsx(lsx_content: &str) -> ModMetadata {
    let mut metadata = ModMetadata::default();
    let mut in_module_info = false;

    for line in lsx_content.lines() {
        let line = line.trim();

        // Track when we're inside ModuleInfo node
        if line.contains("<node id=\"ModuleInfo\">") {
            in_module_info = true;
        }
        if in_module_info && line == "</node>" {
            in_module_info = false;
        }

        // Only parse inside ModuleInfo to avoid picking up wrong attributes
        if in_module_info {
            if line.contains("attribute id=\"Name\"") {
                if let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.name = value;
                }
            }
            if line.contains("attribute id=\"Folder\"") {
                if let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.folder = value;
                }
            }
            if line.contains("attribute id=\"UUID\"") {
                if let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.uuid = value;
                }
            }
            if line.contains("attribute id=\"Author\"") {
                if let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.author = value;
                }
            }
            if line.contains("attribute id=\"Description\"") {
                if let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.description = value;
                }
            }
            if line.contains("attribute id=\"Version64\"") {
                if let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.version64 = value.parse().ok();
                }
            }
        }
    }

    metadata
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string() {
        // Version64 value for 1.0.0.0
        let metadata = ModMetadata {
            version64: Some(36028797018963968),
            ..Default::default()
        };
        assert_eq!(metadata.version_string(), Some("1.0.0.0".to_string()));
    }

    #[test]
    fn test_parse_meta_lsx() {
        let content = r#"<?xml version="1.0" encoding="utf-8"?>
<save>
    <version major="4" minor="7" revision="1" build="3" />
    <region id="Config">
        <node id="root">
            <children>
                <node id="ModuleInfo">
                    <attribute id="Author" type="LSString" value="TestAuthor"/>
                    <attribute id="Description" type="LSString" value="Test mod description"/>
                    <attribute id="Folder" type="LSString" value="TestMod"/>
                    <attribute id="Name" type="LSString" value="Test Mod Name"/>
                    <attribute id="UUID" type="FixedString" value="12345678-1234-1234-1234-123456789abc"/>
                    <attribute id="Version64" type="int64" value="36028797018963968"/>
                </node>
            </children>
        </node>
    </region>
</save>"#;

        let metadata = parse_meta_lsx(content);
        assert_eq!(metadata.name, "Test Mod Name");
        assert_eq!(metadata.folder, "TestMod");
        assert_eq!(metadata.uuid, "12345678-1234-1234-1234-123456789abc");
        assert_eq!(metadata.author, "TestAuthor");
        assert_eq!(metadata.description, "Test mod description");
        assert!(metadata.is_valid());
    }
}
