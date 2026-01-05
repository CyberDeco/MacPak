//! Embedded speaker database
//!
//! Provides compile-time access to speaker names.
//! The database is built from extracted BG3 game files using scripts/build_dialogue_db.py

use std::collections::HashMap;
use std::sync::OnceLock;
use serde::Deserialize;

/// Embedded speaker database JSON
const SPEAKERS_DB_JSON: &str = include_str!("../../../data/speakers_db.json");

/// Speaker database structure
#[derive(Debug, Deserialize)]
struct SpeakerDb {
    #[allow(dead_code)]
    version: u32,
    companions: HashMap<String, String>,
    handles: HashMap<String, String>,
}

/// Combined speaker lookup (companions + handles)
pub struct EmbeddedSpeakers {
    /// UUID -> Character name (for companions)
    pub companions: HashMap<String, String>,
    /// UUID -> DisplayName handle (for NPCs)
    pub handles: HashMap<String, String>,
}

impl EmbeddedSpeakers {
    /// Look up a speaker name by UUID (companion only)
    pub fn get_companion_name(&self, uuid: &str) -> Option<&str> {
        self.companions.get(uuid).map(|s| s.as_str())
    }

    /// Look up the DisplayName handle for a UUID
    pub fn get_display_handle(&self, uuid: &str) -> Option<&str> {
        self.handles.get(uuid).map(|s| s.as_str())
    }
}

/// Get the embedded speaker database (cached)
pub fn embedded_speakers() -> &'static EmbeddedSpeakers {
    static SPEAKERS: OnceLock<EmbeddedSpeakers> = OnceLock::new();
    SPEAKERS.get_or_init(|| {
        let db: SpeakerDb = serde_json::from_str(SPEAKERS_DB_JSON)
            .expect("Embedded speaker database should be valid JSON");
        EmbeddedSpeakers {
            companions: db.companions,
            handles: db.handles,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_speakers_loads() {
        let speakers = embedded_speakers();
        assert!(!speakers.companions.is_empty());
        assert!(!speakers.handles.is_empty());
        println!("Loaded {} companions, {} handles",
            speakers.companions.len(), speakers.handles.len());
    }

    #[test]
    fn test_companion_lookup() {
        let speakers = embedded_speakers();
        // Astarion's GlobalTemplate UUID
        assert_eq!(speakers.get_companion_name("c7c13742-bacd-460a-8f65-f864fe41f255"), Some("Astarion"));
        // Shadowheart's GlobalTemplate UUID
        assert_eq!(speakers.get_companion_name("3ed74f06-3c60-42dc-83f6-f034cb47c679"), Some("Shadowheart"));
    }
}
