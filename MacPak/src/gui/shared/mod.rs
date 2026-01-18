//! Shared components for batch operation tabs (GR2, Virtual Textures, etc.)
//!
//! This module provides reusable UI components and state traits for tabs that
//! perform batch file operations with progress tracking and results logging.

mod progress;
mod results;
mod styles;
pub mod theme;

pub use progress::{SharedProgress, progress_overlay};
pub use results::results_section;
pub use styles::{card_style, header_section, operation_button};
pub use theme::{Theme, ThemeColors, EffectiveTheme, init_theme, theme_signal, colors, themed};

use floem::prelude::*;
use im::Vector as ImVector;

/// Trait for state types that support batch operations with progress tracking.
///
/// Both `Gr2State` and `VirtualTexturesState` implement this trait, allowing
/// shared components to work with either state type.
pub trait BatchOperationState: Clone + 'static {
    /// Returns the signal indicating whether an operation is in progress
    fn is_processing(&self) -> RwSignal<bool>;

    /// Returns the signal containing the results log
    fn results_log(&self) -> RwSignal<ImVector<String>>;

    /// Returns the signal containing the status message
    fn status_message(&self) -> RwSignal<String>;

    /// Add a result message to the log
    fn add_result(&self, message: &str);

    /// Add multiple result messages in a batch
    fn add_results_batch(&self, messages: Vec<String>);

    /// Clear all results from the log
    fn clear_results(&self);

    /// Get the shared progress instance for this operation type
    fn get_shared_progress(&self) -> &'static SharedProgress;
}
