//! CLI command for PAK extraction

use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

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
            // Try matching zero or more characters
            for i in ti..=text.len() {
                if matches_glob_recursive(pattern, text, pi + 1, i) {
                    return true;
                }
            }
            false
        }
        '?' => {
            // Match exactly one character
            if ti < text.len() {
                matches_glob_recursive(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
        c => {
            // Match literal character (case-insensitive for paths)
            if ti < text.len() && text[ti].to_ascii_lowercase() == c.to_ascii_lowercase() {
                matches_glob_recursive(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}

/// Create a progress bar with consistent styling
fn create_progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {wide_msg}")
            .expect("valid template")
            .progress_chars("█▓░"),
    );
    pb.set_message(message.to_string());
    pb
}

pub fn execute(
    source: &Path,
    destination: &Path,
    filter: Option<&str>,
    file: Option<&str>,
    progress: bool,
) -> anyhow::Result<()> {
    use crate::pak::PakOperations;

    // Single file extraction
    if let Some(file_path) = file {
        println!("Extracting single file: {file_path}");
        PakOperations::extract_files(source, destination, &[file_path])?;
        println!("Extraction complete");
        return Ok(());
    }

    // Filtered extraction
    if let Some(pattern) = filter {
        println!("Extracting files matching: {pattern}");

        // List all files and filter
        let all_files = PakOperations::list(source)?;
        let matching: Vec<&str> = all_files
            .iter()
            .filter(|f| {
                // Match against filename or full path
                let filename = Path::new(f)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(f);
                matches_glob(pattern, filename) || matches_glob(pattern, f)
            })
            .map(String::as_str)
            .collect();

        if matching.is_empty() {
            println!("No files match pattern: {pattern}");
            return Ok(());
        }

        println!("Found {} matching files", matching.len());

        if progress {
            let pb = create_progress_bar(matching.len() as u64, "Extracting");
            let count = AtomicUsize::new(0);

            PakOperations::extract_files_with_progress(
                source,
                destination,
                &matching,
                &|_current, _total, name| {
                    let n = count.fetch_add(1, Ordering::SeqCst) + 1;
                    pb.set_position(n as u64);
                    // Show just the filename, not full path
                    let short_name = Path::new(name)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(name);
                    pb.set_message(short_name.to_string());
                },
            )?;

            pb.finish_with_message("done");
        } else {
            PakOperations::extract_files(source, destination, &matching)?;
        }

        println!("Extraction complete");
        return Ok(());
    }

    // Full extraction
    if progress {
        // First, get the file count for the progress bar
        let files = PakOperations::list(source)?;
        let total = files.len() as u64;

        println!("Extracting {} files from {:?}", total, source);

        let pb = create_progress_bar(total, "Extracting");
        let count = AtomicUsize::new(0);

        PakOperations::extract_with_progress(source, destination, &|_current, _total, name| {
            let n = count.fetch_add(1, Ordering::SeqCst) + 1;
            pb.set_position(n as u64);
            let short_name = Path::new(name)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(name);
            pb.set_message(short_name.to_string());
        })?;

        pb.finish_with_message("done");
    } else {
        println!("Extracting {:?} to {:?}", source, destination);
        PakOperations::extract(source, destination)?;
    }

    println!("Extraction complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_matching() {
        // Wildcard tests
        assert!(matches_glob("*.lsf", "test.lsf"));
        assert!(matches_glob("*.lsf", "path/to/test.lsf"));
        assert!(!matches_glob("*.lsf", "test.lsx"));

        // Pattern with multiple wildcards
        assert!(matches_glob("*_merged.lsf", "Public/Shared/_merged.lsf"));
        assert!(matches_glob("*_merged*", "something_merged.lsf"));

        // Question mark
        assert!(matches_glob("test?.lsf", "test1.lsf"));
        assert!(!matches_glob("test?.lsf", "test12.lsf"));

        // Case insensitivity
        assert!(matches_glob("*.LSF", "test.lsf"));
        assert!(matches_glob("*.lsf", "TEST.LSF"));
    }
}
