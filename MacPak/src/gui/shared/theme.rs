//! Theme system for MacPak
//!
//! Provides light and dark mode color palettes with system appearance detection.

use floem::prelude::*;
use serde::{Deserialize, Serialize};

/// Available themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    System,
}

impl Theme {
    /// Get the effective theme (resolving System to actual Light/Dark)
    pub fn effective(&self) -> EffectiveTheme {
        match self {
            Self::Dark => EffectiveTheme::Dark,
            Self::Light => EffectiveTheme::Light,
            Self::System => {
                // Check macOS system appearance
                if is_system_dark_mode() {
                    EffectiveTheme::Dark
                } else {
                    EffectiveTheme::Light
                }
            }
        }
    }
}

/// Resolved theme (no System variant)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveTheme {
    Dark,
    Light,
}

/// Check if macOS is in dark mode
#[cfg(target_os = "macos")]
fn is_system_dark_mode() -> bool {
    use std::process::Command;
    Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("Dark"))
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
fn is_system_dark_mode() -> bool {
    true // Default to dark on non-macOS
}

/// Color palette for a theme
#[derive(Debug, Clone, Copy)]
pub struct ThemeColors {
    // Backgrounds
    pub bg_base: Color,
    pub bg_surface: Color,
    pub bg_elevated: Color,
    pub bg_hover: Color,
    pub bg_selected: Color,

    // Text
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_inverse: Color,

    // Borders
    pub border: Color,
    pub border_strong: Color,

    // Accents
    pub accent: Color,
    pub accent_hover: Color,
    pub success: Color,
    pub success_bg: Color,
    pub error: Color,
    pub error_bg: Color,
    pub warning: Color,
    pub warning_bg: Color,
}

impl ThemeColors {
    /// Get colors for the dark theme
    pub const fn dark() -> Self {
        Self {
            // Backgrounds
            bg_base: Color::rgb8(30, 30, 30),
            bg_surface: Color::rgb8(38, 38, 38),
            bg_elevated: Color::rgb8(50, 50, 50),
            bg_hover: Color::rgb8(60, 60, 60),
            bg_selected: Color::rgb8(70, 70, 70),

            // Text
            text_primary: Color::WHITE,
            text_secondary: Color::rgb8(180, 180, 180),
            text_muted: Color::rgb8(128, 128, 128),
            text_inverse: Color::rgb8(30, 30, 30),

            // Borders
            border: Color::rgb8(60, 60, 60),
            border_strong: Color::rgb8(80, 80, 80),

            // Accents
            accent: Color::rgb8(33, 150, 243),
            accent_hover: Color::rgb8(66, 165, 245),
            success: Color::rgb8(46, 125, 50),
            success_bg: Color::rgb8(30, 60, 35),
            error: Color::rgb8(211, 47, 47),
            error_bg: Color::rgb8(60, 30, 30),
            warning: Color::rgb8(255, 160, 0),
            warning_bg: Color::rgb8(60, 50, 30),
        }
    }

    /// Get colors for the light theme
    pub const fn light() -> Self {
        Self {
            // Backgrounds
            bg_base: Color::WHITE,
            bg_surface: Color::rgb8(250, 250, 250),
            bg_elevated: Color::rgb8(245, 245, 245),
            bg_hover: Color::rgb8(235, 235, 235),
            bg_selected: Color::rgb8(225, 225, 225),

            // Text
            text_primary: Color::rgb8(30, 30, 30),
            text_secondary: Color::rgb8(80, 80, 80),
            text_muted: Color::rgb8(128, 128, 128),
            text_inverse: Color::WHITE,

            // Borders
            border: Color::rgb8(220, 220, 220),
            border_strong: Color::rgb8(200, 200, 200),

            // Accents
            accent: Color::rgb8(25, 118, 210),
            accent_hover: Color::rgb8(21, 101, 192),
            success: Color::rgb8(46, 125, 50),
            success_bg: Color::rgb8(232, 245, 233),
            error: Color::rgb8(180, 30, 30),
            error_bg: Color::rgb8(255, 235, 235),
            warning: Color::rgb8(230, 140, 0),
            warning_bg: Color::rgb8(255, 248, 225),
        }
    }

    /// Get colors for the given effective theme
    pub const fn for_theme(theme: EffectiveTheme) -> Self {
        match theme {
            EffectiveTheme::Dark => Self::dark(),
            EffectiveTheme::Light => Self::light(),
        }
    }
}

/// Global theme signal
static THEME_SIGNAL: std::sync::OnceLock<RwSignal<Theme>> = std::sync::OnceLock::new();

/// Initialize the global theme signal
pub fn init_theme(theme: Theme) -> RwSignal<Theme> {
    let signal = RwSignal::new(theme);
    let _ = THEME_SIGNAL.set(signal);
    signal
}

/// Get the global theme signal (returns None if not initialized)
pub fn theme_signal() -> Option<RwSignal<Theme>> {
    THEME_SIGNAL.get().copied()
}

/// Get the current theme colors (convenience function)
/// Returns dark theme colors if theme signal is not initialized
pub fn colors() -> ThemeColors {
    theme_signal()
        .map(|s| ThemeColors::for_theme(s.get().effective()))
        .unwrap_or_else(ThemeColors::dark)
}

/// Create a reactive closure that returns theme-aware colors
/// Use this in style closures for automatic theme updates
pub fn themed<F, T>(f: F) -> impl Fn() -> T + Clone + 'static
where
    F: Fn(ThemeColors) -> T + Clone + 'static,
{
    move || {
        let colors = theme_signal()
            .map(|s| ThemeColors::for_theme(s.get().effective()))
            .unwrap_or_else(ThemeColors::dark);
        f(colors)
    }
}
