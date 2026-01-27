//! Config state management for recent files tracking

use crate::gui::state::ConfigState;

/// Global config state for recent files tracking
static CONFIG_STATE: std::sync::OnceLock<ConfigState> = std::sync::OnceLock::new();

/// Initialize the config state for recent files tracking
pub fn init_config_state(config: ConfigState) {
    let _ = CONFIG_STATE.set(config);
}

/// Add a file to recent files (if config state is available)
pub fn track_recent_file(path: &str) {
    if let Some(config) = CONFIG_STATE.get() {
        config.add_recent_file(path);
    }
}
