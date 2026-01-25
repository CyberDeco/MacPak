//! CLI commands for LOCA localization file operations

use std::path::Path;

use crate::formats::loca::read_loca;

/// List all entries in a LOCA file
pub fn list(path: &Path, limit: Option<usize>) -> anyhow::Result<()> {
    let resource = read_loca(path)?;

    println!("LOCA file: {}", path.display());
    println!("Total entries: {}", resource.entries.len());
    println!();

    let display_limit = limit.unwrap_or(resource.entries.len());
    for (i, entry) in resource.entries.iter().take(display_limit).enumerate() {
        // Truncate long text for display
        let text_preview = if entry.text.len() > 80 {
            format!("{}...", &entry.text[..77])
        } else {
            entry.text.clone()
        };
        let text_preview = text_preview.replace('\n', "\\n");

        println!("{:>5}. {} (v{})", i + 1, entry.key, entry.version);
        println!("       {text_preview}");
    }

    if display_limit < resource.entries.len() {
        println!();
        println!(
            "... and {} more entries (use --limit to see more)",
            resource.entries.len() - display_limit
        );
    }

    Ok(())
}

/// Get a specific entry by handle
pub fn get(path: &Path, handle: &str) -> anyhow::Result<()> {
    let resource = read_loca(path)?;

    // Try to find by exact key match first
    if let Some(entry) = resource.entries.iter().find(|e| e.key == handle) {
        println!("Key: {}", entry.key);
        println!("Version: {}", entry.version);
        println!("Text:");
        println!("{}", entry.text);
        return Ok(());
    }

    // Try partial match
    let matches: Vec<_> = resource
        .entries
        .iter()
        .filter(|e| e.key.contains(handle))
        .collect();

    if matches.is_empty() {
        println!("No entry found matching '{handle}'");
    } else if matches.len() == 1 {
        let entry = matches[0];
        println!("Key: {}", entry.key);
        println!("Version: {}", entry.version);
        println!("Text:");
        println!("{}", entry.text);
    } else {
        println!("Multiple entries match '{handle}':");
        for entry in matches.iter().take(10) {
            println!("  {}", entry.key);
        }
        if matches.len() > 10 {
            println!("  ... and {} more", matches.len() - 10);
        }
    }

    Ok(())
}

/// Search for entries containing text
pub fn search(path: &Path, query: &str, limit: usize) -> anyhow::Result<()> {
    let resource = read_loca(path)?;
    let query_lower = query.to_lowercase();

    let matches: Vec<_> = resource
        .entries
        .iter()
        .filter(|e| e.text.to_lowercase().contains(&query_lower))
        .take(limit)
        .collect();

    if matches.is_empty() {
        println!("No entries found containing '{query}'");
    } else {
        println!("Found {} entries containing '{query}':", matches.len());
        println!();
        for entry in &matches {
            // Truncate long text for display
            let text_preview = if entry.text.len() > 100 {
                format!("{}...", &entry.text[..97])
            } else {
                entry.text.clone()
            };
            let text_preview = text_preview.replace('\n', "\\n");

            println!("{}", entry.key);
            println!("  {text_preview}");
        }
    }

    Ok(())
}

/// Export LOCA to XML format
pub fn export_xml(path: &Path, output: &Path) -> anyhow::Result<()> {
    use crate::converter::loca::convert_loca_to_xml;

    convert_loca_to_xml(path, output)?;
    println!("Exported to: {}", output.display());
    Ok(())
}
