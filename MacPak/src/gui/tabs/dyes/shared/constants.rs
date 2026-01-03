//! Shared constants for colors, spacing, and typography

use floem::prelude::Color;

// =============================================================================
// COLORS - Background
// =============================================================================

/// Card background - very light gray (250, 250, 250)
pub const BG_CARD: Color = Color::rgb8(250, 250, 250);

/// Read-only input background (245, 245, 245)
pub const BG_INPUT_READONLY: Color = Color::rgb8(245, 245, 245);

/// Disabled/subtle background (240, 240, 240)
pub const BG_DISABLED: Color = Color::rgb8(240, 240, 240);

/// Secondary button background - Tailwind gray-100 (243, 244, 246)
pub const BG_SECONDARY: Color = Color::rgb8(243, 244, 246);

/// Navigation button background - Tailwind gray-200 (229, 231, 235)
pub const BG_NAV_BUTTON: Color = Color::rgb8(229, 231, 235);

/// Success/green background (232, 245, 233)
pub const BG_SUCCESS: Color = Color::rgb8(232, 245, 233);

// =============================================================================
// COLORS - Border
// =============================================================================

/// Input border - medium gray (200, 200, 200)
pub const BORDER_INPUT: Color = Color::rgb8(200, 200, 200);

/// Card border - light gray (220, 220, 220)
pub const BORDER_CARD: Color = Color::rgb8(220, 220, 220);

/// Darker border (180, 180, 180)
pub const BORDER_DARK: Color = Color::rgb8(180, 180, 180);

/// Secondary border - Tailwind gray-300 (209, 213, 219)
pub const BORDER_SECONDARY: Color = Color::rgb8(209, 213, 219);

/// Success/green border (129, 199, 132)
pub const BORDER_SUCCESS: Color = Color::rgb8(129, 199, 132);

// =============================================================================
// COLORS - Text
// =============================================================================

/// Muted text - placeholder gray (150, 150, 150)
pub const TEXT_MUTED: Color = Color::rgb8(150, 150, 150);

/// Dark text (80, 80, 80)
pub const TEXT_DARK: Color = Color::rgb8(80, 80, 80);

/// Button text - Tailwind gray-700 (55, 65, 81)
pub const TEXT_BUTTON: Color = Color::rgb8(55, 65, 81);

/// Success text - green (46, 125, 50)
pub const TEXT_SUCCESS: Color = Color::rgb8(46, 125, 50);

// =============================================================================
// COLORS - Accent
// =============================================================================

/// Primary blue - Tailwind blue-500 (59, 130, 246)
pub const ACCENT_PRIMARY: Color = Color::rgb8(59, 130, 246);

/// Success green (76, 175, 80)
pub const ACCENT_SUCCESS: Color = Color::rgb8(76, 175, 80);

/// Danger red - Tailwind red-600 (220, 38, 38)
pub const ACCENT_DANGER: Color = Color::rgb8(220, 38, 38);

/// Default gray (128, 128, 128)
pub const COLOR_DEFAULT_GRAY: Color = Color::rgb8(128, 128, 128);

// =============================================================================
// TYPOGRAPHY - Font sizes
// =============================================================================

/// Title font size
pub const FONT_TITLE: f32 = 18.0;

/// Section header font size
pub const FONT_HEADER: f32 = 14.0;

/// Status message font size
pub const FONT_STATUS: f32 = 12.0;

/// Body text/labels font size
pub const FONT_BODY: f32 = 11.0;

/// Small text font size
pub const FONT_SMALL: f32 = 10.0;

/// Tiny text font size
pub const FONT_TINY: f32 = 9.0;

// =============================================================================
// SPACING
// =============================================================================

/// Standard padding
pub const PADDING_STD: f32 = 8.0;

/// Large padding (sections)
pub const PADDING_LG: f32 = 16.0;

/// Button horizontal padding
pub const PADDING_BTN_H: f32 = 12.0;

/// Button vertical padding
pub const PADDING_BTN_V: f32 = 6.0;

/// Standard gap between elements
pub const GAP_STD: f32 = 8.0;

/// Large gap
pub const GAP_LG: f32 = 16.0;

/// Standard border radius
pub const RADIUS_STD: f32 = 4.0;

/// Small border radius
pub const RADIUS_SM: f32 = 3.0;

/// Label width for form fields
pub const LABEL_WIDTH: f32 = 90.0;

/// Narrow label width
pub const LABEL_WIDTH_SM: f32 = 80.0;

/// Minimum input width
pub const INPUT_MIN_WIDTH: f32 = 100.0;

/// Standard input minimum width
pub const INPUT_MIN_WIDTH_LG: f32 = 150.0;
