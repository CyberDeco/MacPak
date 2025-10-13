//! Hashing utilities

/// DJB2 hash algorithm (used by LSLib)
pub fn hash_string_djb2(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
}

/// C# String.GetHashCode() equivalent for LSLib compatibility
pub fn hash_string_lslib(s: &str) -> u32 {
    let mut hash1 = 5381u32;
    let mut hash2 = hash1;
    
    let bytes = s.as_bytes();
    let mut i = 0;
    
    while i < bytes.len() {
        hash1 = ((hash1 << 5).wrapping_add(hash1)) ^ (bytes[i] as u32);
        if i + 1 < bytes.len() {
            hash2 = ((hash2 << 5).wrapping_add(hash2)) ^ (bytes[i + 1] as u32);
        }
        i += 2;
    }
    
    hash1.wrapping_add(hash2.wrapping_mul(1566083941))
}

/// Generic string hash (alias for DJB2)
pub fn hash_string(s: &str) -> u32 {
    hash_string_djb2(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_djb2_hash() {
        // Test known values
        assert_eq!(hash_string_djb2("test"), 2090756197);
        assert_eq!(hash_string_djb2(""), 5381);
    }
}