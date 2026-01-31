//! CLI commands for LOCA localization file operations

use std::path::Path;

use crate::formats::loca::read_loca;

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
