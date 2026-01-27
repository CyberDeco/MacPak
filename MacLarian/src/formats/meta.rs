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
    #[must_use] 
    pub fn version_string(&self) -> Option<String> {
        self.version64.map(|v| {
            let major = (v >> 55) & 0x7F;
            let minor = (v >> 47) & 0xFF;
            let revision = (v >> 31) & 0xFFFF;
            let build = v & 0x7FFFFFFF;
            format!("{major}.{minor}.{revision}.{build}")
        })
    }

    /// Check if this metadata is valid (has at minimum a UUID)
    #[must_use] 
    pub fn is_valid(&self) -> bool {
        !self.uuid.is_empty()
    }
}

/// Extract an XML attribute value from a line
fn extract_xml_attribute(line: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{attr_name}=\"");
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
#[must_use] 
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
            if line.contains("attribute id=\"Name\"")
                && let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.name = value;
                }
            if line.contains("attribute id=\"Folder\"")
                && let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.folder = value;
                }
            if line.contains("attribute id=\"UUID\"")
                && let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.uuid = value;
                }
            if line.contains("attribute id=\"Author\"")
                && let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.author = value;
                }
            if line.contains("attribute id=\"Description\"")
                && let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.description = value;
                }
            if line.contains("attribute id=\"Version64\"")
                && let Some(value) = extract_xml_attribute(line, "value") {
                    metadata.version64 = value.parse().ok();
                }
        }
    }

    metadata
}

