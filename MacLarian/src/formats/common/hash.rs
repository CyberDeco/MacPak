//! Hashing utilities

/// DJB2 hash algorithm (used by `LSLib`)
#[must_use] 
pub fn hash_string_djb2(s: &str) -> u32 {
    let mut hash: u32 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(u32::from(byte));
    }
    hash
}

/// C# `String.GetHashCode()` equivalent for `LSLib` compatibility
#[must_use] 
pub fn hash_string_lslib(s: &str) -> u32 {
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

/// Generic string hash (alias for DJB2)
#[must_use] 
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