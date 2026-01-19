//! Embedded production database
//!
//! Provides access to a pre-built asset database that's compiled into the binary.

use super::types::MergedDatabase;

/// Embedded production database JSON
const EMBEDDED_DB_JSON: &str = include_str!("../../data/models_textures_db.json");

/// Load the embedded database
///
/// Returns a pre-built database containing mappings for all supported
/// armor, clothing, and creature meshes. The database is embedded at compile
/// time so no file I/O is required at runtime.
///
/// # Panics
/// Panics if the embedded JSON database is malformed. This would indicate
/// a build-time error and should never occur in a properly built binary.
///
/// # Example
///
/// ```
/// use MacLarian::merged::embedded_database;
///
/// let db = embedded_database();
/// println!("Database contains {} visuals", db.visuals_by_id.len());
///
/// // Look up by visual name
/// if let Some(asset) = db.get_by_visual_name("HUM_M_ARM_Leather_A_Body") {
///     println!("GR2 path: {}", asset.gr2_path);
/// }
/// ```
#[must_use]
pub fn embedded_database() -> MergedDatabase {
    serde_json::from_str(EMBEDDED_DB_JSON).expect("Embedded database JSON should be valid")
}

/// Load the embedded production database (cached version)
///
/// Same as `embedded_database()` but only parses the JSON once.
/// Subsequent calls return a reference to the cached database.
pub fn embedded_database_cached() -> &'static MergedDatabase {
    use std::sync::OnceLock;
    static DB: OnceLock<MergedDatabase> = OnceLock::new();
    DB.get_or_init(embedded_database)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_database_loads() {
        let db = embedded_database();
        assert!(db.visuals_by_id.len() > 0);
    }

    #[test]
    fn test_embedded_database_cached() {
        let db1 = embedded_database_cached();
        let db2 = embedded_database_cached();
        assert_eq!(db1.visuals_by_id.len(), db2.visuals_by_id.len());
    }
}
