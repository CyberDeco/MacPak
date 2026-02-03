//! CLI progress display utilities
//!
//! Provides yarnish-style progress display with step indicators, emojis,
//! and multi-progress support for batch operations.

use std::time::Duration;

use console::{Emoji, style};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};

// =============================================================================
// Emoji Constants (with ASCII fallbacks for terminals without emoji support)
// =============================================================================

/// Magnifying glass - for reading/scanning operations
pub static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍 ", "");
/// Package - for extraction/compression operations
pub static PACKAGE: Emoji<'_, '_> = Emoji("📦 ", "");
/// Floppy disk - for writing/saving operations
pub static DISK: Emoji<'_, '_> = Emoji("💾 ", "");
/// Gear - for processing/conversion operations
pub static GEAR: Emoji<'_, '_> = Emoji("⚙️  ", "");
/// Sparkles - for completion
pub static SPARKLE: Emoji<'_, '_> = Emoji("✨ ", "");
/// Truck - for batch/transport operations
pub static TRUCK: Emoji<'_, '_> = Emoji("🚚 ", "");
/// Link - for linking/indexing operations
pub static LINK: Emoji<'_, '_> = Emoji("🔗 ", "");
/// Document - for file operations
pub static DOCUMENT: Emoji<'_, '_> = Emoji("📄 ", "");
/// Picture - for texture/image operations
pub static PICTURE: Emoji<'_, '_> = Emoji("🖼️  ", "");
/// Cube - for 3D model operations
pub static CUBE: Emoji<'_, '_> = Emoji("📐 ", "");

// =============================================================================
// Step-Based Progress (Option A)
// =============================================================================

/// Print a step indicator: `[1/3] 📦 Message...`
///
/// # Example
/// ```ignore
/// print_step(1, 3, LOOKING_GLASS, "Reading PAK header...");
/// print_step(2, 3, PACKAGE, "Extracting files...");
/// print_step(3, 3, DISK, "Writing files...");
/// ```
pub fn print_step(current: usize, total: usize, emoji: Emoji, msg: &str) {
    println!(
        "{} {}{}",
        style(format!("[{current}/{total}]")).bold().dim(),
        emoji,
        msg
    );
}

/// Print completion message: `✨ Done in 2s`
pub fn print_done(elapsed: Duration) {
    println!("{} Done in {}", SPARKLE, HumanDuration(elapsed));
}

// =============================================================================
// Progress Styles
// =============================================================================

/// Spinner style for indeterminate progress
///
/// Format: `[1/4]  ⠋ processing file.pak`
///
/// # Panics
/// Panics if the template string is invalid (this is a compile-time constant).
#[must_use]
pub fn spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .expect("valid template")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
}

/// Progress bar style for determinate progress
///
/// Format: `Extracting [████████░░░░░░░░] 50/100`
///
/// # Panics
/// Panics if the template string is invalid (this is a compile-time constant).
#[must_use]
pub fn bar_style() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
        .expect("valid template")
}

/// Progress bar style with percentage
///
/// Format: `Extracting [████████░░░░░░░░] 50% (50/100)`
///
/// # Panics
/// Panics if the template string is invalid (this is a compile-time constant).
#[must_use]
pub fn bar_style_with_percent() -> ProgressStyle {
    ProgressStyle::default_bar()
        .template("{msg} [{bar:40.cyan/blue}] {percent}% ({pos}/{len})")
        .expect("valid template")
}

// =============================================================================
// Multi-Progress Helpers (Option B)
// =============================================================================

/// Create a new multi-progress manager for batch operations
#[must_use]
pub fn multi_progress() -> MultiProgress {
    MultiProgress::new()
}

/// Add a spinner to a multi-progress display
///
/// # Arguments
/// * `mp` - The multi-progress manager
/// * `prefix` - Prefix like `[1/4]`
/// * `msg` - Initial message
#[must_use]
pub fn add_spinner(mp: &MultiProgress, prefix: &str, msg: &str) -> ProgressBar {
    let pb = mp.add(ProgressBar::new_spinner());
    pb.set_style(spinner_style());
    pb.set_prefix(prefix.to_string());
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Add a progress bar to a multi-progress display
///
/// # Arguments
/// * `mp` - The multi-progress manager
/// * `total` - Total number of items
/// * `msg` - Progress bar message
#[must_use]
pub fn add_bar(mp: &MultiProgress, total: u64, msg: &str) -> ProgressBar {
    let pb = mp.add(ProgressBar::new(total));
    pb.set_style(bar_style());
    pb.set_message(msg.to_string());
    pb
}

// =============================================================================
// Simple Progress Helpers
// =============================================================================

/// Create a simple spinner (not part of multi-progress)
///
/// # Panics
/// Panics if the template string is invalid (this is a compile-time constant).
#[must_use]
pub fn simple_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .expect("valid template"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Create a simple progress bar (not part of multi-progress)
#[must_use]
pub fn simple_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(bar_style());
    pb.set_message(msg.to_string());
    pb
}
