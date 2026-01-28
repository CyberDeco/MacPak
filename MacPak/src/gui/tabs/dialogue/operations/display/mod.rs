//! Display node building - converts Dialog to DisplayNode list
//!
//! This module is organized into submodules:
//! - `chain`: Chain traversal helpers for following node chains to find text
//! - `maps`: Map building helpers for dialogue tree construction
//! - `tree`: Tree building logic for constructing the display node tree
//! - `resolve`: Resolution functions for converting UUIDs to display names

mod chain;
mod maps;
mod resolve;
mod tree;

// Re-export public API
pub use resolve::{
    resolve_difficulty_classes, resolve_flag_names, resolve_localized_text, resolve_speaker_names,
};
pub use tree::build_display_nodes;
