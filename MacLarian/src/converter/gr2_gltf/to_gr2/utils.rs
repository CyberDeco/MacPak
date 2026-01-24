//! SPDX-FileCopyrightText: 2025 CyberDeco, 2015 Norbyte (LSLib, MIT)
//!
//! SPDX-License-Identifier: MIT
//!
//! Utility functions for glTF → GR2 conversion.
//!
//! Re-exports shared utilities from the parent module, plus GR2-specific functions.

pub use crate::converter::gr2_gltf::shared::{f32_to_half, encode_qtangent};

/// Calculate CRC32 checksum (GR2 uses standard CRC-32)
#[must_use] 
pub fn crc32(data: &[u8]) -> u32 {
    const TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0u32;
        while i < 256 {
            let mut crc = i;
            let mut j = 0;
            while j < 8 {
                if crc & 1 != 0 {
                    crc = (crc >> 1) ^ 0xEDB88320;
                } else {
                    crc >>= 1;
                }
                j += 1;
            }
            table[i as usize] = crc;
            i += 1;
        }
        table
    };

    let mut crc = 0xFFFFFFFFu32;
    for &byte in data {
        crc = TABLE[((crc ^ u32::from(byte)) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crc32() {
        // Test vector from standard CRC-32
        let data = b"123456789";
        let crc = crc32(data);
        assert_eq!(crc, 0xCBF43926);
    }
}
