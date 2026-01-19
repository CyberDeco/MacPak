//! PAK archive operations

use crate::error::{Error, Result};
use super::lspk::{CompressionMethod, FileTableEntry, LspkReader, LspkWriter};
use rayon::prelude::*;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Progress callback for PAK operations
/// Arguments: (current, total, description)
/// Must be Sync + Send to support parallel decompression.
pub type ProgressCallback<'a> = &'a (dyn Fn(usize, usize, &str) + Sync + Send);

pub struct PakOperations;

impl PakOperations {
    /// Extract a PAK file to a directory
    ///
    /// # Errors
    /// Returns an error if the PAK file cannot be read or extraction fails.
    pub fn extract<P: AsRef<Path>>(pak_path: P, output_dir: P) -> Result<()> {
        Self::extract_with_progress(pak_path, output_dir, &|_, _, _| {})
    }

    /// Extract a PAK file to a directory with progress callback
    ///
    /// The callback receives (current, total, filename) during file extraction.
    /// Uses parallel decompression for improved performance on multi-core systems.
    ///
    /// # Errors
    /// Returns an error if the PAK file cannot be read or extraction fails.
    pub fn extract_with_progress<P: AsRef<Path>>(
        pak_path: P,
        output_dir: P,
        progress: ProgressCallback,
    ) -> Result<()> {
        let pak_path = pak_path.as_ref();
        let output_dir = output_dir.as_ref();

        let mut file = File::open(pak_path)?;
        let mut reader = LspkReader::with_path(File::open(pak_path)?, pak_path);

        // Get file list without decompressing
        let entries = reader.list_files()?;
        let total_files = entries.len();

        std::fs::create_dir_all(output_dir)?;

        progress(0, total_files, "Reading compressed data...");

        // Phase 1: Read all compressed data sequentially (PAK is a single file)
        let compressed_files: Vec<CompressedFile> = entries
            .iter()
            .filter(|entry| {
                // Skip .DS_Store files
                entry.path.file_name() != Some(std::ffi::OsStr::new(".DS_Store"))
            })
            .filter_map(|entry| {
                // Seek and read compressed data
                if file.seek(SeekFrom::Start(entry.offset)).is_err() {
                    tracing::warn!("Failed to seek to {}", entry.path.display());
                    return None;
                }

                let mut compressed_data = vec![0u8; entry.size_compressed as usize];
                if file.read_exact(&mut compressed_data).is_err() {
                    tracing::warn!("Failed to read {}", entry.path.display());
                    return None;
                }

                Some(CompressedFile {
                    entry: entry.clone(),
                    compressed_data,
                })
            })
            .collect();

        let files_to_process = compressed_files.len();
        let processed = AtomicUsize::new(0);
        let error_count = AtomicUsize::new(0);

        // Phase 2: Decompress and write in parallel
        let errors: Vec<(PathBuf, String)> = compressed_files
            .par_iter()
            .filter_map(|cf| {
                let file_name = cf
                    .entry
                    .path
                    .file_name().map_or_else(|| cf.entry.path.to_string_lossy().to_string(), |n| n.to_string_lossy().to_string());

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                progress(current, files_to_process, &file_name);

                // Decompress
                let data = match decompress_data(
                    &cf.compressed_data,
                    cf.entry.compression,
                    cf.entry.size_decompressed,
                ) {
                    Ok(data) => data,
                    Err(e) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        tracing::warn!("Failed to decompress {}: {}", cf.entry.path.display(), e);
                        return Some((cf.entry.path.clone(), e.to_string()));
                    }
                };

                // Calculate output path (handle virtual texture subfolders)
                let output_path = if is_virtual_texture_file(&file_name) {
                    if let Some(subfolder) = get_virtual_texture_subfolder(&file_name) {
                        if let Some(parent) = cf.entry.path.parent() {
                            output_dir.join(parent).join(&subfolder).join(&file_name)
                        } else {
                            output_dir.join(&subfolder).join(&file_name)
                        }
                    } else {
                        output_dir.join(&cf.entry.path)
                    }
                } else {
                    output_dir.join(&cf.entry.path)
                };

                // Create parent directories (idempotent)
                if let Some(parent) = output_path.parent()
                    && let Err(e) = std::fs::create_dir_all(parent) {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        return Some((cf.entry.path.clone(), format!("Failed to create dir: {e}")));
                    }

                // Write file
                if let Err(e) = std::fs::write(&output_path, &data) {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    return Some((cf.entry.path.clone(), format!("Failed to write: {e}")));
                }

                None
            })
            .collect();

        // If there were errors, return a summary error
        if !errors.is_empty() {
            return Err(Error::ConversionError(format!(
                "Extracted {} files with {} errors. First error: {}",
                files_to_process - errors.len(),
                errors.len(),
                errors[0].1
            )));
        }

        Ok(())
    }

    /// Create a PAK file from a directory
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read or PAK creation fails.
    pub fn create<P: AsRef<Path>>(source_dir: P, output_pak: P) -> Result<()> {
        Self::create_with_progress(source_dir, output_pak, &|_, _, _| {})
    }

    /// Create a PAK file from a directory with progress callback
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read or PAK creation fails.
    pub fn create_with_progress<P: AsRef<Path>>(
        source_dir: P,
        output_pak: P,
        progress: ProgressCallback,
    ) -> Result<()> {
        let writer = LspkWriter::new(source_dir.as_ref())?;
        writer.write_with_progress(output_pak.as_ref(), progress)?;
        Ok(())
    }

    /// Create a PAK file from a directory with specified compression
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read or PAK creation fails.
    pub fn create_with_compression<P: AsRef<Path>>(
        source_dir: P,
        output_pak: P,
        compression: CompressionMethod,
    ) -> Result<()> {
        Self::create_with_compression_and_progress(source_dir, output_pak, compression, &|_, _, _| {})
    }

    /// Create a PAK file from a directory with compression and progress callback
    ///
    /// # Errors
    /// Returns an error if the directory cannot be read or PAK creation fails.
    pub fn create_with_compression_and_progress<P: AsRef<Path>>(
        source_dir: P,
        output_pak: P,
        compression: CompressionMethod,
        progress: ProgressCallback,
    ) -> Result<()> {
        let writer = LspkWriter::new(source_dir.as_ref())?
            .with_compression(compression);
        writer.write_with_progress(output_pak.as_ref(), progress)?;
        Ok(())
    }

    /// List contents of a PAK file
    ///
    /// # Errors
    /// Returns an error if the PAK file cannot be read.
    pub fn list<P: AsRef<Path>>(pak_path: P) -> Result<Vec<String>> {
        Self::list_with_progress(pak_path, &|_, _, _| {})
    }

    /// List contents of a PAK file with progress callback
    ///
    /// # Errors
    /// Returns an error if the PAK file cannot be read.
    pub fn list_with_progress<P: AsRef<Path>>(
        pak_path: P,
        progress: ProgressCallback,
    ) -> Result<Vec<String>> {
        let file = File::open(pak_path.as_ref())?;

        let mut reader = LspkReader::with_path(file, pak_path.as_ref());

        progress(0, 1, "Reading PAK...");

        let entries = reader.list_files()?;

        progress(1, 1, "Complete");

        Ok(entries.iter()
            .map(|e| e.path.to_string_lossy().to_string())
            .collect())
    }

    /// Extract specific files from a PAK to a directory
    ///
    /// Takes a list of file paths (as they appear in the PAK) and extracts only those files.
    /// File paths should match exactly as returned by `list()`.
    ///
    /// # Errors
    /// Returns an error if the PAK file cannot be read or extraction fails.
    pub fn extract_files<P: AsRef<Path>, S: AsRef<str>>(
        pak_path: P,
        output_dir: P,
        file_paths: &[S],
    ) -> Result<()> {
        Self::extract_files_with_progress(pak_path, output_dir, file_paths, &|_, _, _| {})
    }

    /// Extract specific files from a PAK to a directory with progress callback
    ///
    /// Takes a list of file paths (as they appear in the PAK) and extracts only those files.
    /// The callback receives (current, total, filename) during extraction.
    /// Uses parallel decompression for improved performance on multi-core systems.
    ///
    /// # Errors
    /// Returns an error if the PAK file cannot be read or extraction fails.
    pub fn extract_files_with_progress<P: AsRef<Path>, S: AsRef<str>>(
        pak_path: P,
        output_dir: P,
        file_paths: &[S],
        progress: ProgressCallback,
    ) -> Result<()> {
        if file_paths.is_empty() {
            return Ok(());
        }

        let mut file = File::open(pak_path.as_ref())?;
        let mut reader = LspkReader::with_path(File::open(pak_path.as_ref())?, pak_path.as_ref());

        // Build a set of requested paths for fast lookup
        let requested: std::collections::HashSet<&str> = file_paths
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();

        // Get file list and filter to only requested files
        let all_entries = reader.list_files()?;
        let entries_to_extract: Vec<_> = all_entries
            .into_iter()
            .filter(|e| {
                // Skip .DS_Store files
                if e.path.file_name() == Some(std::ffi::OsStr::new(".DS_Store")) {
                    return false;
                }
                requested.contains(e.path.to_string_lossy().as_ref())
            })
            .collect();

        if entries_to_extract.is_empty() {
            return Err(Error::ConversionError(
                "None of the requested files were found in the PAK".to_string()
            ));
        }

        std::fs::create_dir_all(&output_dir)?;

        progress(0, entries_to_extract.len(), "Reading compressed data...");

        // Phase 1: Read all compressed data sequentially (PAK is a single file)
        let compressed_files: Vec<CompressedFile> = entries_to_extract
            .iter()
            .filter_map(|entry| {
                // Seek and read compressed data
                if file.seek(SeekFrom::Start(entry.offset)).is_err() {
                    tracing::warn!("Failed to seek to {}", entry.path.display());
                    return None;
                }

                let mut compressed_data = vec![0u8; entry.size_compressed as usize];
                if file.read_exact(&mut compressed_data).is_err() {
                    tracing::warn!("Failed to read {}", entry.path.display());
                    return None;
                }

                Some(CompressedFile {
                    entry: entry.clone(),
                    compressed_data,
                })
            })
            .collect();

        let files_to_process = compressed_files.len();
        let processed = AtomicUsize::new(0);
        let output_dir = output_dir.as_ref();

        // Phase 2: Decompress and write in parallel
        let errors: Vec<(PathBuf, String)> = compressed_files
            .par_iter()
            .filter_map(|cf| {
                let file_name = cf
                    .entry
                    .path
                    .file_name().map_or_else(|| cf.entry.path.to_string_lossy().to_string(), |n| n.to_string_lossy().to_string());

                // Update progress (atomic)
                let current = processed.fetch_add(1, Ordering::SeqCst) + 1;
                progress(current, files_to_process, &file_name);

                // Decompress
                let data = match decompress_data(
                    &cf.compressed_data,
                    cf.entry.compression,
                    cf.entry.size_decompressed,
                ) {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::warn!("Failed to decompress {}: {}", cf.entry.path.display(), e);
                        return Some((cf.entry.path.clone(), e.to_string()));
                    }
                };

                // Calculate output path (handle virtual texture subfolders)
                let output_path = if is_virtual_texture_file(&file_name) {
                    if let Some(subfolder) = get_virtual_texture_subfolder(&file_name) {
                        if let Some(parent) = cf.entry.path.parent() {
                            output_dir.join(parent).join(&subfolder).join(&file_name)
                        } else {
                            output_dir.join(&subfolder).join(&file_name)
                        }
                    } else {
                        output_dir.join(&cf.entry.path)
                    }
                } else {
                    output_dir.join(&cf.entry.path)
                };

                // Create parent directories (idempotent)
                if let Some(parent) = output_path.parent()
                    && let Err(e) = std::fs::create_dir_all(parent) {
                        return Some((cf.entry.path.clone(), format!("Failed to create dir: {e}")));
                    }

                // Write file
                if let Err(e) = std::fs::write(&output_path, &data) {
                    return Some((cf.entry.path.clone(), format!("Failed to write: {e}")));
                }

                None
            })
            .collect();

        // If there were errors, return a summary error
        if !errors.is_empty() {
            return Err(Error::ConversionError(format!(
                "Extracted {} files with {} errors. First error: {}",
                files_to_process - errors.len(),
                errors.len(),
                errors[0].1
            )));
        }

        Ok(())
    }

    /// Read a single file's bytes from a PAK without writing to disk
    ///
    /// Returns the decompressed file contents, or an error if the file is not found.
    ///
    /// # Errors
    /// Returns an error if the PAK cannot be read or the file is not found.
    pub fn read_file_bytes<P: AsRef<Path>>(pak_path: P, file_path: &str) -> Result<Vec<u8>> {
        let file = File::open(pak_path.as_ref())?;
        let mut reader = LspkReader::with_path(file, pak_path.as_ref());

        // Get file list
        let entries = reader.list_files()?;

        // Find the requested file
        let entry = entries
            .into_iter()
            .find(|e| e.path.to_string_lossy() == file_path)
            .ok_or_else(|| Error::FileNotFoundInPak(file_path.to_string()))?;

        // Decompress and return
        reader.decompress_file(&entry)
    }

    /// Read multiple files' bytes from a PAK without writing to disk
    ///
    /// Returns a map of file paths to their decompressed contents.
    /// Files that fail to decompress are skipped with a warning.
    /// Uses parallel decompression for improved performance on multi-core systems.
    ///
    /// # Errors
    /// Returns an error if the PAK cannot be read.
    pub fn read_files_bytes<P: AsRef<Path>, S: AsRef<str>>(
        pak_path: P,
        file_paths: &[S],
    ) -> Result<std::collections::HashMap<String, Vec<u8>>> {
        use std::collections::HashMap;

        if file_paths.is_empty() {
            return Ok(HashMap::new());
        }

        let mut file = File::open(pak_path.as_ref())?;
        let mut reader = LspkReader::with_path(File::open(pak_path.as_ref())?, pak_path.as_ref());

        // Build a set of requested paths
        let requested: std::collections::HashSet<&str> = file_paths
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();

        // Get file list and filter
        let all_entries = reader.list_files()?;
        let entries_to_read: Vec<_> = all_entries
            .into_iter()
            .filter(|e| requested.contains(e.path.to_string_lossy().as_ref()))
            .collect();

        // Phase 1: Read all compressed data sequentially (PAK is a single file)
        let compressed_files: Vec<(String, CompressedFile)> = entries_to_read
            .iter()
            .filter_map(|entry| {
                // Seek and read compressed data
                if file.seek(SeekFrom::Start(entry.offset)).is_err() {
                    tracing::warn!("Failed to seek to {}", entry.path.display());
                    return None;
                }

                let mut compressed_data = vec![0u8; entry.size_compressed as usize];
                if file.read_exact(&mut compressed_data).is_err() {
                    tracing::warn!("Failed to read {}", entry.path.display());
                    return None;
                }

                Some((
                    entry.path.to_string_lossy().to_string(),
                    CompressedFile {
                        entry: entry.clone(),
                        compressed_data,
                    },
                ))
            })
            .collect();

        // Phase 2: Decompress in parallel and collect results
        let decompressed: Vec<(String, Vec<u8>)> = compressed_files
            .par_iter()
            .filter_map(|(path, cf)| {
                match decompress_data(
                    &cf.compressed_data,
                    cf.entry.compression,
                    cf.entry.size_decompressed,
                ) {
                    Ok(data) => Some((path.clone(), data)),
                    Err(e) => {
                        tracing::warn!("Failed to decompress {}: {}", path, e);
                        None
                    }
                }
            })
            .collect();

        // Collect into HashMap
        Ok(decompressed.into_iter().collect())
    }

    /// Extract meta.lsx from a PAK
    ///
    /// # Errors
    /// Returns an error if the PAK cannot be read or meta.lsx is not found.
    pub fn extract_meta<P: AsRef<Path>>(pak_path: P) -> Result<String> {
        let file = File::open(pak_path.as_ref())?;

        let mut reader = LspkReader::with_path(file, pak_path.as_ref());
        let contents = reader.read_all(None)?;

        // Find meta.lsx
        let meta_file = contents.files.iter()
            .find(|f| {
                let path = &f.path;
                let mut components = path.components();

                // Look for Mods/*/meta.lsx pattern
                if let Some(first) = components.next()
                    && first.as_os_str() == "Mods"
                        && components.next().is_some()
                            && let Some(third) = components.next() {
                                return third.as_os_str() == "meta.lsx";
                            }
                false
            })
            .ok_or_else(|| Error::FileNotFoundInPak("meta.lsx".to_string()))?;

        String::from_utf8(meta_file.data.clone())
            .map_err(|e| Error::ConversionError(format!("Invalid UTF-8 in meta.lsx: {e}")))
    }
}

/// Compressed file data ready for parallel decompression
struct CompressedFile {
    entry: FileTableEntry,
    compressed_data: Vec<u8>,
}

/// Cache for PAK file tables to avoid re-decompressing the file table
/// for every file access during batch operations like deep search.
///
/// The file table decompression is the critical bottleneck: if a PAK has
/// 50,000 files and we search 1,000 of them, without caching we'd decompress
/// the file table 1,000 times. With caching, we decompress it once.
pub struct PakReaderCache {
    /// Cached file tables keyed by PAK path
    tables: std::collections::HashMap<PathBuf, Vec<FileTableEntry>>,
    /// Maximum number of PAK tables to cache
    max_paks: usize,
    /// Access order for LRU eviction (most recent at end)
    access_order: Vec<PathBuf>,
}

impl PakReaderCache {
    /// Create a new cache with a maximum number of PAK tables to hold
    #[must_use] 
    pub fn new(max_paks: usize) -> Self {
        Self {
            tables: std::collections::HashMap::new(),
            max_paks: max_paks.max(1),
            access_order: Vec::new(),
        }
    }

    /// Ensure the file table is loaded for this PAK
    fn ensure_loaded(&mut self, pak_path: &Path) -> Result<()> {
        if self.tables.contains_key(pak_path) {
            self.update_access_order(pak_path);
            return Ok(());
        }

        // Load the file table
        let file = File::open(pak_path)?;
        let mut reader = LspkReader::with_path(file, pak_path);
        let entries = reader.list_files()?;

        // Evict oldest entry if at capacity
        while self.tables.len() >= self.max_paks && !self.access_order.is_empty() {
            let to_evict = self.access_order.remove(0);
            self.tables.remove(&to_evict);
        }

        // Insert new entry
        self.tables.insert(pak_path.to_path_buf(), entries);
        self.access_order.push(pak_path.to_path_buf());

        Ok(())
    }

    /// Update access order for LRU (move to end)
    fn update_access_order(&mut self, pak_path: &Path) {
        if let Some(pos) = self.access_order.iter().position(|p| p == pak_path) {
            self.access_order.remove(pos);
        }
        self.access_order.push(pak_path.to_path_buf());
    }

    /// Read a file's bytes using the cached file table
    ///
    /// This is much faster than `PakOperations::read_file_bytes` when reading
    /// multiple files from the same PAK, as it reuses the decompressed file table.
    ///
    /// # Errors
    /// Returns an error if the PAK cannot be read or the file is not found.
    pub fn read_file_bytes(&mut self, pak_path: &Path, file_path: &str) -> Result<Vec<u8>> {
        self.ensure_loaded(pak_path)?;

        // Find the entry in the cached table
        let entry = self
            .tables
            .get(pak_path)
            .and_then(|t| t.iter().find(|e| e.path.to_string_lossy() == file_path))
            .ok_or_else(|| Error::FileNotFoundInPak(file_path.to_string()))?
            .clone();

        // Read compressed data from the PAK file
        let mut file = File::open(pak_path)?;
        file.seek(SeekFrom::Start(entry.offset))?;

        let mut compressed_data = vec![0u8; entry.size_compressed as usize];
        file.read_exact(&mut compressed_data)?;

        // Decompress and return
        decompress_data(&compressed_data, entry.compression, entry.size_decompressed)
    }

    /// Read multiple files' bytes in bulk with optimized I/O
    ///
    /// This is MUCH faster than calling `read_file_bytes` in a loop because:
    /// 1. Files are sorted by offset for sequential I/O (no random seeks)
    /// 2. All compressed data is read in one pass
    /// 3. Decompression happens in parallel
    ///
    /// Returns a `HashMap` of `file_path` -> decompressed bytes.
    /// Files that fail to read/decompress are silently skipped.
    ///
    /// # Errors
    /// Returns an error if the PAK cannot be read or the file table is not loaded.
    pub fn read_files_bulk(
        &mut self,
        pak_path: &Path,
        file_paths: &[&str],
    ) -> Result<std::collections::HashMap<String, Vec<u8>>> {
        use rayon::prelude::*;
        use std::collections::{HashMap, HashSet};

        self.ensure_loaded(pak_path)?;

        // Build set of requested paths for O(1) lookup
        let requested: HashSet<&str> = file_paths.iter().copied().collect();

        // Get matching entries from cached table
        let table = self.tables.get(pak_path).ok_or_else(|| {
            Error::FileNotFoundInPak(pak_path.to_string_lossy().to_string())
        })?;

        let mut entries_to_read: Vec<&FileTableEntry> = table
            .iter()
            .filter(|e| requested.contains(e.path.to_string_lossy().as_ref()))
            .collect();

        if entries_to_read.is_empty() {
            return Ok(HashMap::new());
        }

        // CRITICAL: Sort by offset for sequential I/O
        entries_to_read.sort_by_key(|e| e.offset);

        // Open file once for all reads
        let mut file = File::open(pak_path)?;

        // Phase 1: Read all compressed data sequentially
        let compressed_files: Vec<(String, Vec<u8>, CompressionMethod, u32)> = entries_to_read
            .iter()
            .filter_map(|entry| {
                // Seek and read
                if file.seek(SeekFrom::Start(entry.offset)).is_err() {
                    return None;
                }

                let mut compressed_data = vec![0u8; entry.size_compressed as usize];
                if file.read_exact(&mut compressed_data).is_err() {
                    return None;
                }

                Some((
                    entry.path.to_string_lossy().to_string(),
                    compressed_data,
                    entry.compression,
                    entry.size_decompressed,
                ))
            })
            .collect();

        // Phase 2: Decompress in parallel
        let results: Vec<(String, Vec<u8>)> = compressed_files
            .par_iter()
            .filter_map(|(path, data, compression, expected_size)| {
                decompress_data(data, *compression, *expected_size)
                    .ok()
                    .map(|bytes| (path.clone(), bytes))
            })
            .collect();

        Ok(results.into_iter().collect())
    }
}


/// Standalone LZ4 decompression (for parallel use)
fn decompress_lz4_standalone(compressed: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    // Try standard block decompression first
    if let Ok(data) = lz4_flex::block::decompress(compressed, expected_size) {
        return Ok(data);
    }

    // Try with a larger buffer
    let larger_size = expected_size.saturating_mul(2).max(65536);
    if let Ok(data) = lz4_flex::block::decompress(compressed, larger_size) {
        return Ok(data);
    }

    // Try decompressing without size hint
    if let Ok(data) = lz4_flex::decompress_size_prepended(compressed) {
        return Ok(data);
    }

    // Try treating it as a frame
    let mut decoder = lz4_flex::frame::FrameDecoder::new(compressed);
    let mut decompressed = Vec::with_capacity(expected_size);
    if decoder.read_to_end(&mut decompressed).is_ok() && !decompressed.is_empty() {
        return Ok(decompressed);
    }

    Err(Error::DecompressionError(format!(
        "Failed to decompress LZ4 data: all methods failed (compressed: {} bytes, expected: {} bytes)",
        compressed.len(),
        expected_size
    )))
}

/// Standalone Zlib decompression (for parallel use)
fn decompress_zlib_standalone(compressed: &[u8], expected_size: usize) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;

    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::with_capacity(expected_size);

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| Error::DecompressionError(format!("Failed to decompress Zlib data: {e}")))?;

    Ok(decompressed)
}

/// Decompress data based on compression method (standalone for parallel use)
fn decompress_data(
    compressed: &[u8],
    compression: CompressionMethod,
    size_decompressed: u32,
) -> Result<Vec<u8>> {
    if compression == CompressionMethod::None || size_decompressed == 0 {
        return Ok(compressed.to_vec());
    }

    match compression {
        CompressionMethod::None => Ok(compressed.to_vec()),
        CompressionMethod::Lz4 => decompress_lz4_standalone(compressed, size_decompressed as usize),
        CompressionMethod::Zlib => decompress_zlib_standalone(compressed, size_decompressed as usize),
    }
}

/// Check if a filename is a virtual texture file (.gts or .gtp)
fn is_virtual_texture_file(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.ends_with(".gts") || lower.ends_with(".gtp")
}

/// Extract the subfolder name for a virtual texture file
/// e.g., "`Albedo_Normal_Physical_0.gts`" -> "`Albedo_Normal_Physical_0`"
/// e.g., "`Albedo_Normal_Physical_0_abc123def.gtp`" -> "`Albedo_Normal_Physical_0`"
fn get_virtual_texture_subfolder(filename: &str) -> Option<String> {
    let stem = filename.strip_suffix(".gts")
        .or_else(|| filename.strip_suffix(".gtp"))
        .or_else(|| filename.strip_suffix(".GTS"))
        .or_else(|| filename.strip_suffix(".GTP"))?;

    // For .gts files, the stem is already the subfolder name
    // e.g., "Albedo_Normal_Physical_0" from "Albedo_Normal_Physical_0.gts"
    if filename.to_lowercase().ends_with(".gts") {
        return Some(stem.to_string());
    }

    // For .gtp files, strip the hash suffix
    // e.g., "Albedo_Normal_Physical_0_abc123...def" -> "Albedo_Normal_Physical_0"
    if let Some(last_underscore) = stem.rfind('_') {
        let suffix = &stem[last_underscore + 1..];
        // Hash is 32 hex characters
        if suffix.len() == 32 && suffix.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(stem[..last_underscore].to_string());
        }
    }

    // Fallback: use the full stem
    Some(stem.to_string())
}
