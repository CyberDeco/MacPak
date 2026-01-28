//! Hashing utilities
//!
//! SPDX-FileCopyrightText: 2025 `CyberDeco`, 2015 Norbyte (`LSLib`, MIT)
//!
//! SPDX-License-Identifier: MIT

/// C# `String.GetHashCode()` equivalent for `LSLib` compatibility
#[must_use]
pub(crate) fn hash_string_lslib(s: &str) -> u32 {
    let mut hash1 = 5381u32;
    let mut hash2 = hash1;

    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        hash1 = ((hash1 << 5).wrapping_add(hash1)) ^ u32::from(bytes[i]);
        if i + 1 < bytes.len() {
            hash2 = ((hash2 << 5).wrapping_add(hash2)) ^ u32::from(bytes[i + 1]);
        }
        i += 2;
    }

    hash1.wrapping_add(hash2.wrapping_mul(1566083941))
}
