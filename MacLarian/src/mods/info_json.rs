//! info.json generation for `BaldursModManager` compatibility

use std::fmt::Write;
use std::io::Read;
use std::path::Path;

use crate::formats::ModMetadata;

use super::types::{ModPhase, ModProgress, ModProgressCallback};

/// Result of info.json generation
pub struct InfoJsonResult {
    /// Whether generation was successful
    pub success: bool,
    /// Generated JSON content (if successful)
    pub content: Option<String>,
    /// Status message
    pub message: String,
}

/// Generate info.json content for a mod
///
/// # Arguments
/// * `source_dir` - Path to the mod source directory (extracted PAK contents)
/// * `pak_path` - Path to the PAK file (for MD5 calculation)
///
/// # Returns
/// `InfoJsonResult` with the generated JSON and status
#[must_use]
pub fn generate_info_json(source_dir: &str, pak_path: &str) -> InfoJsonResult {
    generate_info_json_with_progress(source_dir, pak_path, &|_| {})
}

/// Generate info.json content for a mod with progress callback
///
/// # Arguments
/// * `source_dir` - Path to the mod source directory (extracted PAK contents)
/// * `pak_path` - Path to the PAK file (for MD5 calculation)
/// * `progress` - Progress callback
///
/// # Returns
/// `InfoJsonResult` with the generated JSON and status
#[must_use]
pub fn generate_info_json_with_progress(
    source_dir: &str,
    pak_path: &str,
    progress: ModProgressCallback,
) -> InfoJsonResult {
    progress(&ModProgress::with_file(
        ModPhase::Validating,
        0,
        3,
        "Finding meta.lsx",
    ));

    // Find meta.lsx in the source directory
    let meta_lsx_content = find_and_read_meta_lsx(source_dir);

    let Some(lsx_content) = meta_lsx_content else {
        return InfoJsonResult {
            success: false,
            content: None,
            message: "No meta.lsx found".to_string(),
        };
    };

    // Parse the metadata using MacLarian
    let metadata = crate::formats::parse_meta_lsx(&lsx_content);

    if metadata.uuid.is_empty() {
        return InfoJsonResult {
            success: false,
            content: None,
            message: "meta.lsx missing UUID".to_string(),
        };
    }

    progress(&ModProgress::with_file(
        ModPhase::CalculatingHash,
        1,
        3,
        "Calculating PAK MD5",
    ));

    // Calculate MD5 of the PAK file
    let pak_md5 = calculate_file_md5(pak_path).unwrap_or_default();

    progress(&ModProgress::with_file(
        ModPhase::GeneratingJson,
        2,
        3,
        "Generating info.json",
    ));

    // Generate the info.json content
    let json = generate_info_json_content(&metadata, &pak_md5);

    progress(&ModProgress::new(ModPhase::Complete, 3, 3));

    InfoJsonResult {
        success: true,
        content: Some(json),
        message: "Generated successfully".to_string(),
    }
}

/// Find and read meta.lsx from a mod source directory
#[must_use]
pub fn find_and_read_meta_lsx(source_dir: &str) -> Option<String> {
    let source_path = Path::new(source_dir);

    // Look for Mods/*/meta.lsx pattern
    let mods_dir = source_path.join("Mods");
    if mods_dir.exists()
        && mods_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&mods_dir)
    {
        for entry in entries.flatten() {
            let meta_path = entry.path().join("meta.lsx");
            if meta_path.exists()
                && let Ok(content) = std::fs::read_to_string(&meta_path)
            {
                return Some(content);
            }
        }
    }

    // Also check for meta.lsx directly in source (some mod structures)
    let direct_meta = source_path.join("meta.lsx");
    if direct_meta.exists()
        && let Ok(content) = std::fs::read_to_string(&direct_meta)
    {
        return Some(content);
    }

    None
}

/// Calculate MD5 hash of a file (streaming for large files)
#[must_use]
pub fn calculate_file_md5(file_path: &str) -> Option<String> {
    let mut file = std::fs::File::open(file_path).ok()?;
    let mut hasher = md5::Context::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).ok()?;
        if bytes_read == 0 {
            break;
        }
        hasher.consume(&buffer[..bytes_read]);
    }

    let digest = hasher.compute();
    // Convert digest bytes to hex string (MD5 = 16 bytes = 32 hex chars)
    let mut hex = String::with_capacity(32);
    for b in digest.iter() {
        let _ = write!(hex, "{b:02x}");
    }
    Some(hex)
}

/// Generate info.json content from metadata
fn generate_info_json_content(metadata: &ModMetadata, pak_md5: &str) -> String {
    // Use raw Version64 integer as string (matches BG3 Modder's Multitool format)
    let version_json = match metadata.version64 {
        Some(v) => format!("\"{v}\""),
        None => "null".to_string(),
    };

    // Escape strings for JSON
    let name = escape_json_string(&metadata.name);
    let folder = escape_json_string(&metadata.folder);
    let author = escape_json_string(&metadata.author);
    let description = escape_json_string(&metadata.description);

    // Get current timestamp in ISO format
    let now = chrono::Utc::now();
    let created = now.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    // Generate a random Group UUID
    let group_uuid = uuid::Uuid::new_v4().to_string();

    format!(
        r#"{{"Mods":[{{"Author":"{}","Name":"{}","Folder":"{}","Version":{},"Description":"{}","UUID":"{}","Created":"{}","Dependencies":[],"Group":"{}"}}],"MD5":"{}"}}"#,
        author,
        name,
        folder,
        version_json,
        description,
        metadata.uuid,
        created,
        group_uuid,
        pak_md5
    )
}

/// Escape a string for JSON
fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
