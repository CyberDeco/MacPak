//! Tools tab state

use floem::prelude::*;

/// Tools tab state (UUID, Handle, Color Picker, Version Calculator)
#[derive(Clone)]
pub struct ToolsState {
    // UUID
    pub generated_uuid: RwSignal<String>,
    pub uuid_format: RwSignal<UuidFormat>,
    pub uuid_history: RwSignal<Vec<String>>,

    // Handle
    pub generated_handle: RwSignal<String>,
    pub handle_history: RwSignal<Vec<String>>,

    // Color Picker
    pub color_hex: RwSignal<String>,
    pub color_r: RwSignal<u8>,
    pub color_g: RwSignal<u8>,
    pub color_b: RwSignal<u8>,
    pub color_a: RwSignal<u8>,
    pub color_history: RwSignal<Vec<String>>,

    // Version Calculator
    pub version_int: RwSignal<String>,
    pub version_major: RwSignal<u32>,
    pub version_minor: RwSignal<u32>,
    pub version_patch: RwSignal<u32>,
    pub version_build: RwSignal<u32>,

    // Status
    pub status_message: RwSignal<String>,
}

impl ToolsState {
    pub fn new() -> Self {
        Self {
            generated_uuid: RwSignal::new(String::new()),
            uuid_format: RwSignal::new(UuidFormat::Standard),
            uuid_history: RwSignal::new(Vec::new()),

            generated_handle: RwSignal::new(String::new()),
            handle_history: RwSignal::new(Vec::new()),

            color_hex: RwSignal::new("FF5500".to_string()),
            color_r: RwSignal::new(255),
            color_g: RwSignal::new(85),
            color_b: RwSignal::new(0),
            color_a: RwSignal::new(255),
            color_history: RwSignal::new(Vec::new()),

            version_int: RwSignal::new(String::new()),
            version_major: RwSignal::new(1),
            version_minor: RwSignal::new(0),
            version_patch: RwSignal::new(0),
            version_build: RwSignal::new(0),

            status_message: RwSignal::new(String::new()),
        }
    }
}

impl Default for ToolsState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UuidFormat {
    Standard,    // 8-4-4-4-12
    Compact,     // No dashes
    Larian,      // Larian's format (h prefix + specific format)
}
