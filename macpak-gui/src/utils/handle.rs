//! Handle generation utilities for BG3 TranslatedStrings

use rand::Rng;

/// Generate a new random u64 handle for TranslatedStrings
pub fn generate_handle() -> String {
    let handle: u64 = rand::thread_rng().gen();
    handle.to_string()
}
