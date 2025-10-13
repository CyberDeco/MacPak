//! Hashing utilities

/// DJB2 hash algorithm (used by LSLib)
pub fn hash_string_djb2(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    hash
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
