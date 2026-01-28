//! CLI command for listing PAK contents

use std::path::Path;

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

/// Format byte size for human-readable output
fn format_size(bytes: u32) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{bytes}")
    }
}

pub fn execute(
    source: &Path,
    detailed: bool,
    filter: Option<&str>,
    count: bool,
) -> anyhow::Result<()> {
    use crate::pak::PakOperations;

    if detailed {
        // Get detailed file entries
        let entries = PakOperations::list_detailed(source)?;

        // Filter if pattern provided
        let filtered: Vec<_> = if let Some(pattern) = filter {
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
        } else {
            entries.iter().collect()
        };

        if count {
            println!("{}", filtered.len());
            return Ok(());
        }

        // Print header
        println!(
            "{:>10}  {:>10}  {:>6}  PATH",
            "SIZE", "COMPRESSED", "RATIO"
        );

        // Print entries
        for entry in &filtered {
            let ratio = if entry.size_decompressed > 0 {
                (entry.size_compressed as f64 / entry.size_decompressed as f64) * 100.0
            } else {
                100.0
            };

            println!(
                "{:>10}  {:>10}  {:>5.1}%  {}",
                format_size(entry.size_decompressed),
                format_size(entry.size_compressed),
                ratio,
                entry.path.display()
            );
        }

        // Print summary
        let total_decompressed: u64 = filtered.iter().map(|e| e.size_decompressed as u64).sum();
        let total_compressed: u64 = filtered.iter().map(|e| e.size_compressed as u64).sum();
        let overall_ratio = if total_decompressed > 0 {
            (total_compressed as f64 / total_decompressed as f64) * 100.0
        } else {
            100.0
        };

        println!();
        println!(
            "{} files, {} total ({} compressed, {:.1}% ratio)",
            filtered.len(),
            format_size(total_decompressed as u32),
            format_size(total_compressed as u32),
            overall_ratio
        );
    } else {
        // Simple listing (paths only)
        let files = PakOperations::list(source)?;

        // Filter if pattern provided
        let filtered: Vec<_> = if let Some(pattern) = filter {
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
