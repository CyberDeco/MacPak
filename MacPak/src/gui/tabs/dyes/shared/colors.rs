//! Color utilities for the Dyes tab
//!
//! Provides shared functions for collecting and resetting color picker values.

use super::{COLOR_REGISTRY, DEFAULT_HEX};
use crate::gui::state::DyesState;
use floem::prelude::*;
use std::collections::HashMap;

/// Collect all color values from state into a HashMap.
/// Always includes all colors regardless of their value.
pub fn collect_all_colors(state: &DyesState) -> HashMap<String, String> {
    state
        .all_colors()
        .iter()
        .map(|entry| (entry.name.to_string(), entry.hex.get()))
        .collect()
}

/// Collect color values, only including optional colors if they're not default.
/// Required colors are always included. Optional colors are only included
/// if their value differs from the default gray.
pub fn collect_colors_skip_defaults(state: &DyesState) -> HashMap<String, String> {
    state
        .all_colors()
        .iter()
        .enumerate()
        .filter_map(|(i, entry)| {
            let def = &COLOR_REGISTRY[i];
            let hex = entry.hex.get();

            // Always include required colors, only include optional if not default
            if def.is_required || hex.to_lowercase() != DEFAULT_HEX {
                Some((entry.name.to_string(), hex))
            } else {
                None
            }
        })
        .collect()
}

/// Reset all color pickers to default gray
pub fn reset_colors_to_default(state: &DyesState) {
    for entry in state.all_colors() {
        entry.hex.set(DEFAULT_HEX.to_string());
    }
}

/// Load colors from a HashMap into the state color pickers.
/// Only sets colors that exist in the provided HashMap.
pub fn load_colors_from_map(state: &DyesState, colors: &HashMap<String, String>) {
    for (param_name, hex_color) in colors {
        if let Some(hex_signal) = state.color_hex(param_name) {
            hex_signal.set(hex_color.clone());
        }
    }
}
