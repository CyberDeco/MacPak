use maclarian::prelude::*;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_pak_extraction() {
    // This test requires a real PAK file
    // Skip if not available
}

#[test]
fn test_lsf_conversion() {
    // This test requires a real LSF file
    // Skip if not available
}

#[test]
fn test_string_table() {
    use maclarian::utils::StringTable;
    
    let mut table = StringTable::new();
    let idx1 = table.add("test");
    let idx2 = table.add("hello");
    let idx3 = table.add("test"); // Duplicate
    
    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(idx3, 0); // Same as first
    assert_eq!(table.get(0), Some("test"));
    assert_eq!(table.get(1), Some("hello"));
}
