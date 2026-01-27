//! CLI commands for PAK operations

use std::collections::HashMap;
use std::path::Path;

use indicatif::{ProgressBar, ProgressStyle};

use crate::pak::{batch_create, batch_extract, find_pak_files, find_packable_folders, PakOperations};
use crate::search::FileType;

/// Show aggregate info about a PAK file
pub fn info(pak: &Path) -> anyhow::Result<()> {
    let entries = PakOperations::list_detailed(pak)?;

    // Calculate statistics
    let total_files = entries.len();
    let total_compressed: u64 = entries.iter().map(|e| u64::from(e.size_compressed)).sum();
    let total_decompressed: u64 = entries.iter().map(|e| u64::from(e.size_decompressed)).sum();

    // Count by file type
    let mut by_type: HashMap<FileType, (usize, u64)> = HashMap::new();
    for entry in &entries {
        let ext = entry
            .path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();
        let file_type = FileType::from_extension(&ext);
        let (count, size) = by_type.entry(file_type).or_insert((0, 0));
        *count += 1;
        *size += u64::from(entry.size_decompressed);
    }

    // Find largest files
    let mut largest: Vec<_> = entries.iter().collect();
    largest.sort_by_key(|e| std::cmp::Reverse(e.size_decompressed));

    // Print statistics
    println!("PAK Information: {}", pak.display());
    println!();
    println!("Total files: {total_files}");
    println!(
        "Total size (compressed): {} ({} bytes)",
        format_size(total_compressed),
        total_compressed
    );
    println!(
        "Total size (decompressed): {} ({} bytes)",
        format_size(total_decompressed),
        total_decompressed
    );
    if total_compressed > 0 {
        let ratio = (total_decompressed as f64) / (total_compressed as f64);
        println!("Compression ratio: {ratio:.2}x");
    }
    println!();

    // Print by type
    println!("Files by type:");
    let mut type_list: Vec<_> = by_type.iter().collect();
    type_list.sort_by_key(|(_, (count, _))| std::cmp::Reverse(*count));
    for (file_type, (count, size)) in type_list {
        println!(
            "  {:8} {:>6} files  {:>10}",
            file_type.display_name(),
            count,
            format_size(*size)
        );
    }
    println!();

    // Print largest files
    println!("Largest files:");
    for entry in largest.iter().take(10) {
        println!(
            "  {:>10}  {}",
            format_size(u64::from(entry.size_decompressed)),
            entry.path.display()
        );
    }

    Ok(())
}

/// Find all PAK files in a directory
pub fn find(dir: &Path) -> anyhow::Result<()> {
    let paks = find_pak_files(dir);

    if paks.is_empty() {
        println!("No PAK files found in: {}", dir.display());
    } else {
        println!("Found {} PAK files:", paks.len());
        for pak in &paks {
            // Show relative path if possible
            let display = pak
                .strip_prefix(dir)
                .unwrap_or(pak.as_path())
                .display();
            println!("  {display}");
        }
    }

    Ok(())
}

/// Batch extract PAK files
pub fn batch_extract_cmd(source: &Path, dest: &Path) -> anyhow::Result<()> {
    let paks = find_pak_files(source);

    if paks.is_empty() {
        println!("No PAK files found in: {}", source.display());
        return Ok(());
    }

    println!("Found {} PAK files to extract", paks.len());

    let pb = ProgressBar::new(paks.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
            .expect("valid template")
            .progress_chars("##-"),
    );

    let result = batch_extract(&paks, source, dest, |progress| {
        pb.set_position(progress.current as u64);
        if let Some(ref name) = progress.current_file {
            pb.set_message(name.clone());
        }
    });

    pb.finish_and_clear();

    println!();
    println!("Extraction complete:");
    println!("  Success: {}", result.success_count);
    println!("  Failed: {}", result.fail_count);

    if result.fail_count > 0 {
        println!();
        println!("Failures:");
        for msg in result.results.iter().filter(|m| m.starts_with("Failed")) {
            println!("  {msg}");
        }
    }

    Ok(())
}

/// Batch create PAK files
pub fn batch_create_cmd(source: &Path, dest: &Path) -> anyhow::Result<()> {
    let folders = find_packable_folders(source);

    if folders.is_empty() {
        println!("No packable folders found in: {}", source.display());
        return Ok(());
    }

    println!("Found {} folders to pack", folders.len());

    let pb = ProgressBar::new(folders.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
            .expect("valid template")
            .progress_chars("##-"),
    );

    let result = batch_create(&folders, source, dest, |progress| {
        pb.set_position(progress.current as u64);
        if let Some(ref name) = progress.current_file {
            pb.set_message(name.clone());
        }
    });

    pb.finish_and_clear();

    println!();
    println!("Creation complete:");
    println!("  Success: {}", result.success_count);
    println!("  Failed: {}", result.fail_count);

    if result.fail_count > 0 {
        println!();
        println!("Failures:");
        for msg in result.results.iter().filter(|m| m.starts_with("Failed")) {
            println!("  {msg}");
        }
    }

    Ok(())
}

/// Format byte size for display
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
