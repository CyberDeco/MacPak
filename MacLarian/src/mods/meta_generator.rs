//! Meta.lsx Generator
//!
//! Generate mod metadata files for BG3 mods.

/// Convert version components to BG3's int64 format
///
/// BG3 version format: major << 55 | minor << 47 | patch << 31 | build
#[must_use]
pub fn version_to_int64(major: u32, minor: u32, patch: u32, build: u32) -> i64 {
    ((major as i64) << 55) | ((minor as i64) << 47) | ((patch as i64) << 31) | (build as i64)
}

/// Parse a version string into components
///
/// Accepts:
/// - Full version: "1.0.0.0"
/// - Partial versions: "1", "1.0", "1.0.0" (missing parts default to 0)
/// - Raw Version64 integer: "36028797018963968"
///
/// Returns (major, minor, patch, build) or None if invalid
#[must_use]
pub fn parse_version_string(version: &str) -> Option<(u32, u32, u32, u32)> {
    let trimmed = version.trim();

    // Check if it's a raw Version64 integer (no dots, parses as i64)
    // Version64 for major=1 is 2^55, so any value >= 2^31 is likely a Version64
    // (values below that are treated as simple major version numbers)
    const VERSION64_THRESHOLD: i64 = 1 << 31;
    if !trimmed.contains('.') {
        if let Ok(v64) = trimmed.parse::<i64>() {
            if v64 >= VERSION64_THRESHOLD {
                // Decode Version64: major << 55 | minor << 47 | patch << 31 | build
                let major = ((v64 >> 55) & 0x7F) as u32;
                let minor = ((v64 >> 47) & 0xFF) as u32;
                let patch = ((v64 >> 31) & 0xFFFF) as u32;
                let build = (v64 & 0x7FFFFFFF) as u32;
                return Some((major, minor, patch, build));
            }
            // Small integer - treat as major version (e.g., "1" -> 1.0.0.0)
            return Some((v64 as u32, 0, 0, 0));
        }
    }

    // Parse as dotted version string, padding missing parts with 0
    let parts: Vec<&str> = trimmed.split('.').collect();
    if parts.is_empty() || parts.len() > 4 {
        return None;
    }

    let major = parts.first().and_then(|s| s.parse().ok())?;
    let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let build = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    Some((major, minor, patch, build))
}

/// Convert a string to a safe folder name
///
/// - Spaces become underscores
/// - Special characters (apostrophes, hyphens, parentheses, etc.) are stripped
/// - Multiple underscores are collapsed
#[must_use]
pub fn to_folder_name(s: &str) -> String {
    let result: String = s
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() {
                Some(c)
            } else if c == ' ' {
                Some('_')
            } else {
                None // Strip special characters entirely
            }
        })
        .collect();

    // Collapse multiple underscores and trim leading/trailing
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_underscore = true;
    for c in result.chars() {
        if c == '_' {
            if !prev_underscore {
                collapsed.push('_');
            }
            prev_underscore = true;
        } else {
            collapsed.push(c);
            prev_underscore = false;
        }
    }
    if collapsed.ends_with('_') {
        collapsed.pop();
    }
    collapsed
}

/// Generate the meta.lsx XML content
///
/// # Arguments
/// * `mod_name` - Display name of the mod
/// * `folder` - Folder name in Mods directory
/// * `author` - Author name
/// * `description` - Mod description
/// * `uuid` - Unique identifier for the mod
/// * `version_major` - Major version number
/// * `version_minor` - Minor version number
/// * `version_patch` - Patch version number
/// * `version_build` - Build version number
#[must_use]
pub fn generate_meta_lsx(
    mod_name: &str,
    folder: &str,
    author: &str,
    description: &str,
    uuid: &str,
    version_major: u32,
    version_minor: u32,
    version_patch: u32,
    version_build: u32,
) -> String {
    let version64 = version_to_int64(version_major, version_minor, version_patch, version_build);

    // Escape XML special characters
    let mod_name = escape_xml(mod_name);
    let folder = escape_xml(folder);
    let author = escape_xml(author);
    let description = escape_xml(description);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<save>
    <version major="4" minor="0" revision="9" build="331"/>
    <region id="Config">
        <node id="root">
            <children>
                <node id="Dependencies"/>
                <node id="ModuleInfo">
                    <attribute id="Author" type="LSString" value="{author}"/>
                    <attribute id="CharacterCreationLevelName" type="FixedString" value=""/>
                    <attribute id="Description" type="LSString" value="{description}"/>
                    <attribute id="Folder" type="LSString" value="{folder}"/>
                    <attribute id="LobbyLevelName" type="FixedString" value=""/>
                    <attribute id="MD5" type="LSString" value=""/>
                    <attribute id="MainMenuBackgroundVideo" type="FixedString" value=""/>
                    <attribute id="MenuLevelName" type="FixedString" value=""/>
                    <attribute id="Name" type="LSString" value="{mod_name}"/>
                    <attribute id="NumPlayers" type="uint8" value="4"/>
                    <attribute id="PhotoBooth" type="FixedString" value=""/>
                    <attribute id="StartupLevelName" type="FixedString" value=""/>
                    <attribute id="Tags" type="LSString" value=""/>
                    <attribute id="Type" type="FixedString" value="Add-on"/>
                    <attribute id="UUID" type="FixedString" value="{uuid}"/>
                    <attribute id="Version64" type="int64" value="{version64}"/>
                    <children>
                        <node id="PublishVersion">
                            <attribute id="Version64" type="int64" value="{version64}"/>
                        </node>
                        <node id="TargetModes">
                            <children>
                                <node id="Target">
                                    <attribute id="Object" type="FixedString" value="Story"/>
                                </node>
                            </children>
                        </node>
                    </children>
                </node>
            </children>
        </node>
    </region>
</save>"#,
        author = author,
        description = description,
        folder = folder,
        mod_name = mod_name,
        uuid = uuid,
        version64 = version64,
    )
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
