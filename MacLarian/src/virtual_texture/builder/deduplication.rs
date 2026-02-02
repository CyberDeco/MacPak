//! Tile deduplication using MD5 hashing
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`
//! SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

use super::tile_processor::ProcessedTile;
use std::collections::HashMap;

/// Build deduplication map by hashing tiles (memory-efficient)
///
/// Returns a tuple of:
/// - `is_first`: `Vec<bool>` where `is_first[i]` is true if tile `i` is the first occurrence of its hash
/// - `unique_idx`: `Vec<usize>` where `unique_idx[i]` is the unique tile index for tile `i`
///
/// This allows streaming compression: only compress tiles where `is_first[i]` is true,
/// and reference the existing chunk for duplicates.
pub fn build_dedup_map(tiles: &[ProcessedTile]) -> (Vec<bool>, Vec<usize>) {
    let mut hash_to_unique_idx: HashMap<[u8; 16], usize> = HashMap::with_capacity(tiles.len());
    let mut is_first: Vec<bool> = Vec::with_capacity(tiles.len());
    let mut unique_idx: Vec<usize> = Vec::with_capacity(tiles.len());
    let mut next_unique = 0;

    for tile in tiles {
        let hash = compute_md5(&tile.full_data());

        if let Some(&existing) = hash_to_unique_idx.get(&hash) {
            is_first.push(false);
            unique_idx.push(existing);
        } else {
            hash_to_unique_idx.insert(hash, next_unique);
            is_first.push(true);
            unique_idx.push(next_unique);
            next_unique += 1;
        }
    }

    (is_first, unique_idx)
}

/// Compute MD5 hash of data
fn compute_md5(data: &[u8]) -> [u8; 16] {
    let digest = md5::compute(data);
    *digest
}
