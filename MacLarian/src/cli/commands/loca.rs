//! CLI commands for LOCA localization file operations

use std::path::Path;

use crate::formats::loca::{
    ExportFormat, LocaResource, read_loca, write_loca,
    export_for_translation, import_translations,
};

/// Search for entries in a LOCA file
///
/// If `search_handle` is true, searches handle/key names instead of text content.
pub fn search(
    path: &Path,
    query: &str,
    search_handle: bool,
    limit: usize,
    quiet: bool,
) -> anyhow::Result<()> {
    let resource = read_loca(path)?;
    let query_lower = query.to_lowercase();

    if search_handle {
        // Search by handle/key
        search_by_handle(&resource, &query_lower, limit, quiet)
    } else {
        // Search by text content
        search_by_text(&resource, &query_lower, limit, quiet)
    }
}

/// Search entries by handle/key name
fn search_by_handle(
    resource: &crate::formats::loca::LocaResource,
    query_lower: &str,
    limit: usize,
    quiet: bool,
) -> anyhow::Result<()> {
    // Try exact match first
    if let Some(entry) = resource
        .entries
        .iter()
        .find(|e| e.key.to_lowercase() == *query_lower)
    {
        if !quiet {
            println!("Key: {}", entry.key);
            println!("Version: {}", entry.version);
            println!("Text:");
        }
        println!("{}", entry.text);
        return Ok(());
    }

    // Try partial match
    let matches: Vec<_> = resource
        .entries
        .iter()
        .filter(|e| e.key.to_lowercase().contains(query_lower))
        .take(limit)
        .collect();

    if matches.is_empty() {
        if !quiet {
            println!("No entries found matching handle '{query_lower}'");
        }
    } else if matches.len() == 1 {
        let entry = matches[0];
        if !quiet {
            println!("Key: {}", entry.key);
            println!("Version: {}", entry.version);
            println!("Text:");
        }
        println!("{}", entry.text);
    } else {
        if !quiet {
            println!(
                "Found {} entries matching '{}':",
                matches.len(),
                query_lower
            );
            println!();
        }
        for entry in &matches {
            // Truncate long text for display
            let text_preview = if entry.text.len() > 80 {
                format!("{}...", &entry.text[..77])
            } else {
                entry.text.clone()
            };
            let text_preview = text_preview.replace('\n', "\\n");

            println!("{}", entry.key);
            if !quiet {
                println!("  {text_preview}");
            }
        }
    }

    Ok(())
}

/// Search entries by text content
fn search_by_text(
    resource: &crate::formats::loca::LocaResource,
    query_lower: &str,
    limit: usize,
    quiet: bool,
) -> anyhow::Result<()> {
    let matches: Vec<_> = resource
        .entries
        .iter()
        .filter(|e| e.text.to_lowercase().contains(query_lower))
        .take(limit)
        .collect();

    if matches.is_empty() {
        if !quiet {
            println!("No entries found containing '{query_lower}'");
        }
    } else {
        if !quiet {
            println!(
                "Found {} entries containing '{}':",
                matches.len(),
                query_lower
            );
            println!();
        }
        for entry in &matches {
            // Truncate long text for display
            let text_preview = if entry.text.len() > 100 {
                format!("{}...", &entry.text[..97])
            } else {
                entry.text.clone()
            };
            let text_preview = text_preview.replace('\n', "\\n");

            println!("{}", entry.key);
            if !quiet {
                println!("  {text_preview}");
            }
        }
    }

    Ok(())
}

/// Get a specific entry by key
pub fn get(path: &Path, key: &str) -> anyhow::Result<()> {
    let resource = read_loca(path)?;

    if let Some(entry) = resource.get_entry(key) {
        println!("Key: {}", entry.key);
        println!("Version: {}", entry.version);
        println!("Text:");
        println!("{}", entry.text);
    } else {
        anyhow::bail!("Entry not found: {}", key);
    }

    Ok(())
}

/// Add or update an entry
pub fn set(path: &Path, key: &str, text: &str, create: bool) -> anyhow::Result<()> {
    let mut resource = if path.exists() {
        read_loca(path)?
    } else if create {
        LocaResource::new()
    } else {
        anyhow::bail!(
            "File does not exist: {}. Use --create to create a new file.",
            path.display()
        );
    };

    let is_new = resource.add_entry(key, text);

    write_loca(path, &resource)?;

    if is_new {
        println!("Added entry: {key}");
    } else {
        println!("Updated entry: {key}");
    }

    Ok(())
}

/// Delete an entry
pub fn delete(path: &Path, key: &str) -> anyhow::Result<()> {
    let mut resource = read_loca(path)?;

    if let Some(entry) = resource.delete_entry(key) {
        write_loca(path, &resource)?;
        println!("Deleted entry: {}", entry.key);
        println!("  Previous text: {}", truncate_text(&entry.text, 80));
    } else {
        anyhow::bail!("Entry not found: {}", key);
    }

    Ok(())
}

/// Find and replace text
pub fn replace(
    path: &Path,
    find: &str,
    replace_with: &str,
    case_sensitive: bool,
    key_pattern: Option<&str>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let mut resource = read_loca(path)?;

    let result = if let Some(pattern) = key_pattern {
        resource.replace_in_matching(pattern, find, replace_with, case_sensitive)
    } else {
        resource.replace_all(find, replace_with, case_sensitive)
    };

    if result.entries_modified == 0 {
        println!("No matches found for '{}'", find);
        return Ok(());
    }

    println!(
        "Found {} replacements in {} entries:",
        result.replacements, result.entries_modified
    );

    for key in &result.modified_keys {
        if let Some(entry) = resource.get_entry(key) {
            println!("  {} -> {}", key, truncate_text(&entry.text, 60));
        }
    }

    if dry_run {
        println!("\n(Dry run - no changes made)");
    } else {
        write_loca(path, &resource)?;
        println!("\nChanges saved to {}", path.display());
    }

    Ok(())
}

/// Export LOCA to translation file
pub fn export(path: &Path, output: Option<&Path>, format: &str) -> anyhow::Result<()> {
    let resource = read_loca(path)?;

    let export_format = match format {
        "csv" => ExportFormat::Csv,
        _ => ExportFormat::Tsv,
    };

    let output_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| path.with_extension(export_format.extension()));

    let count = export_for_translation(&resource, &output_path, export_format)?;

    println!("Exported {} entries to {}", count, output_path.display());
    println!("\nThe file has 4 columns:");
    println!("  1. Key (handle)");
    println!("  2. Version");
    println!("  3. Original text");
    println!("  4. Translation (fill this in)");

    Ok(())
}

/// Import translations from file
pub fn import(
    path: &Path,
    translations: &Path,
    format: &str,
    backup: bool,
) -> anyhow::Result<()> {
    let mut resource = read_loca(path)?;

    let import_format = match format {
        "csv" => ExportFormat::Csv,
        _ => ExportFormat::Tsv,
    };

    // Create backup if requested
    if backup {
        let backup_path = path.with_extension("loca.bak");
        std::fs::copy(path, &backup_path)?;
        println!("Created backup: {}", backup_path.display());
    }

    let result = import_translations(&mut resource, translations, import_format)?;

    if result.translated == 0 {
        println!("No translations found in {}", translations.display());
        return Ok(());
    }

    write_loca(path, &resource)?;

    println!("Import complete:");
    println!("  Translated: {}", result.translated);
    println!("  Skipped (empty): {}", result.skipped);

    if result.not_found > 0 {
        println!("  Not found: {}", result.not_found);
        if result.missing_keys.len() <= 5 {
            for key in &result.missing_keys {
                println!("    - {}", key);
            }
        } else {
            println!("    (showing first 5)");
            for key in result.missing_keys.iter().take(5) {
                println!("    - {}", key);
            }
        }
    }

    Ok(())
}

/// Show LOCA file statistics
pub fn stats(paths: &[std::path::PathBuf]) -> anyhow::Result<()> {
    let mut total_entries = 0;
    let mut total_chars = 0;

    for path in paths {
        let resource = read_loca(path)?;
        let entries = resource.len();
        let chars: usize = resource.entries.iter().map(|e| e.text.len()).sum();
        let avg_len = if entries > 0 { chars / entries } else { 0 };

        println!("{}:", path.display());
        println!("  Entries: {}", entries);
        println!("  Total characters: {}", chars);
        println!("  Average length: {} chars", avg_len);

        // Find longest and shortest
        if let Some(longest) = resource.entries.iter().max_by_key(|e| e.text.len()) {
            println!(
                "  Longest: {} ({} chars)",
                longest.key,
                longest.text.len()
            );
        }

        if !resource.entries.is_empty() {
            if let Some(shortest) = resource
                .entries
                .iter()
                .filter(|e| !e.text.is_empty())
                .min_by_key(|e| e.text.len())
            {
                println!(
                    "  Shortest (non-empty): {} ({} chars)",
                    shortest.key,
                    shortest.text.len()
                );
            }
        }

        let empty_count = resource.entries.iter().filter(|e| e.text.is_empty()).count();
        if empty_count > 0 {
            println!("  Empty entries: {}", empty_count);
        }

        println!();

        total_entries += entries;
        total_chars += chars;
    }

    if paths.len() > 1 {
        println!("Total across {} files:", paths.len());
        println!("  Entries: {}", total_entries);
        println!("  Characters: {}", total_chars);
    }

    Ok(())
}

/// Truncate text for display
fn truncate_text(text: &str, max_len: usize) -> String {
    let text = text.replace('\n', "\\n");
    if text.len() > max_len {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    } else {
        text
    }
}
