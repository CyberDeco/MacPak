//! CLI commands for PAK operations

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::cli::progress::simple_bar;
use crate::pak::{CompressionMethod, PakOperations};

/// Default BG3 installation paths
const BG3_PATHS: &[&str] = &[
    // macOS
    "~/Library/Application Support/Steam/steamapps/common/Baldurs Gate 3/Baldur's Gate 3.app/Contents/Data",
    // Windows
    "C:/Program Files (x86)/Steam/steamapps/common/Baldurs Gate 3/Data",
    // Linux
    "~/.steam/steam/steamapps/common/Baldurs Gate 3/Data",
];

/// Check if a path is within a BG3 installation directory
fn is_bg3_install_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    for bg3_path in BG3_PATHS {
        let expanded = shellexpand::tilde(bg3_path);
        if path_str.starts_with(expanded.as_ref()) {
            return true;
        }
    }
    false
}

/// Warn if destination is within BG3 install path
fn warn_if_bg3_path(destination: &Path) {
    if is_bg3_install_path(destination) {
        eprintln!("WARNING: Destination is within BG3 installation directory!");
        eprintln!("         This may corrupt your game installation.");
        eprintln!("         Path: {}", destination.display());
        eprintln!();
    }
}

/// Simple glob pattern matching (supports * and ?)
fn matches_glob(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();
    matches_glob_recursive(&pattern_chars, &text_chars, 0, 0)
}

fn matches_glob_recursive(pattern: &[char], text: &[char], pi: usize, ti: usize) -> bool {
    if pi == pattern.len() && ti == text.len() {
        return true;
    }
    if pi == pattern.len() {
        return false;
    }

    match pattern[pi] {
        '*' => {
            for i in ti..=text.len() {
                if matches_glob_recursive(pattern, text, pi + 1, i) {
                    return true;
                }
            }
            false
        }
        '?' => {
            if ti < text.len() {
                matches_glob_recursive(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
        c => {
            if ti < text.len() && text[ti].eq_ignore_ascii_case(&c) {
                matches_glob_recursive(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}

/// Check if a string looks like a UUID (with or without dashes)
fn looks_like_uuid(s: &str) -> bool {
    let clean: String = s.chars().filter(char::is_ascii_hexdigit).collect();
    clean.len() == 32
}

/// Normalize a UUID for matching (remove dashes, lowercase)
fn normalize_uuid(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_hexdigit)
        .collect::<String>()
        .to_lowercase()
}

/// Check if a path contains a UUID (normalized comparison)
fn path_contains_uuid(path: &str, uuid: &str) -> bool {
    let normalized_uuid = normalize_uuid(uuid);
    let path_lower = path.to_lowercase();

    // Check for UUID with dashes
    if normalized_uuid.len() == 32 {
        let with_dashes = format!(
            "{}-{}-{}-{}-{}",
            &normalized_uuid[0..8],
            &normalized_uuid[8..12],
            &normalized_uuid[12..16],
            &normalized_uuid[16..20],
            &normalized_uuid[20..32]
        );
        if path_lower.contains(&with_dashes) || path_lower.contains(&normalized_uuid) {
            return true;
        }
    }

    false
}

/// Format byte size for human-readable output
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// Extract files from PAK archive(s)
pub fn extract(
    sources: &[PathBuf],
    destination: &Path,
    filter: Option<&str>,
    file: Option<&str>,
    quiet: bool,
) -> anyhow::Result<()> {
    // Warn if destination is BG3 install path
    warn_if_bg3_path(destination);

    // Handle multiple sources (batch extraction)
    if sources.len() > 1 {
        return extract_batch(sources, destination, filter, quiet);
    }

    let source = &sources[0];

    // Single file extraction
    if let Some(file_paths) = file {
        let paths: Vec<&str> = file_paths.split(',').map(str::trim).collect();
        println!("Extracting {} file(s) from {}", paths.len(), source.display());
        let dest = destination.to_path_buf();
        PakOperations::extract_files(source, &dest, &paths)?;
        println!("Extraction complete");
        return Ok(());
    }

    // Filtered extraction
    if let Some(pattern) = filter {
        println!("Extracting files matching: {pattern}");

        let all_files = PakOperations::list(source)?;
        let matching: Vec<String> = all_files
            .iter()
            .filter(|f| {
                let filename = Path::new(f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(f);
                matches_glob(pattern, filename) || matches_glob(pattern, f)
            })
            .cloned()
            .collect();

        if matching.is_empty() {
            println!("No files match pattern: {pattern}");
            return Ok(());
        }

        println!("Found {} matching files", matching.len());

        let dest = destination.to_path_buf();
        if !quiet {
            let pb = simple_bar(matching.len() as u64, "Extracting");
            let count = AtomicUsize::new(0);
            let matching_refs: Vec<&str> = matching.iter().map(String::as_str).collect();

            PakOperations::extract_files_with_progress(source, &dest, &matching_refs, &|progress| {
                let n = count.fetch_add(1, Ordering::SeqCst) + 1;
                pb.set_position(n as u64);
                if let Some(name) = &progress.current_file {
                    let short_name = Path::new(name)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(name);
                    pb.set_message(short_name.to_string());
                }
            })?;
            pb.finish_with_message("done");
        } else {
            let matching_refs: Vec<&str> = matching.iter().map(String::as_str).collect();
            PakOperations::extract_files(source, &dest, &matching_refs)?;
        }

        println!("Extraction complete");
        return Ok(());
    }

    // Full extraction
    let dest = destination.to_path_buf();
    if !quiet {
        let files = PakOperations::list(source)?;
        let total = files.len() as u64;
        println!("Extracting {total} files from {}", source.display());

        let pb = simple_bar(total, "Extracting");
        let count = AtomicUsize::new(0);

        PakOperations::extract_with_progress(source, &dest, &|progress| {
            let n = count.fetch_add(1, Ordering::SeqCst) + 1;
            pb.set_position(n as u64);
            if let Some(name) = &progress.current_file {
                let short_name = Path::new(name)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(name);
                pb.set_message(short_name.to_string());
            }
        })?;
        pb.finish_with_message("done");
    } else {
        println!("Extracting {} to {}", source.display(), destination.display());
        PakOperations::extract(source, &dest)?;
    }

    println!("Extraction complete");
    Ok(())
}

/// Batch extract multiple PAK files
fn extract_batch(
    sources: &[PathBuf],
    destination: &Path,
    filter: Option<&str>,
    quiet: bool,
) -> anyhow::Result<()> {
    println!("Batch extracting {} PAK files", sources.len());

    let mut success = 0;
    let mut failed = 0;

    for source in sources {
        let pak_name = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let pak_dest = destination.join(pak_name);

        if !quiet {
            println!("Extracting: {}", source.display());
        }

        match extract(std::slice::from_ref(source), &pak_dest, filter, None, quiet) {
            Ok(()) => success += 1,
            Err(e) => {
                eprintln!("Failed to extract {}: {e}", source.display());
                failed += 1;
            }
        }
    }

    println!();
    println!("Batch extraction complete:");
    println!("  Success: {success}");
    if failed > 0 {
        println!("  Failed: {failed}");
    }

    Ok(())
}

/// Create PAK file(s) from directory(ies)
pub fn create(
    sources: &[PathBuf],
    destination: &Path,
    compression: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    // Warn if destination is BG3 install path
    warn_if_bg3_path(destination);

    let method = match compression.to_lowercase().as_str() {
        "lz4" => CompressionMethod::Lz4,
        "zlib" => CompressionMethod::Zlib,
        "none" => CompressionMethod::None,
        other => {
            anyhow::bail!("Unknown compression method: '{other}'. Valid options: lz4, zlib, none");
        }
    };

    // Handle multiple sources (batch creation)
    if sources.len() > 1 {
        return create_batch(sources, destination, method, quiet);
    }

    let source = &sources[0];

    // For single source, destination is the PAK filename
    println!(
        "Creating PAK from {} to {} (compression: {:?})",
        source.display(),
        destination.display(),
        method
    );

    let dest = destination.to_path_buf();
    if !quiet {
        let pb = simple_bar(100, "Creating PAK");
        PakOperations::create_with_compression_and_progress(source, &dest, method, &|p| {
            pb.set_position((p.percentage() * 100.0) as u64);
            if let Some(ref file) = p.current_file {
                pb.set_message(file.clone());
            }
        })?;
        pb.finish_and_clear();
    } else {
        PakOperations::create_with_compression(source, &dest, method)?;
    }

    println!("PAK created successfully");
    Ok(())
}

/// Batch create PAK files from multiple directories
fn create_batch(
    sources: &[PathBuf],
    destination: &Path,
    method: CompressionMethod,
    quiet: bool,
) -> anyhow::Result<()> {
    println!("Batch creating {} PAK files", sources.len());

    // Ensure destination directory exists
    std::fs::create_dir_all(destination)?;

    let mut success = 0;
    let mut failed = 0;

    for source in sources {
        let pak_name = source
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let pak_dest = destination.join(format!("{pak_name}.pak"));

        if !quiet {
            println!("Creating: {}", pak_dest.display());
        }

        if !quiet {
            let pb = simple_bar(100, "Creating");
            match PakOperations::create_with_compression_and_progress(source, &pak_dest, method, &|p| {
                pb.set_position((p.percentage() * 100.0) as u64);
            }) {
                Ok(()) => {
                    pb.finish_and_clear();
                    success += 1;
                }
                Err(e) => {
                    pb.finish_and_clear();
                    eprintln!("Failed to create {}: {e}", pak_dest.display());
                    failed += 1;
                }
            }
        } else {
            match PakOperations::create_with_compression(source, &pak_dest, method) {
                Ok(()) => success += 1,
                Err(e) => {
                    eprintln!("Failed to create {}: {e}", pak_dest.display());
                    failed += 1;
                }
            }
        }
    }

    println!();
    println!("Batch creation complete:");
    println!("  Success: {success}");
    if failed > 0 {
        println!("  Failed: {failed}");
    }

    Ok(())
}

/// List contents of a PAK file
pub fn list(
    source: &Path,
    detailed: bool,
    filter: Option<&str>,
    count: bool,
    _quiet: bool,
) -> anyhow::Result<()> {
    // Check if filter looks like a UUID for smart matching
    let is_uuid_filter = filter.is_some_and(looks_like_uuid);

    if detailed {
        let entries = PakOperations::list_detailed(source)?;

        // Filter entries
        let filtered: Vec<_> = if let Some(pattern) = filter {
            if is_uuid_filter {
                entries
                    .iter()
                    .filter(|e| path_contains_uuid(&e.path.to_string_lossy(), pattern))
                    .collect()
            } else {
                entries
                    .iter()
                    .filter(|e| {
                        let path_str = e.path.to_string_lossy();
                        let filename = e
                            .path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&path_str);
                        matches_glob(pattern, filename) || matches_glob(pattern, &path_str)
                    })
                    .collect()
            }
        } else {
            entries.iter().collect()
        };

        if count {
            println!("{}", filtered.len());
            return Ok(());
        }

        // Print header
        println!("{:>10}  {:>10}  {:>6}  PATH", "SIZE", "COMPRESSED", "RATIO");

        // Print entries
        for entry in &filtered {
            let ratio = if entry.size_decompressed > 0 {
                (entry.size_compressed as f64 / entry.size_decompressed as f64) * 100.0
            } else {
                100.0
            };

            println!(
                "{:>10}  {:>10}  {:>5.1}%  {}",
                format_size(u64::from(entry.size_decompressed)),
                format_size(u64::from(entry.size_compressed)),
                ratio,
                entry.path.display()
            );
        }

        // Print summary
        let total_decompressed: u64 = filtered.iter().map(|e| u64::from(e.size_decompressed)).sum();
        let total_compressed: u64 = filtered.iter().map(|e| u64::from(e.size_compressed)).sum();
        let overall_ratio = if total_decompressed > 0 {
            (total_compressed as f64 / total_decompressed as f64) * 100.0
        } else {
            100.0
        };

        println!();
        println!(
            "{} files, {} total ({} compressed, {:.1}% ratio)",
            filtered.len(),
            format_size(total_decompressed),
            format_size(total_compressed),
            overall_ratio
        );
    } else {
        // Simple listing
        let files = PakOperations::list(source)?;

        // Filter files
        let filtered: Vec<_> = if let Some(pattern) = filter {
            if is_uuid_filter {
                files
                    .iter()
                    .filter(|f| path_contains_uuid(f, pattern))
                    .collect()
            } else {
                files
                    .iter()
                    .filter(|f| {
                        let filename = Path::new(f)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(f);
                        matches_glob(pattern, filename) || matches_glob(pattern, f)
                    })
                    .collect()
            }
        } else {
            files.iter().collect()
        };

        if count {
            println!("{}", filtered.len());
            return Ok(());
        }

        for file in filtered {
            println!("{file}");
        }
    }

    Ok(())
}
