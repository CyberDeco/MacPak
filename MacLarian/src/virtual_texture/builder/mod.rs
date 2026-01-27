//! Virtual texture builder module
//!
//! SPDX-FileCopyrightText: 2025 CyberDeco
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
//!
//! This module provides functionality for creating virtual textures (GTS/GTP files)
//! from source DDS textures.
//!
//! # Example
//!
//! ```no_run
//! use maclarian::virtual_texture::builder::{VirtualTextureBuilder, SourceTexture};
//!
//! let result = VirtualTextureBuilder::new()
//!     .add_texture(
//!         SourceTexture::new("MyTexture")
//!             .with_base_map("base.dds")
//!             .with_normal_map("normal.dds")
//!     )
//!     .build("output/")?;
//! # Ok::<(), maclarian::error::Error>(())
//! ```

pub mod config;
pub mod geometry;
pub mod tile_processor;
pub mod compression;
pub mod deduplication;

pub use config::{
    BcFormat, SourceTexture, TileCompressionPreference, TileSetConfiguration,
};

use crate::error::{Error, Result};
use crate::virtual_texture::types::{GtsFlatTileInfo, GtsCodec};
use crate::virtual_texture::writer::{
    fourcc::build_metadata_tree,
    gts_writer::{GtsWriter, LayerInfo, LevelInfo as GtsLevelInfo, PageFileInfo, create_bc_parameter_block},
    gtp_writer::{GtpWriter, Chunk},
};
use rayon::prelude::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

use self::compression::{compress_tile, CompressedTile};
use self::deduplication::build_dedup_map;
use self::geometry::calculate_geometry;
use self::tile_processor::{DdsTexture, extract_tiles_from_dds, ProcessedTile};

/// Build progress phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPhase {
    /// Validating configuration and inputs
    Validating,
    /// Calculating texture geometry and tile layout
    CalculatingGeometry,
    /// Extracting tiles from source textures
    ExtractingTiles,
    /// Generating tile borders
    GeneratingBorders,
    /// Embedding mip levels
    EmbeddingMips,
    /// Deduplicating identical tiles
    Deduplicating,
    /// Compressing tile data
    Compressing,
    /// Writing GTP page files
    WritingGtp,
    /// Writing GTS metadata file
    WritingGts,
    /// Build complete
    Complete,
}

impl BuildPhase {
    /// Get a human-readable description of this phase
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Validating => "Validating configuration",
            Self::CalculatingGeometry => "Calculating geometry",
            Self::ExtractingTiles => "Extracting tiles",
            Self::GeneratingBorders => "Generating borders",
            Self::EmbeddingMips => "Embedding mip levels",
            Self::Deduplicating => "Deduplicating tiles",
            Self::Compressing => "Compressing tiles",
            Self::WritingGtp => "Writing page files",
            Self::WritingGts => "Writing metadata",
            Self::Complete => "Complete",
        }
    }
}

/// Progress information during build
#[derive(Debug, Clone)]
pub struct BuildProgress {
    /// Current phase
    pub phase: BuildPhase,
    /// Current item index (0-based)
    pub current: usize,
    /// Total items in this phase
    pub total: usize,
    /// Optional message with details
    pub message: Option<String>,
}

impl BuildProgress {
    /// Create a new progress update
    #[must_use]
    pub fn new(phase: BuildPhase, current: usize, total: usize) -> Self {
        Self {
            phase,
            current,
            total,
            message: None,
        }
    }

    /// Create a progress update with a message
    #[must_use]
    pub fn with_message(phase: BuildPhase, current: usize, total: usize, message: impl Into<String>) -> Self {
        Self {
            phase,
            current,
            total,
            message: Some(message.into()),
        }
    }

    /// Get the progress percentage (0.0 - 1.0)
    #[must_use]
    pub fn percentage(&self) -> f32 {
        if self.total == 0 {
            1.0
        } else {
            self.current as f32 / self.total as f32
        }
    }
}

/// Result of building a virtual texture set
#[derive(Debug)]
pub struct BuildResult {
    /// Path to the generated GTS file
    pub gts_path: PathBuf,
    /// Paths to the generated GTP files
    pub gtp_paths: Vec<PathBuf>,
    /// Total number of tiles created
    pub tile_count: usize,
    /// Number of unique tiles (after deduplication)
    pub unique_tile_count: usize,
    /// Total size of all generated files in bytes
    pub total_size_bytes: u64,
}

/// Builder for creating virtual texture sets
pub struct VirtualTextureBuilder {
    config: TileSetConfiguration,
    textures: Vec<SourceTexture>,
    guid: [u8; 16],
    name: Option<String>,
}

impl Default for VirtualTextureBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualTextureBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        Self {
            config: TileSetConfiguration::default(),
            textures: Vec::new(),
            guid: *uuid.as_bytes(),
            name: None,
        }
    }

    /// Create a new builder with the specified configuration
    #[must_use]
    pub fn with_config(config: TileSetConfiguration) -> Self {
        let uuid = Uuid::new_v4();
        Self {
            config,
            textures: Vec::new(),
            guid: *uuid.as_bytes(),
            name: None,
        }
    }

    /// Set the name for the output files
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add a texture to the build
    #[must_use]
    pub fn add_texture(mut self, texture: SourceTexture) -> Self {
        self.textures.push(texture);
        self
    }

    /// Set the tile compression preference
    #[must_use]
    pub fn compression(mut self, compression: TileCompressionPreference) -> Self {
        self.config.compression = compression;
        self
    }

    /// Set the tile dimensions
    #[must_use]
    pub fn tile_size(mut self, width: u32, height: u32) -> Self {
        self.config.tile_width = width;
        self.config.tile_height = height;
        self
    }

    /// Enable or disable mip embedding
    #[must_use]
    pub fn embed_mip(mut self, enable: bool) -> Self {
        self.config.embed_mip = enable;
        self
    }

    /// Enable or disable tile deduplication
    #[must_use]
    pub fn deduplicate(mut self, enable: bool) -> Self {
        self.config.deduplicate = enable;
        self
    }

    /// Build the virtual texture set
    ///
    /// # Arguments
    /// * `output_dir` - Directory to write the GTS and GTP files to
    ///
    /// # Returns
    /// Result containing build information on success
    pub fn build<P: AsRef<Path>>(self, output_dir: P) -> Result<BuildResult> {
        self.build_with_progress(output_dir, |_| {})
    }

    /// Build with simple progress callback (matches reader/batch API)
    ///
    /// Progress callback receives (current, total, description) matching the
    /// signature used by `extract_gts_file` and `extract_batch`.
    ///
    /// # Arguments
    /// * `output_dir` - Directory to write the GTS and GTP files to
    /// * `progress` - Callback function that receives (current, total, description)
    ///
    /// # Returns
    /// Result containing build information on success
    pub fn build_with_simple_progress<P, F>(self, output_dir: P, progress: F) -> Result<BuildResult>
    where
        P: AsRef<Path>,
        F: Fn(usize, usize, &str) + Send + Sync,
    {
        self.build_with_progress(output_dir, |p| {
            let desc = p.message.as_deref().unwrap_or(p.phase.description());
            progress(p.current, p.total, desc);
        })
    }

    /// Build the virtual texture set with progress reporting
    ///
    /// # Arguments
    /// * `output_dir` - Directory to write the GTS and GTP files to
    /// * `progress` - Callback function that receives progress updates
    ///
    /// # Returns
    /// Result containing build information on success
    pub fn build_with_progress<P, F>(self, output_dir: P, progress: F) -> Result<BuildResult>
    where
        P: AsRef<Path>,
        F: Fn(&BuildProgress) + Send + Sync,
    {
        let output_dir = output_dir.as_ref();

        // Phase: Validate
        progress(&BuildProgress::new(BuildPhase::Validating, 0, 1));
        self.validate()?;

        // Determine output name
        let name = self.name.as_ref()
            .or_else(|| self.textures.first().map(|t| &t.name))
            .cloned()
            .unwrap_or_else(|| "VirtualTexture".to_string());

        // Create output directory if needed
        std::fs::create_dir_all(output_dir)?;

        // For now, support single texture (first one)
        let texture = &self.textures[0];
        let layers_present = [
            texture.base_map.is_some(),
            texture.normal_map.is_some(),
            texture.physical_map.is_some(),
        ];

        // Phase: Calculate Geometry
        progress(&BuildProgress::new(BuildPhase::CalculatingGeometry, 0, 1));

        // Load first available layer to get dimensions
        let (first_dds, _first_layer_idx) = self.load_first_layer(texture)?;
        let tex_info = (
            texture.name.clone(),
            first_dds.width,
            first_dds.height,
        );

        // Limit mip levels to what's actually in the DDS file
        let geometry = calculate_geometry(&[tex_info], layers_present, &self.config, Some(first_dds.mip_count));

        // Phase: Extract Tiles
        progress(&BuildProgress::new(BuildPhase::ExtractingTiles, 0, 3));

        // Pre-allocate based on estimated total tiles across all layers
        let estimated_tiles: usize = geometry.tiles_per_layer.iter().map(|t| t.len()).sum();
        let mut all_tiles: Vec<ProcessedTile> = Vec::with_capacity(estimated_tiles);
        let layer_paths = texture.layer_paths();

        // Load DDS textures for each layer
        let mut dds_textures: [Option<DdsTexture>; 3] = [None, None, None];
        for (i, path) in layer_paths.iter().enumerate() {
            if let Some(p) = path {
                progress(&BuildProgress::with_message(
                    BuildPhase::ExtractingTiles,
                    i,
                    3,
                    format!("Loading layer {}", i),
                ));
                dds_textures[i] = Some(DdsTexture::load(p)?);
            }
        }

        // Extract tiles from each layer
        for (layer_idx, dds_opt) in dds_textures.iter().enumerate() {
            if let Some(dds) = dds_opt {
                let coords = &geometry.tiles_per_layer[layer_idx];
                if !coords.is_empty() {
                    progress(&BuildProgress::with_message(
                        BuildPhase::ExtractingTiles,
                        layer_idx,
                        3,
                        format!("Extracting {} tiles from layer {}", coords.len(), layer_idx),
                    ));
                    let tiles = extract_tiles_from_dds(dds, coords, &self.config)?;
                    all_tiles.extend(tiles);
                }
            }
        }

        let total_tile_count = all_tiles.len();

        // Phase: Deduplicate (build map only - memory efficient)
        progress(&BuildProgress::new(BuildPhase::Deduplicating, 0, total_tile_count));

        let (is_first, unique_idx) = if self.config.deduplicate {
            build_dedup_map(&all_tiles)
        } else {
            // No dedup: all tiles are unique
            let is_first: Vec<bool> = vec![true; all_tiles.len()];
            let unique_idx: Vec<usize> = (0..all_tiles.len()).collect();
            (is_first, unique_idx)
        };

        let unique_tile_count = is_first.iter().filter(|&&x| x).count();

        // Collect indices of unique tiles for parallel compression
        let unique_indices: Vec<usize> = is_first
            .iter()
            .enumerate()
            .filter_map(|(i, &first)| if first { Some(i) } else { None })
            .collect();

        // Phase: Compress (parallelized with rayon, only unique tiles)
        progress(&BuildProgress::new(BuildPhase::Compressing, 0, unique_tile_count));

        let processed = AtomicUsize::new(0);
        let compression = self.config.compression;

        let compressed_unique: Result<Vec<CompressedTile>> = unique_indices
            .par_iter()
            .map(|&idx| {
                let current = processed.fetch_add(1, Ordering::Relaxed);
                if current % 100 == 0 {
                    progress(&BuildProgress::new(BuildPhase::Compressing, current, unique_tile_count));
                }
                compress_tile(&all_tiles[idx].full_data(), compression)
            })
            .collect();

        let compressed_unique = compressed_unique?;
        progress(&BuildProgress::new(BuildPhase::Compressing, unique_tile_count, unique_tile_count));

        // Phase: Write GTP (streaming - write chunks and track locations)
        progress(&BuildProgress::new(BuildPhase::WritingGtp, 0, 1));

        // Generate hash from GUID for filename (extractor expects Name_HASH.gtp format)
        let gtp_hash: String = self.guid.iter().map(|b| format!("{b:02x}")).collect();
        let gtp_filename = format!("{name}_{gtp_hash}.gtp");
        let gtp_path = output_dir.join(&gtp_filename);

        let mut gtp_writer = GtpWriter::new(self.guid, self.config.page_size);

        // Determine compression strings from config
        let (compression1, compression2) = self.config.compression.compression_strings();

        // Write unique chunks to GTP and track their locations
        // chunk_locations[unique_idx] = (page_idx, chunk_idx)
        let mut chunk_locations: Vec<(u16, u16)> = Vec::with_capacity(unique_tile_count);

        for compressed in &compressed_unique {
            let chunk = Chunk {
                codec: GtsCodec::Bc,
                parameter_block_id: 0,
                data: compressed.data.clone(),
            };
            let (page_idx, chunk_idx) = gtp_writer.add_chunk(chunk);
            chunk_locations.push((page_idx, chunk_idx));
        }

        // Build flat_tile_infos for ALL tiles (including duplicates)
        // Each tile references the chunk location of its unique counterpart
        let mut flat_tile_infos: Vec<(GtsFlatTileInfo, usize, u32)> = Vec::with_capacity(total_tile_count);

        for (i, tile) in all_tiles.iter().enumerate() {
            let u_idx = unique_idx[i];
            let (page_idx, chunk_idx) = chunk_locations[u_idx];

            flat_tile_infos.push((
                GtsFlatTileInfo {
                    page_file_index: 0, // Single GTP file
                    page_index: page_idx,
                    chunk_index: chunk_idx,
                    d: 0,
                    packed_tile_id_index: 0, // Will be set when adding to GTS
                },
                tile.coord.level as usize,
                tile.packed_id,
            ));
        }

        // Write GTP file
        let gtp_file = File::create(&gtp_path)?;
        let mut gtp_buf = BufWriter::new(gtp_file);
        gtp_writer.write(&mut gtp_buf)?;
        drop(gtp_buf);

        // Phase: Write GTS
        progress(&BuildProgress::new(BuildPhase::WritingGts, 0, 1));

        let gts_path = output_dir.join(format!("{name}.gts"));

        let mut gts_writer = GtsWriter::new(
            self.guid,
            self.config.tile_width as i32,
            self.config.tile_height as i32,
            self.config.tile_border as i32,
            self.config.page_size,
        );

        // Add layers
        for (i, present) in layers_present.iter().enumerate() {
            if *present {
                // Data type: 6 = BC3, 12 = BC5, etc.
                let data_type = match i {
                    1 => 12, // Normal map uses BC5
                    _ => 6,  // Base and physical use BC3
                };
                gts_writer.add_layer(LayerInfo { data_type });
            }
        }

        // Add levels
        for level in &geometry.levels {
            gts_writer.add_level(GtsLevelInfo {
                width: level.width_tiles,
                height: level.height_tiles,
                width_pixels: level.width_pixels,
                height_pixels: level.height_pixels,
            });
        }

        // Add parameter block
        let param_block = create_bc_parameter_block(
            compression1,
            compression2,
            6, // BC3 data type
            BcFormat::Bc3.fourcc(),
            self.config.embed_mip,
        );
        gts_writer.add_parameter_block(param_block);

        // Add page file info
        gts_writer.add_page_file(PageFileInfo {
            filename: gtp_filename,
            num_pages: gtp_writer.num_pages(),
            guid: self.guid,
        });

        // Add packed tile IDs and flat tile infos (using tile_mapping for deduplication)
        // First, create entries for unique tiles
        for (mut info, level, packed_id) in flat_tile_infos {
            let packed_idx = gts_writer.add_packed_tile_id(packed_id);
            info.packed_tile_id_index = packed_idx;
            gts_writer.add_flat_tile_info(info, level);
        }

        // Build FourCC metadata
        let layer_info: Vec<(&str, &str)> = layers_present
            .iter()
            .enumerate()
            .filter(|(_, present)| **present)
            .map(|(i, _)| {
                match i {
                    0 => ("BaseMap", "BaseColor"),
                    1 => ("NormalMap", "NormalMap"),
                    2 => ("PhysicalMap", "PhysicalMap"),
                    _ => ("Unknown", "Unknown"),
                }
            })
            .collect();

        let fourcc_tree = build_metadata_tree(
            &texture.name,
            geometry.total_width,
            geometry.total_height,
            0, // x offset
            0, // y offset
            &layer_info,
            &self.guid,
        );
        gts_writer.set_fourcc_tree(fourcc_tree);

        // Write GTS file
        let gts_file = File::create(&gts_path)?;
        let mut gts_buf = BufWriter::new(gts_file);
        gts_writer.write(&mut gts_buf)?;
        drop(gts_buf);

        // Calculate total size
        let gts_size = std::fs::metadata(&gts_path)?.len();
        let gtp_size = std::fs::metadata(&gtp_path)?.len();
        let total_size = gts_size + gtp_size;

        progress(&BuildProgress::new(BuildPhase::Complete, 1, 1));

        Ok(BuildResult {
            gts_path,
            gtp_paths: vec![gtp_path],
            tile_count: total_tile_count,
            unique_tile_count,
            total_size_bytes: total_size,
        })
    }

    /// Load the first available layer to get texture dimensions
    fn load_first_layer(&self, texture: &SourceTexture) -> Result<(DdsTexture, usize)> {
        for (i, path) in texture.layer_paths().iter().enumerate() {
            if let Some(p) = path {
                let dds = DdsTexture::load(p)?;
                return Ok((dds, i));
            }
        }
        Err(Error::VirtualTexture("No layers found in texture".to_string()))
    }

    /// Validate the builder configuration and inputs
    fn validate(&self) -> Result<()> {
        // Validate configuration
        self.config.validate().map_err(|e| Error::VirtualTexture(e))?;

        // Check we have at least one texture
        if self.textures.is_empty() {
            return Err(Error::VirtualTexture("No textures added to builder".to_string()));
        }

        // Check all textures have at least one layer
        for tex in &self.textures {
            if !tex.has_any_layer() {
                return Err(Error::VirtualTexture(
                    format!("Texture '{}' has no layers defined", tex.name)
                ));
            }
        }

        // Validate texture file paths exist
        for tex in &self.textures {
            for (i, path) in tex.layer_paths().iter().enumerate() {
                if let Some(p) = path {
                    if !p.exists() {
                        let layer_name = match i {
                            0 => "base_map",
                            1 => "normal_map",
                            2 => "physical_map",
                            _ => "unknown",
                        };
                        return Err(Error::VirtualTexture(
                            format!("Texture '{}' {}: file not found: {}", tex.name, layer_name, p.display())
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}
