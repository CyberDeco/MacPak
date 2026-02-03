//! Public accessor methods for GtsFile.

use super::super::types::{GtsParameterBlock, TileCompression, TileLocation};
use super::GtsFile;

impl GtsFile {
    /// Get compression method for a parameter block.
    #[must_use]
    pub fn get_compression_method(&self, param_block_id: u32) -> TileCompression {
        match self.parameter_blocks.get(&param_block_id) {
            Some(GtsParameterBlock::BC(bc)) => bc.get_compression_method(),
            _ => TileCompression::Raw,
        }
    }

    /// Find the page file index by filename hash.
    #[must_use]
    pub fn find_page_file_index(&self, hash: &str) -> Option<u16> {
        for (i, pf) in self.page_files.iter().enumerate() {
            if pf.filename.contains(hash) {
                return Some(i as u16);
            }
        }
        None
    }

    /// Find the page file index by exact filename match.
    ///
    /// Used for mod GTP files that don't have a hash in their filename.
    #[must_use]
    pub fn find_page_file_index_by_name(&self, gtp_filename: &str) -> Option<u16> {
        for (i, pf) in self.page_files.iter().enumerate() {
            if pf.filename == gtp_filename {
                return Some(i as u16);
            }
        }
        None
    }

    /// Get tiles for a specific page file, organized by layer.
    ///
    /// Prefers level 0 (full resolution) but falls back to higher level numbers
    /// if a layer doesn't have level 0 tiles (e.g., PhysicalMap is often stored
    /// at lower resolution).
    #[must_use]
    pub(crate) fn get_tiles_for_page_file(&self, page_file_index: u16) -> [Vec<TileLocation>; 3] {
        // First pass: collect all tiles by layer and level
        let mut tiles_by_layer_level: [std::collections::HashMap<u8, Vec<TileLocation>>; 3] = [
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
        ];

        for tile_info in &self.flat_tile_infos {
            if tile_info.page_file_index != page_file_index {
                continue;
            }

            let packed_idx = tile_info.packed_tile_id_index as usize;
            if packed_idx >= self.packed_tiles.len() {
                continue;
            }

            let packed = &self.packed_tiles[packed_idx];
            let layer_idx = packed.layer as usize;

            if layer_idx >= 3 {
                continue;
            }

            tiles_by_layer_level[layer_idx]
                .entry(packed.level)
                .or_default()
                .push(TileLocation {
                    page: tile_info.page_index,
                    chunk: tile_info.chunk_index,
                    x: packed.x,
                    y: packed.y,
                });
        }

        // Second pass: for each layer, select the best available level (lowest number = highest res)
        let mut tiles_by_layer: [Vec<TileLocation>; 3] = [Vec::new(), Vec::new(), Vec::new()];

        for (layer_idx, level_map) in tiles_by_layer_level.iter().enumerate() {
            if level_map.is_empty() {
                continue;
            }

            // Find the minimum level (highest resolution available)
            let best_level = *level_map.keys().min().expect("level_map is non-empty");
            tiles_by_layer[layer_idx] = level_map.get(&best_level).cloned().unwrap_or_default();

            if best_level != 0 {
                tracing::debug!(
                    "Layer {layer_idx} using level {best_level} (level 0 not available)"
                );
            }
        }

        let num_layers = self.header.num_layers;
        let l0 = tiles_by_layer[0].len();
        let l1 = tiles_by_layer[1].len();
        let l2 = tiles_by_layer[2].len();
        tracing::debug!(
            "GTS num_layers={num_layers}, selected tiles by layer: [0]={l0}, [1]={l1}, [2]={l2}"
        );

        tiles_by_layer
    }

    /// Get content dimensions (tile size minus border).
    #[must_use]
    pub fn content_width(&self) -> i32 {
        self.header.tile_width - self.header.tile_border * 2
    }

    /// Get content height (tile size minus border).
    #[must_use]
    pub fn content_height(&self) -> i32 {
        self.header.tile_height - self.header.tile_border * 2
    }
}
