//! CLI commands for searching PAK contents

use std::path::Path;
use std::time::Instant;

use crate::cli::progress::{print_step, print_done, simple_spinner, LOOKING_GLASS, LINK, DISK, SPARKLE};
use crate::search::{FileType, SearchIndex};

/// Build a search index from a PAK file
pub fn build_index(pak: &Path, output: Option<&Path>, fulltext: bool) -> anyhow::Result<()> {
    let started = Instant::now();
    let total_steps = if fulltext { 3 } else { 2 } + if output.is_some() { 1 } else { 0 };
    let mut step = 1;

    print_step(step, total_steps, LOOKING_GLASS, &format!("Reading {}...", pak.display()));
    step += 1;

    let mut index = SearchIndex::new();
    index.build_index(&[pak.to_path_buf()])?;

    print_step(step, total_steps, LINK, &format!("Indexed {} files", index.file_count()));
    step += 1;

    if fulltext {
        print_step(step, total_steps, LINK, "Building full-text index...");
        step += 1;

        let pb = simple_spinner("Indexing content...");
        let doc_count = index.build_fulltext_index(&|progress| {
            let msg = progress.current_file.as_deref().unwrap_or(progress.phase.as_str());
            pb.set_message(format!("{msg}: {}/{}", progress.current, progress.total));
        })?;
        pb.finish_and_clear();

        println!("  {} Indexed {} documents for full-text search", SPARKLE, doc_count);
    }

    if let Some(out) = output {
        print_step(step, total_steps, DISK, &format!("Exporting to {}...", out.display()));

        let pb = simple_spinner("Exporting...");
        index.export_index_with_progress(out, &|progress| {
            let msg = progress.current_file.as_deref().unwrap_or(progress.phase.as_str());
            pb.set_message(format!("{msg}: {}/{}", progress.current, progress.total));
        })?;
        pb.finish_and_clear();
    }

    print_done(started.elapsed());
    Ok(())
}

/// Search for files by filename
pub fn search_filename(
    pak: &Path,
    query: &str,
    type_filter: Option<&str>,
) -> anyhow::Result<()> {
    let mut index = SearchIndex::new();

    let pb = simple_spinner("Building index...");
    index.build_index(&[pak.to_path_buf()])?;
    pb.finish_and_clear();

    let filter = type_filter.and_then(parse_file_type);
    let results = index.search_filename(query, filter);

    if results.is_empty() {
        println!("No files found matching '{query}'");
    } else {
        println!("Found {} files matching '{query}':", results.len());
        for file in results {
            println!(
                "  {} ({}) - {} bytes",
                file.path,
                file.file_type.display_name(),
                file.size
            );
        }
    }

    Ok(())
}

/// Search for files by path
pub fn search_path(pak: &Path, query: &str, type_filter: Option<&str>) -> anyhow::Result<()> {
    let mut index = SearchIndex::new();

    let pb = simple_spinner("Building index...");
    index.build_index(&[pak.to_path_buf()])?;
    pb.finish_and_clear();

    let filter = type_filter.and_then(parse_file_type);
    let results = index.search_path(query, filter);

    if results.is_empty() {
        println!("No files found with path containing '{query}'");
    } else {
        println!("Found {} files with path containing '{query}':", results.len());
        for file in results {
            println!(
                "  {} ({}) - {} bytes",
                file.path,
                file.file_type.display_name(),
                file.size
            );
        }
    }

    Ok(())
}

/// Search for files by UUID
pub fn search_uuid(pak: &Path, uuid: &str) -> anyhow::Result<()> {
    let mut index = SearchIndex::new();

    let pb = simple_spinner("Building index...");
    index.build_index(&[pak.to_path_buf()])?;
    pb.finish_and_clear();

    let results = index.search_uuid(uuid);

    if results.is_empty() {
        println!("No files found matching UUID '{uuid}'");
    } else {
        println!("Found {} files matching UUID '{uuid}':", results.len());
        for file in results {
            println!(
                "  {} ({}) - {} bytes",
                file.path,
                file.file_type.display_name(),
                file.size
            );
        }
    }

    Ok(())
}

/// Full-text content search
pub fn search_content(pak: &Path, query: &str, limit: usize) -> anyhow::Result<()> {
    let started = std::time::Instant::now();
    let mut index = SearchIndex::new();

    print_step(1, 2, LOOKING_GLASS, &format!("Reading {}...", pak.display()));
    let pb = simple_spinner("Building index...");
    index.build_index(&[pak.to_path_buf()])?;
    pb.finish_and_clear();

    print_step(2, 2, LINK, "Building full-text index...");
    let pb = simple_spinner("Indexing content...");
    index.build_fulltext_index(&|progress| {
        let msg = progress.current_file.as_deref().unwrap_or(progress.phase.as_str());
        pb.set_message(format!("{msg}: {}/{}", progress.current, progress.total));
    })?;
    pb.finish_and_clear();

    print_done(started.elapsed());

    if let Some(results) = index.search_fulltext(query, limit) {
        if results.is_empty() {
            println!("No content found matching '{query}'");
        } else {
            println!("Found {} results for '{query}':", results.len());
            for (i, result) in results.iter().enumerate() {
                println!(
                    "  {}. {} (score: {:.2})",
                    i + 1,
                    result.path,
                    result.score
                );
                if let Some(snippet) = &result.snippet {
                    // Show a truncated snippet
                    let clean_snippet = snippet.replace('\n', " ");
                    let truncated = if clean_snippet.len() > 100 {
                        format!("{}...", &clean_snippet[..100])
                    } else {
                        clean_snippet
                    };
                    println!("     {truncated}");
                }
            }
        }
    } else {
        println!("Full-text search not available");
    }

    Ok(())
}

/// Import and search a pre-built index
pub fn search_index(index_dir: &Path, query: &str, limit: usize) -> anyhow::Result<()> {
    let mut index = SearchIndex::new();
    index.import_index(index_dir)?;

    println!(
        "Loaded index: {} files, {} docs",
        index.file_count(),
        index.fulltext_doc_count()
    );

    if let Some(results) = index.search_fulltext(query, limit) {
        if results.is_empty() {
            println!("No content found matching '{query}'");
        } else {
            println!("Found {} results for '{query}':", results.len());
            for (i, result) in results.iter().enumerate() {
                println!(
                    "  {}. {} (score: {:.2})",
                    i + 1,
                    result.path,
                    result.score
                );
            }
        }
    } else {
        println!("Full-text search not available in this index");
    }

    Ok(())
}

/// Show index statistics
pub fn index_stats(index_dir: &Path) -> anyhow::Result<()> {
    let mut index = SearchIndex::new();
    index.import_index(index_dir)?;

    println!("Index Statistics:");
    println!("  Files indexed: {}", index.file_count());
    println!("  PAKs indexed: {}", index.pak_count());
    println!("  Full-text docs: {}", index.fulltext_doc_count());
    println!("  Has full-text: {}", index.has_fulltext());
    println!();
    println!("Indexed PAKs:");
    for pak in index.indexed_paks() {
        println!("  {}", pak.display());
    }

    Ok(())
}

/// Parse file type from string
fn parse_file_type(s: &str) -> Option<FileType> {
    match s.to_lowercase().as_str() {
        "lsx" => Some(FileType::Lsx),
        "lsf" => Some(FileType::Lsf),
        "lsj" => Some(FileType::Lsj),
        "lsbc" => Some(FileType::Lsbc),
        "xml" => Some(FileType::Xml),
        "json" => Some(FileType::Json),
        "dds" => Some(FileType::Dds),
        "png" | "image" => Some(FileType::Png),
        "gr2" => Some(FileType::Gr2),
        "wem" | "audio" => Some(FileType::Wem),
        "gts" => Some(FileType::Gts),
        "gtp" => Some(FileType::Gtp),
        _ => None,
    }
}
