//! info.json generation for ``BaldursModManager`` compatibility

use std::fmt::Write;
use std::io::Read;
use std::path::Path;

use crate::formats::ModMetadata;
use crate::pak::PakOperations;

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
            message: "No meta.lsx found. Use 'maclarian mods meta' to generate one first.".to_string(),
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

/// Generate info.json from a single source (PAK file or mod directory)
///
/// # Arguments
/// * `source` - Path to either a .pak file or a mod directory
/// * `progress` - Progress callback
///
/// # Returns
/// `InfoJsonResult` with the generated JSON and status
#[must_use]
pub fn generate_info_json_from_source(
    source: &Path,
    progress: ModProgressCallback,
) -> InfoJsonResult {
    let is_pak = source
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"));

    if is_pak {
        generate_info_json_from_pak(source, progress)
    } else {
        generate_info_json_from_directory(source, progress)
    }
}

/// Generate info.json from a PAK file (reads meta.lsx from within)
fn generate_info_json_from_pak(pak_path: &Path, progress: ModProgressCallback) -> InfoJsonResult {
    progress(&ModProgress::with_file(
        ModPhase::Validating,
        0,
        3,
        "Reading meta.lsx from PAK",
    ));

    // Find and read meta.lsx from the PAK
    let Some(meta_lsx_content) = find_and_read_meta_lsx_from_pak(pak_path) else {
        return InfoJsonResult {
            success: false,
            content: None,
            message: "No meta.lsx found in PAK file. Use 'maclarian mods meta' to generate one first, then recreate the .pak with 'maclarian pak create'.".to_string(),
        };
    };

    // Parse the metadata
    let metadata = crate::formats::parse_meta_lsx(&meta_lsx_content);

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
    let pak_md5 = calculate_file_md5(&pak_path.to_string_lossy()).unwrap_or_default();

    progress(&ModProgress::with_file(
        ModPhase::GeneratingJson,
        2,
        3,
        "Generating info.json",
    ));

    let json = generate_info_json_content(&metadata, &pak_md5);

    progress(&ModProgress::new(ModPhase::Complete, 3, 3));

    InfoJsonResult {
        success: true,
        content: Some(json),
        message: "Generated successfully".to_string(),
    }
}

/// Generate info.json from a mod directory (finds PAK file for MD5)
fn generate_info_json_from_directory(
    dir_path: &Path,
    progress: ModProgressCallback,
) -> InfoJsonResult {
    progress(&ModProgress::with_file(
        ModPhase::Validating,
        0,
        3,
        "Finding meta.lsx",
    ));

    // Find meta.lsx in the directory
    let meta_lsx_content = find_and_read_meta_lsx(&dir_path.to_string_lossy());

    let Some(lsx_content) = meta_lsx_content else {
        return InfoJsonResult {
            success: false,
            content: None,
            message: "No meta.lsx found in directory. Use 'maclarian mods meta' to generate one first.".to_string(),
        };
    };

    // Parse the metadata
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
        "Finding and hashing PAK file",
    ));

    // Find .pak file in directory
    let pak_md5 = find_pak_and_calculate_md5(dir_path).unwrap_or_default();

    if pak_md5.is_empty() {
        return InfoJsonResult {
            success: false,
            content: None,
            message: "No .pak file found in directory".to_string(),
        };
    }

    progress(&ModProgress::with_file(
        ModPhase::GeneratingJson,
        2,
        3,
        "Generating info.json",
    ));

    let json = generate_info_json_content(&metadata, &pak_md5);

    progress(&ModProgress::new(ModPhase::Complete, 3, 3));

    InfoJsonResult {
        success: true,
        content: Some(json),
        message: "Generated successfully".to_string(),
    }
}

/// Find and read meta.lsx from within a PAK file
fn find_and_read_meta_lsx_from_pak(pak_path: &Path) -> Option<String> {
    // List files in PAK and find meta.lsx
    let files = PakOperations::list(pak_path).ok()?;

    let meta_path = files
        .iter()
        .find(|f| f.to_lowercase().ends_with("meta.lsx"))?;

    let bytes = PakOperations::read_file_bytes(pak_path, meta_path).ok()?;
    String::from_utf8(bytes).ok()
}

/// Find a .pak file in a directory and calculate its MD5
fn find_pak_and_calculate_md5(dir_path: &Path) -> Option<String> {
    let entries = std::fs::read_dir(dir_path).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"))
        {
            return calculate_file_md5(&path.to_string_lossy());
        }
    }

    None
}
