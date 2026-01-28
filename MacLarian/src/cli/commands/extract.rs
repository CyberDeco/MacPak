//! CLI command for PAK extraction

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::cli::progress::simple_bar;

/// GR2 processing options from CLI flags
#[derive(Debug, Clone, Default)]
pub struct Gr2CliOptions {
    /// Enable all GR2 processing (--bundle)
    pub bundle: bool,
    /// Convert GR2 to GLB (--convert-gr2)
    pub convert_gr2: bool,
    /// Extract DDS textures (--extract-textures)
    pub extract_textures: bool,
    /// Extract virtual textures (--extract-virtual-textures)
    pub extract_virtual_textures: bool,
    /// Path to BG3 install folder (--bg3-path)
    pub game_data: Option<PathBuf>,
    /// Path to virtual textures folder (--virtual-textures)
    pub virtual_textures: Option<PathBuf>,
    /// Delete original GR2 after conversion (--delete-gr2)
    pub delete_gr2: bool,
    /// Convert extracted DDS textures to PNG (--png)
    pub convert_textures_to_png: bool,
    /// Keep original DDS files after PNG conversion (--keep-dds)
    pub keep_original_dds: bool,
}

impl Gr2CliOptions {
    /// Check if any GR2 processing is enabled
    fn has_processing(&self) -> bool {
        self.bundle || self.convert_gr2 || self.extract_textures || self.extract_virtual_textures
    }

    /// Convert to library extraction options
    fn to_extraction_options(&self) -> crate::pak::Gr2ExtractionOptions {
        use crate::pak::Gr2ExtractionOptions;

        if self.bundle {
            // Bundle mode enables everything
            Gr2ExtractionOptions::bundle()
                .with_game_data_path(self.game_data.clone())
                .with_virtual_textures_path(self.virtual_textures.clone())
                .with_keep_original(!self.delete_gr2)
                .with_convert_to_png(self.convert_textures_to_png)
                .with_keep_original_dds(self.keep_original_dds)
        } else {
            Gr2ExtractionOptions::new()
                .with_convert_to_glb(self.convert_gr2)
                .with_extract_textures(self.extract_textures)
                .with_extract_virtual_textures(self.extract_virtual_textures)
                .with_game_data_path(self.game_data.clone())
                .with_virtual_textures_path(self.virtual_textures.clone())
                .with_keep_original(!self.delete_gr2)
                .with_convert_to_png(self.convert_textures_to_png)
                .with_keep_original_dds(self.keep_original_dds)
        }
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
            if ti < text.len() && text[ti].eq_ignore_ascii_case(&c) {
                matches_glob_recursive(pattern, text, pi + 1, ti + 1)
            } else {
                false
            }
        }
    }
}


pub fn execute(
    source: &Path,
    destination: &Path,
    filter: Option<&str>,
    file: Option<&str>,
    progress: bool,
    gr2_options: &Gr2CliOptions,
) -> anyhow::Result<()> {
    use crate::pak::{PakOperations, extract_files_smart};

    // Checks if smart extraction is needed (GR2 processing enabled)
    let use_smart_extract = gr2_options.has_processing();

    // Single file extraction
    if let Some(file_path) = file {
        println!("Extracting single file: {file_path}");

        if use_smart_extract {
            let extraction_opts = gr2_options.to_extraction_options();
            let result = extract_files_smart(
                source,
                destination,
                &[file_path],
                extraction_opts,
                &|_progress| {},
            )?;
            print_smart_extraction_result(&result);
        } else {
            PakOperations::extract_files(source, destination, &[file_path])?;
            println!("Extraction complete");
        }
        return Ok(());
    }

    // Filtered extraction
    if let Some(pattern) = filter {
        println!("Extracting files matching: {pattern}");

        // List all files and filter
        let all_files = PakOperations::list(source)?;
        let matching: Vec<String> = all_files
            .iter()
            .filter(|f| {
                // Match against filename or full path
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

        if use_smart_extract {
            execute_smart_extraction(source, destination, &matching, gr2_options, progress)?;
        } else if progress {
            let pb = simple_bar(matching.len() as u64, "Extracting");
            let count = AtomicUsize::new(0);

            let matching_refs: Vec<&str> = matching.iter().map(String::as_str).collect();
            PakOperations::extract_files_with_progress(
                source,
                destination,
                &matching_refs,
                &|progress| {
                    let n = count.fetch_add(1, Ordering::SeqCst) + 1;
                    pb.set_position(n as u64);
                    if let Some(name) = &progress.current_file {
                        let short_name = Path::new(name)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(name);
                        pb.set_message(short_name.to_string());
                    }
                },
            )?;

            pb.finish_with_message("done");
            println!("Extraction complete");
        } else {
            let matching_refs: Vec<&str> = matching.iter().map(String::as_str).collect();
            PakOperations::extract_files(source, destination, &matching_refs)?;
            println!("Extraction complete");
        }

        return Ok(());
    }

    // Full extraction
    if use_smart_extract {
        // List all files for smart extraction
        let all_files = PakOperations::list(source)?;
        execute_smart_extraction(source, destination, &all_files, gr2_options, progress)?;
    } else if progress {
        let files = PakOperations::list(source)?;
        let total = files.len() as u64;

        println!("Extracting {total} files from {}", source.display());

        let pb = simple_bar(total, "Extracting");
        let count = AtomicUsize::new(0);

        PakOperations::extract_with_progress(source, destination, &|progress| {
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
        println!("Extraction complete");
    } else {
        println!("Extracting {} to {}", source.display(), destination.display());
        PakOperations::extract(source, destination)?;
        println!("Extraction complete");
    }

    Ok(())
}

/// Execute smart extraction with GR2 processing
fn execute_smart_extraction(
    source: &Path,
    destination: &Path,
    files: &[String],
    gr2_options: &Gr2CliOptions,
    progress: bool,
) -> anyhow::Result<()> {
    use crate::pak::extract_files_smart;

    let extraction_opts = gr2_options.to_extraction_options();

    // Count GR2 files for progress reporting
    let gr2_count = files.iter().filter(|f| f.to_lowercase().ends_with(".gr2")).count();
    if gr2_count > 0 {
        println!("Found {gr2_count} GR2 files to process");
    }

    if progress {
        let total = files.len();
        let pb = simple_bar(total as u64, "Extracting");
        let count = AtomicUsize::new(0);

        let result = extract_files_smart(
            source,
            destination,
            files,
            extraction_opts,
            &|progress| {
                let n = count.fetch_add(1, Ordering::SeqCst) + 1;
                pb.set_position(n as u64);
                if let Some(ref name) = progress.current_file {
                    let short_name = Path::new(name)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(name);
                    pb.set_message(short_name.to_string());
                }
            },
        )?;

        pb.finish_with_message("done");
        print_smart_extraction_result(&result);
    } else {
        let result = extract_files_smart(
            source,
            destination,
            files,
            extraction_opts,
            &|_progress| {},
        )?;
        print_smart_extraction_result(&result);
    }

    Ok(())
}

/// Print summary of smart extraction results
fn print_smart_extraction_result(result: &crate::pak::SmartExtractionResult) {
    println!("Extraction complete:");
    println!("  Files extracted: {}", result.files_extracted);

    if result.gr2s_processed > 0 {
        println!("  GR2 files processed: {}", result.gr2s_processed);
        println!("  GLB files created: {}", result.glb_files_created);
        println!("  Textures extracted: {}", result.textures_extracted);

        if !result.gr2_folders.is_empty() {
            println!("\nGR2 bundles created:");
            for folder in &result.gr2_folders {
                println!("  {}", folder.display());
            }
        }
    }

    if !result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &result.warnings {
            println!("  {warning}");
        }
    }
}

