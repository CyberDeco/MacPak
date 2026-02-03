//! CLI commands for diff and merge operations

use std::path::Path;

use crate::diff::{self, DiffOptions, MergeOptions};

/// Compare two files
pub fn compare(
    old: &Path,
    new: &Path,
    ignore_whitespace: bool,
    ignore_version: bool,
    match_by_key: bool,
    format: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let options = DiffOptions {
        ignore_whitespace,
        ignore_version,
        match_by_key,
    };

    let result = diff::diff_files(old, new, &options)?;

    if format == "json" {
        // Output as JSON
        let output = serde_json::json!({
            "old": old.display().to_string(),
            "new": new.display().to_string(),
            "identical": result.is_empty(),
            "change_count": result.change_count(),
            "regions_changed": result.regions_changed(),
            "changes": result.changes.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        // Text output
        if result.is_empty() {
            if !quiet {
                println!("Files are identical");
            }
        } else {
            for change in &result.changes {
                println!("{change}");
            }
            if !quiet {
                println!();
                println!("{}", result.summary());
            }
        }
    }

    Ok(())
}

/// Three-way merge
pub fn merge(
    base: &Path,
    ours: &Path,
    theirs: &Path,
    output: &Path,
    prefer_ours: bool,
    prefer_theirs: bool,
    match_by_key: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    let options = MergeOptions {
        diff_options: DiffOptions {
            match_by_key,
            ..Default::default()
        },
        prefer_ours,
        prefer_theirs,
    };

    let result = diff::merge_files(base, ours, theirs, &options)?;

    if result.has_conflicts() {
        println!("Merge completed with {} conflict(s):", result.conflicts.len());
        println!();
        for conflict in &result.conflicts {
            println!("  {conflict}");
        }
        println!();
        println!(
            "Applied: {} from ours, {} from theirs",
            result.ours_applied, result.theirs_applied
        );

        // Still write the output (with conflicts resolved to ours by default)
        result.write(output)?;
        println!();
        println!("Wrote merged result to: {}", output.display());
        println!("Note: Conflicts were auto-resolved (check the output)");
    } else {
        result.write(output)?;
        if !quiet {
            println!(
                "Merge successful: {} from ours, {} from theirs",
                result.ours_applied, result.theirs_applied
            );
            println!("Wrote: {}", output.display());
        }
    }

    Ok(())
}
