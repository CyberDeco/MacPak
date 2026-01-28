//! PAK file table caching for repeated file access

use crate::error::{Error, Result};
use super::super::lspk::{CompressionMethod, FileTableEntry, LspkReader};
use super::decompression::decompress_data;
use super::helpers::get_part_path;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Cache for PAK file tables to avoid re-decompressing the file table
pub struct PakReaderCache {
    /// Cached file tables keyed by PAK path
    tables: HashMap<PathBuf, Vec<FileTableEntry>>,
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
            tables: HashMap::new(),
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

    /// Read a single file's bytes using the cached file table.
    ///
    /// This is faster than [`crate::pak::PakOperations::read_file_bytes`] when reading multiple
    /// files from the same PAK, as it caches the decompressed file table in memory.
    /// The first call loads and caches the table; subsequent calls reuse it.
    ///
    /// For reading many files at once, prefer [`read_files_bulk`](Self::read_files_bulk)
    /// which optimizes I/O by sorting reads by disk offset.
    ///
    /// Supports multi-part archives (e.g., `Textures.pak` with `Textures_1.pak`).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // PakReaderCache is internal API (pub(crate))
    /// use std::path::Path;
    /// use maclarian::pak::PakReaderCache;
    ///
    /// let mut cache = PakReaderCache::new(4); // Cache up to 4 PAKs
    /// let pak = Path::new("Shared.pak");
    ///
    /// // First read loads the file table
    /// let meta = cache.read_file_bytes(pak, "Public/Shared/meta.lsx")?;
    ///
    /// // Subsequent reads reuse the cached table (fast)
    /// let other = cache.read_file_bytes(pak, "Public/Shared/other.lsf")?;
    /// # Ok::<(), maclarian::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the PAK cannot be read or the file is not found.
    #[allow(dead_code)] // Library API for on-demand cached file reading
    pub fn read_file_bytes(&mut self, pak_path: &Path, file_path: &str) -> Result<Vec<u8>> {
        self.ensure_loaded(pak_path)?;

        // Find the entry in the cached table
        let entry = self
            .tables
            .get(pak_path)
            .and_then(|t| t.iter().find(|e| e.path.to_string_lossy() == file_path))
            .ok_or_else(|| Error::FileNotFoundInPak(file_path.to_string()))?
            .clone();

        // Get the correct part file for this entry
        let part_path = get_part_path(pak_path, entry.archive_part)
            .ok_or_else(|| Error::ConversionError(
                format!("Cannot determine path for archive part {}", entry.archive_part)
            ))?;

        // Read compressed data from the correct part file
        let mut file = File::open(&part_path)?;
        file.seek(SeekFrom::Start(entry.offset))?;

        let mut compressed_data = vec![0u8; entry.size_compressed as usize];
        file.read_exact(&mut compressed_data)?;

        // Decompress and return
        decompress_data(&compressed_data, entry.compression, entry.size_decompressed)
    }

    /// Read multiple files' bytes in bulk with optimized I/O
    ///
    /// This is MUCH faster than calling `read_file_bytes` in a loop because:
    /// 1. Files are grouped by archive part, then sorted by offset for sequential I/O
    /// 2. All compressed data is read in one pass per part file
    /// 3. Decompression happens in parallel
    ///
    /// Supports multi-part archives (e.g., `Textures.pak` with `Textures_1.pak`, `Textures_2.pak`).
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
    ) -> Result<HashMap<String, Vec<u8>>> {
        use std::collections::HashSet;

        self.ensure_loaded(pak_path)?;

        // Build set of requested paths for O(1) lookup
        let requested: HashSet<&str> = file_paths.iter().copied().collect();

        // Get matching entries from cached table
        let table = self.tables.get(pak_path).ok_or_else(|| {
            Error::FileNotFoundInPak(pak_path.to_string_lossy().to_string())
        })?;

        let entries_to_read: Vec<&FileTableEntry> = table
            .iter()
            .filter(|e| requested.contains(e.path.to_string_lossy().as_ref()))
            .collect();

        if entries_to_read.is_empty() {
            return Ok(HashMap::new());
        }

        // Group entries by archive part for multi-part PAK support
        let mut entries_by_part: HashMap<u8, Vec<&FileTableEntry>> = HashMap::new();
        for entry in entries_to_read {
            entries_by_part.entry(entry.archive_part).or_default().push(entry);
        }

        // Phase 1: Read all compressed data sequentially from each part file
        let mut compressed_files: Vec<(String, Vec<u8>, CompressionMethod, u32)> = Vec::new();

        for (part, mut part_entries) in entries_by_part {
            let part_path = match get_part_path(pak_path, part) {
                Some(p) => p,
                None => continue,
            };

            if !part_path.exists() {
                tracing::warn!("Archive part file not found: {}", part_path.display());
                continue;
            }

            // Sort by offset for sequential I/O within this part
            part_entries.sort_by_key(|e| e.offset);

            let mut part_file = match File::open(&part_path) {
                Ok(f) => f,
                Err(_) => continue,
            };

            for entry in part_entries {
                // Seek and read from the correct part file
                if part_file.seek(SeekFrom::Start(entry.offset)).is_err() {
                    continue;
                }

                let mut compressed_data = vec![0u8; entry.size_compressed as usize];
                if part_file.read_exact(&mut compressed_data).is_err() {
                    continue;
                }

                compressed_files.push((
                    entry.path.to_string_lossy().to_string(),
                    compressed_data,
                    entry.compression,
                    entry.size_decompressed,
                ));
            }
        }

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
