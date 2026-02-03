//! CLI commands for mod utilities

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use sevenz_rust::SevenZWriter;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use super::expand_globs;
use crate::cli::progress::simple_spinner;
use crate::mods::{
    generate_meta_lsx, parse_version_string, to_folder_name, validate_mod_structure,
    validate_pak_mod_structure,
};
use crate::pak::PakOperations;

/// Validate mod structure and PAK integrity
///
/// # Errors
/// Returns an error if glob expansion or validation fails.
pub fn validate(sources: &[PathBuf], quiet: bool) -> Result<()> {
    // Expand glob patterns
    let sources = expand_globs(sources)?;

    // Single source validation
    if sources.len() == 1 {
        return validate_single(&sources[0], quiet);
    }

    // Multiple sources - validate each
    let mut all_valid = true;
    for source in &sources {
        if !quiet {
            println!("Validating: {}", source.display());
        }
        if validate_single(source, quiet).is_err() {
            all_valid = false;
        }
        if !quiet {
            println!();
        }
    }

    if all_valid {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Validate a single mod
fn validate_single(source: &Path, quiet: bool) -> Result<()> {
    let is_pak = source
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"));

    let pb = if quiet {
        None
    } else {
        Some(simple_spinner("Validating mod structure..."))
    };

    let result = if is_pak {
        crate::mods::validate_pak_mod_structure_with_progress(source, &|p| {
            if let Some(ref pb) = pb {
                pb.set_message(p.phase.as_str().to_string());
            }
        })?
    } else {
        crate::mods::validate_mod_structure_with_progress(source, &|p| {
            if let Some(ref pb) = pb {
                pb.set_message(p.phase.as_str().to_string());
            }
        })
    };

    // Always check integrity for PAK files
    let integrity_result = if is_pak {
        Some(crate::mods::check_pak_integrity_with_progress(
            source,
            &|p| {
                if let Some(ref pb) = pb {
                    if let Some(ref file) = p.current_file {
                        pb.set_message(file.clone());
                    }
                }
            },
        )?)
    } else {
        None
    };

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    // Print structure elements
    if !result.structure.is_empty() {
        println!("Structure:");
        for item in &result.structure {
            println!("  {item}");
        }
    }

    // Print warnings
    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  - {warning}");
        }
    }

    // Print integrity results
    if let Some(ref integrity) = integrity_result {
        println!("\nIntegrity:");
        println!("  Files: {}", integrity.file_count);
        println!("  Size: {} bytes", integrity.total_size);
        if !integrity.issues.is_empty() {
            println!("  Issues:");
            for issue in &integrity.issues {
                println!("    - {issue}");
            }
        }
    }

    // Print result
    let valid = result.valid && integrity_result.as_ref().is_none_or(|i| i.valid);
    if valid {
        println!("\nValidation: PASSED");
        Ok(())
    } else {
        println!("\nValidation: FAILED");
        std::process::exit(1);
    }
}

/// Package mod for `BaldursModManager` (generates info.json alongside PAK)
///
/// # Errors
/// Returns an error if validation, PAK creation, or file writing fails.
///
/// # Panics
/// Panics if info.json generation succeeds but returns no content (internal invariant).
pub fn package(
    source: &Path,
    destination: &Path,
    compress: Option<&str>,
    quiet: bool,
) -> Result<()> {
    // Validate mod structure first (checks for meta.lsx)
    let is_pak = source
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"));

    if is_pak {
        let validation = validate_pak_mod_structure(source)
            .with_context(|| format!("Failed to validate PAK: {}", source.display()))?;
        if !validation.valid {
            anyhow::bail!(
                "No meta.lsx found in '{}'. Use 'maclarian mods meta' to generate one first, then recreate the .pak with 'maclarian pak create'.",
                source.display()
            );
        }
    } else {
        let validation = validate_mod_structure(source);
        if !validation.valid {
            anyhow::bail!(
                "No meta.lsx found in '{}'. Use 'maclarian mods meta' to generate one first.",
                source.display()
            );
        }
    }

    let pb = if quiet {
        None
    } else {
        Some(simple_spinner("Generating info.json..."))
    };

    // Generate info.json and get mod metadata
    let result = crate::mods::generate_info_json_from_source(source, &|p| {
        if let Some(ref pb) = pb {
            if let Some(ref msg) = p.current_file {
                pb.set_message(msg.clone());
            } else {
                pb.set_message(p.phase.as_str().to_string());
            }
        }
    });

    if !result.success {
        if let Some(ref pb) = pb {
            pb.finish_and_clear();
        }
        anyhow::bail!("{}", result.message);
    }

    let json_content = result.content.expect("success should have content");

    // Get mod name from the generated JSON
    let mod_name = extract_mod_name_from_json(&json_content)
        .context("Failed to extract mod name from generated info.json")?;

    // Determine source PAK path
    let source_pak_path = find_pak_path(source)?;

    // Create output directory: <destination>/<ModName>/
    let mod_output_dir = destination.join(&mod_name);
    fs::create_dir_all(&mod_output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            mod_output_dir.display()
        )
    })?;

    if let Some(ref pb) = pb {
        pb.set_message("Copying PAK file...".to_string());
    }

    // Copy PAK file to <destination>/<ModName>/<ModName>.pak
    let pak_filename = format!("{mod_name}.pak");
    let dest_pak_path = mod_output_dir.join(&pak_filename);
    fs::copy(&source_pak_path, &dest_pak_path)
        .with_context(|| format!("Failed to copy PAK to {}", dest_pak_path.display()))?;

    if let Some(ref pb) = pb {
        pb.set_message("Writing info.json...".to_string());
    }

    // Write info.json to <destination>/<ModName>/info.json
    let info_json_path = mod_output_dir.join("info.json");
    fs::write(&info_json_path, &json_content)
        .with_context(|| format!("Failed to write info.json to {}", info_json_path.display()))?;

    // Handle compression
    let final_output = if let Some(format) = compress {
        if let Some(ref pb) = pb {
            pb.set_message(format!("Compressing as {format}..."));
        }

        let archive_path = match format {
            "zip" => compress_to_zip(&mod_output_dir, &mod_name, destination)?,
            "7z" => compress_to_7z(&mod_output_dir, &mod_name, destination)?,
            _ => anyhow::bail!("Unsupported compression format: {format}"),
        };

        // Remove the uncompressed directory after successful compression
        fs::remove_dir_all(&mod_output_dir).with_context(|| {
            format!(
                "Failed to remove temp directory: {}",
                mod_output_dir.display()
            )
        })?;

        archive_path
    } else {
        mod_output_dir
    };

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    if !quiet {
        println!("Packaged mod to: {}", final_output.display());
    }

    Ok(())
}

/// Extract mod name (Folder field) from the generated info.json
fn extract_mod_name_from_json(json: &str) -> Option<String> {
    // Parse the JSON to get the Folder field
    let parsed: serde_json::Value = serde_json::from_str(json).ok()?;
    parsed["Mods"][0]["Folder"].as_str().map(String::from)
}

/// Find the PAK file path from source (either direct PAK or find in directory)
fn find_pak_path(source: &Path) -> Result<std::path::PathBuf> {
    let is_pak = source
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"));

    if is_pak {
        Ok(source.to_path_buf())
    } else {
        // Find .pak file in directory
        let entries = fs::read_dir(source)
            .with_context(|| format!("Failed to read directory: {}", source.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"))
            {
                return Ok(path);
            }
        }

        anyhow::bail!("No .pak file found in directory: {}", source.display())
    }
}

/// Compress directory to ZIP archive
fn compress_to_zip(
    source_dir: &Path,
    mod_name: &str,
    destination: &Path,
) -> Result<std::path::PathBuf> {
    let zip_path = destination.join(format!("{mod_name}.zip"));
    let file = File::create(&zip_path)
        .with_context(|| format!("Failed to create ZIP file: {}", zip_path.display()))?;

    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Add all files in the source directory
    for entry in fs::read_dir(source_dir)?.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            let archive_path = format!("{mod_name}/{file_name}");

            zip.start_file(&archive_path, options)?;
            let mut f = File::open(&path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }

    zip.finish()?;
    Ok(zip_path)
}

/// Compress directory to 7z archive
fn compress_to_7z(
    source_dir: &Path,
    mod_name: &str,
    destination: &Path,
) -> Result<std::path::PathBuf> {
    let archive_path = destination.join(format!("{mod_name}.7z"));
    let file = File::create(&archive_path)
        .with_context(|| format!("Failed to create 7z file: {}", archive_path.display()))?;

    let mut sz = SevenZWriter::new(file)?;

    // Add all files in the source directory
    for entry in fs::read_dir(source_dir)?.flatten() {
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy();
            let entry_name = format!("{mod_name}/{file_name}");

            let mut source_file = File::open(&path)?;
            sz.push_archive_entry(
                sevenz_rust::SevenZArchiveEntry::from_path(&path, entry_name),
                Some(&mut source_file),
            )?;
        }
    }

    sz.finish()?;
    Ok(archive_path)
}

/// Generate meta.lsx metadata file for a mod
///
/// # Errors
/// Returns an error if version parsing or file writing fails.
pub fn meta(
    source: &Path,
    name: &str,
    author: &str,
    description: &str,
    folder: Option<&str>,
    uuid: Option<&str>,
    version: &str,
) -> Result<()> {
    // Parse version string
    let (major, minor, patch, build) = parse_version_string(version).with_context(|| {
        format!(
            "Invalid version format: {version}. Expected: major.minor.patch.build (e.g., 1.0.0.0)"
        )
    })?;

    // Generate folder name from mod name if not provided, always sanitized
    let folder = to_folder_name(folder.unwrap_or(name));

    // Generate UUID if not provided
    let uuid = uuid.map_or_else(|| uuid::Uuid::new_v4().to_string(), String::from);

    // Generate the meta.lsx content
    let content = generate_meta_lsx(
        name,
        &folder,
        author,
        description,
        &uuid,
        major,
        minor,
        patch,
        build,
    );

    // Create output directory: <source>/Mods/<Folder>/
    let output_dir = source.join("Mods").join(&folder);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create directory: {}", output_dir.display()))?;

    // Write meta.lsx
    let meta_path = output_dir.join("meta.lsx");
    fs::write(&meta_path, &content)
        .with_context(|| format!("Failed to write meta.lsx to {}", meta_path.display()))?;

    println!("Generated: {}", meta_path.display());
    println!("  Name:    {name}");
    println!("  Folder:  {folder}");
    println!("  Author:  {author}");
    println!("  UUID:    {uuid}");
    println!("  Version: {version}");

    Ok(())
}

/// Find files modified by multiple mods (potential conflicts)
///
/// # Errors
/// Returns an error if glob expansion or PAK reading fails.
pub fn conflicts(sources: &[PathBuf], quiet: bool) -> Result<()> {
    // Expand glob patterns
    let sources = expand_globs(sources)?;

    if sources.len() < 2 {
        anyhow::bail!("At least 2 sources are required to check for conflicts");
    }

    let pb = if quiet {
        None
    } else {
        Some(simple_spinner("Scanning mods for conflicts..."))
    };

    // Build map: file_path -> Vec<source_name>
    let mut file_sources: HashMap<String, Vec<String>> = HashMap::new();

    for source in &sources {
        if let Some(ref pb) = pb {
            pb.set_message(format!("Scanning {}...", source.display()));
        }

        let name = source.file_name().map_or_else(
            || source.display().to_string(),
            |n| n.to_string_lossy().to_string(),
        );

        let files = if source.is_dir() {
            collect_mod_files(source)?
        } else {
            // PAK file
            PakOperations::list(source)
                .with_context(|| format!("Failed to list PAK: {}", source.display()))?
        };

        for file in files {
            file_sources.entry(file).or_default().push(name.clone());
        }
    }

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    // Filter to conflicts (files with 2+ sources)
    let mut conflicts: Vec<_> = file_sources
        .into_iter()
        .filter(|(_, sources)| sources.len() > 1)
        .collect();

    if conflicts.is_empty() {
        println!("No conflicts found across {} mods.", sources.len());
        return Ok(());
    }

    // Sort by file path for consistent output
    conflicts.sort_by(|a, b| a.0.cmp(&b.0));

    println!("Files modified by multiple mods:\n");

    for (file_path, mod_names) in &conflicts {
        println!("{file_path}");
        for (i, mod_name) in mod_names.iter().enumerate() {
            let prefix = if i == mod_names.len() - 1 {
                "└─"
            } else {
                "├─"
            };
            println!("  {prefix} {mod_name}");
        }
        println!();
    }

    // Count unique mods involved in conflicts
    let mut mods_with_conflicts: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (_, mod_names) in &conflicts {
        for name in mod_names {
            mods_with_conflicts.insert(name);
        }
    }

    println!(
        "Summary: {} conflicting file(s) across {} mod(s)",
        conflicts.len(),
        mods_with_conflicts.len()
    );

    Ok(())
}

/// Collect all file paths from a mod directory (relative paths)
fn collect_mod_files(dir: &Path) -> Result<Vec<String>> {
    let mut files = Vec::new();
    collect_files_recursive(dir, dir, &mut files)?;
    Ok(files)
}

fn collect_files_recursive(base: &Path, current: &Path, files: &mut Vec<String>) -> Result<()> {
    let entries = fs::read_dir(current)
        .with_context(|| format!("Failed to read directory: {}", current.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files_recursive(base, &path, files)?;
        } else if path.is_file() {
            // Get relative path from base
            if let Ok(rel) = path.strip_prefix(base) {
                // Normalize to forward slashes
                let rel_str = rel.to_string_lossy().replace('\\', "/");
                files.push(rel_str);
            }
        }
    }

    Ok(())
}
