//! Tile deduplication using MD5 hashing
//!
//! SPDX-FileCopyrightText: 2025 CyberDeco
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use std::collections::HashMap;
use super::tile_processor::ProcessedTile;

/// Result of tile deduplication
#[derive(Debug)]
pub struct DeduplicationResult {
    /// Unique tiles (one per unique hash)
    pub unique_tiles: Vec<ProcessedTile>,
    /// Mapping from original tile index to unique tile index
    pub tile_mapping: Vec<usize>,
    /// Number of duplicate tiles removed
    pub duplicates_removed: usize,
}

/// Deduplicate tiles based on their content hash
pub fn deduplicate_tiles(tiles: Vec<ProcessedTile>) -> DeduplicationResult {
    let mut hash_to_index: HashMap<[u8; 16], usize> = HashMap::new();
    let mut unique_tiles: Vec<ProcessedTile> = Vec::new();
    let mut tile_mapping: Vec<usize> = Vec::with_capacity(tiles.len());
    let mut duplicates_removed = 0;

    for tile in tiles {
        let hash = compute_md5(&tile.full_data());

        if let Some(&existing_idx) = hash_to_index.get(&hash) {
            // Duplicate found, map to existing tile
            tile_mapping.push(existing_idx);
            duplicates_removed += 1;
        } else {
            // New unique tile
            let new_idx = unique_tiles.len();
            hash_to_index.insert(hash, new_idx);
            tile_mapping.push(new_idx);
            unique_tiles.push(tile);
        }
    }

    DeduplicationResult {
        unique_tiles,
        tile_mapping,
        duplicates_removed,
    }
}

/// Compute MD5 hash of data
fn compute_md5(data: &[u8]) -> [u8; 16] {
    let digest = md5::compute(data);
    *digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::geometry::TileCoord;

    #[test]
    fn test_deduplication() {
        let tile1 = ProcessedTile {
            coord: TileCoord { layer: 0, level: 0, x: 0, y: 0 },
            packed_id: 0,
            data: vec![1, 2, 3, 4],
            mip_data: None,
        };
        let tile2 = ProcessedTile {
            coord: TileCoord { layer: 0, level: 0, x: 1, y: 0 },
            packed_id: 1,
            data: vec![1, 2, 3, 4], // Same data as tile1
            mip_data: None,
        };
        let tile3 = ProcessedTile {
            coord: TileCoord { layer: 0, level: 0, x: 2, y: 0 },
            packed_id: 2,
            data: vec![5, 6, 7, 8], // Different data
            mip_data: None,
        };

        let result = deduplicate_tiles(vec![tile1, tile2, tile3]);

        assert_eq!(result.unique_tiles.len(), 2);
        assert_eq!(result.tile_mapping, vec![0, 0, 1]);
        assert_eq!(result.duplicates_removed, 1);
    }
}
