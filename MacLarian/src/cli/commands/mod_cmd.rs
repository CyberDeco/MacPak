//! CLI commands for mod utilities

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use sevenz_rust::SevenZWriter;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use crate::cli::progress::simple_spinner;
use crate::mods::{
    BatchValidationOptions, generate_meta_lsx, parse_version_string, to_folder_name,
    validate_mod_structure, validate_pak_mod_structure,
};

/// Validate mod structure with batch support
pub fn validate_batch(
    source: &Path,
    recursive: bool,
    check_integrity: bool,
    paks_only: bool,
    dirs_only: bool,
    quiet: bool,
) -> Result<()> {
    // If not recursive, use single-mod validation
    if !recursive {
        return validate_single(source, check_integrity, quiet);
    }

    // Batch validation
    let options = BatchValidationOptions {
        include_paks: !dirs_only,
        include_directories: !paks_only,
        check_integrity,
        max_depth: None,
    };

    let pb = if quiet {
        None
    } else {
        Some(simple_spinner("Scanning for mods..."))
    };

    let result = crate::mods::validate_directory_recursive_with_progress(source, &options, &|p| {
        if let Some(ref pb) = pb {
            if let Some(ref file) = p.current_file {
                pb.set_message(format!("[{}/{}] {}", p.current, p.total, file));
            } else {
                pb.set_message(p.phase.as_str().to_string());
            }
        }
    })?;

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    if result.total == 0 {
        println!("No mods found in {}", source.display());
        return Ok(());
    }

    // Print results for each mod
    for entry in &result.entries {
        let status = if entry.result.valid { "✓" } else { "✗" };
        let type_str = if entry.is_pak { "PAK" } else { "DIR" };
        println!("{status} [{type_str}] {}", entry.name);

        if !quiet {
            // Print structure
            for item in &entry.result.structure {
                println!("    {item}");
            }

            // Print warnings
            for warning in &entry.result.warnings {
                println!("    ⚠ {warning}");
            }

            // Print integrity issues
            if let Some(ref integrity) = entry.integrity {
                if !integrity.valid {
                    for issue in &integrity.issues {
                        println!("    ⚠ {issue}");
                    }
                } else if !quiet {
                    println!(
                        "    Integrity: OK ({} files, {} bytes)",
                        integrity.file_count, integrity.total_size
                    );
                }
            }
        }
    }

    // Print summary
    println!("\n{}", result.summary());

    if result.all_valid() {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Validate a single mod (helper for non-recursive validation)
fn validate_single(source: &Path, check_integrity: bool, quiet: bool) -> Result<()> {
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

    // Check integrity if requested and it's a PAK
    let integrity_result = if check_integrity && is_pak {
        Some(crate::mods::check_pak_integrity_with_progress(source, &|p| {
            if let Some(ref pb) = pb {
                if let Some(ref file) = p.current_file {
                    pb.set_message(file.clone());
                }
            }
        })?)
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
    let valid = result.valid && integrity_result.as_ref().map_or(true, |i| i.valid);
    if valid {
        println!("\nValidation: PASSED");
        Ok(())
    } else {
        println!("\nValidation: FAILED");
        std::process::exit(1);
    }
}

/// Dry-run PAK creation validation
pub fn dry_run(source: &Path, quiet: bool) -> Result<()> {
    let pb = if quiet {
        None
    } else {
        Some(simple_spinner("Validating directory for PAK creation..."))
    };

    let result = crate::mods::validate_for_pak_creation_with_progress(source, &|p| {
        if let Some(ref pb) = pb {
            if let Some(ref file) = p.current_file {
                pb.set_message(file.clone());
            }
        }
    })?;

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    // Print results
    println!("Files: {}", result.file_count);
    println!("Total size: {} bytes ({:.2} MB)", result.total_size, result.total_size as f64 / 1024.0 / 1024.0);

    if !result.errors.is_empty() {
        println!("\nErrors:");
        for error in &result.errors {
            println!("  ✗ {error}");
        }
    }

    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  ⚠ {warning}");
        }
    }

    if result.valid {
        println!("\nDry run: PAK creation would succeed");
        Ok(())
    } else {
        println!("\nDry run: PAK creation would FAIL");
        std::process::exit(1);
    }
}

/// Check PAK file integrity
pub fn integrity(sources: &[std::path::PathBuf], quiet: bool) -> Result<()> {
    let mut all_valid = true;

    for source in sources {
        let pb = if quiet {
            None
        } else {
            Some(simple_spinner("Checking PAK integrity..."))
        };

        let result = crate::mods::check_pak_integrity_with_progress(source, &|p| {
            if let Some(ref pb) = pb {
                if let Some(ref file) = p.current_file {
                    pb.set_message(file.clone());
                }
            }
        })?;

        if let Some(pb) = pb {
            pb.finish_and_clear();
        }

        let status = if result.valid { "✓" } else { "✗" };
        println!(
            "{status} {} ({} files, {} bytes)",
            source.display(),
            result.file_count,
            result.total_size
        );

        if !result.issues.is_empty() {
            for issue in &result.issues {
                println!("    ⚠ {issue}");
            }
        }

        if !result.valid {
            all_valid = false;
        }
    }

    if all_valid {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

/// Package mod for BaldursModManager (generates info.json alongside PAK)
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
    let (major, minor, patch, build) = parse_version_string(version)
        .with_context(|| format!("Invalid version format: {version}. Expected: major.minor.patch.build (e.g., 1.0.0.0)"))?;

    // Generate folder name from mod name if not provided, always sanitized
    let folder = to_folder_name(folder.unwrap_or(name));

    // Generate UUID if not provided
    let uuid = uuid
        .map(String::from)
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    // Generate the meta.lsx content
    let content = generate_meta_lsx(name, &folder, author, description, &uuid, major, minor, patch, build);

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
