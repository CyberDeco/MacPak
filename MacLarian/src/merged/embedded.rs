//! Embedded production database (DEPRECATED)
//!
//! This module provides access to a pre-built asset database that's compiled into the binary.
//!
//! **Deprecated**: This approach embeds an 18MB JSON file into the binary, which is not
//! suitable for crate publishing. Use [`GameDataResolver`](super::GameDataResolver) instead,
//! which builds the database on-the-fly from game PAK files.
//!
//! # Migration
//!
//! Replace:
//! ```ignore
//! let db = embedded_database_cached();
//! ```
//!
//! With:
//! ```ignore
//! let resolver = GameDataResolver::auto_detect()?;
//! let db = resolver.database();
//! ```

use super::types::MergedDatabase;

/// Embedded production database JSON
const EMBEDDED_DB_JSON: &str = include_str!("../../data/models_textures_db.json");

/// Load the embedded database
///
/// **Deprecated**: Use [`GameDataResolver::auto_detect()`](super::GameDataResolver::auto_detect) instead.
///
/// Returns a pre-built database containing mappings for all supported
/// armor, clothing, and creature meshes. The database is embedded at compile
/// time so no file I/O is required at runtime.
///
/// # Panics
/// Panics if the embedded JSON database is malformed. This would indicate
/// a build-time error and should never occur in a properly built binary.
#[must_use]
#[deprecated(since = "0.2.0", note = "Use GameDataResolver::auto_detect() instead")]
pub fn embedded_database() -> MergedDatabase {
    serde_json::from_str(EMBEDDED_DB_JSON).expect("Embedded database JSON should be valid")
}

/// Load the embedded production database (cached version)
///
/// **Deprecated**: Use [`GameDataResolver::auto_detect()`](super::GameDataResolver::auto_detect) instead.
///
/// Same as `embedded_database()` but only parses the JSON once.
/// Subsequent calls return a reference to the cached database.
#[deprecated(since = "0.2.0", note = "Use GameDataResolver::auto_detect() instead")]
pub fn embedded_database_cached() -> &'static MergedDatabase {
    use std::sync::OnceLock;
    static DB: OnceLock<MergedDatabase> = OnceLock::new();
    #[allow(deprecated)]
    DB.get_or_init(embedded_database)
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]
    use super::*;

    #[test]
    fn test_embedded_database_loads() {
        let db = embedded_database();
        assert!(!db.visuals_by_id.is_empty());
    }

    #[test]
    fn test_embedded_database_cached() {
        let db1 = embedded_database_cached();
        let db2 = embedded_database_cached();
        assert_eq!(db1.visuals_by_id.len(), db2.visuals_by_id.len());
    }
}
