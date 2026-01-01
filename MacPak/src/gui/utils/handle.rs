//! Handle generation utilities for BG3 TranslatedStrings

/// Generate a new random u64 handle for TranslatedStrings
#[allow(dead_code)]
pub fn generate_handle() -> String {
    let handle: u64 = rand::Rng::r#gen(&mut rand::thread_rng());
    handle.to_string()
}
