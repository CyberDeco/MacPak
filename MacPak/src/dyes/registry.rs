//! Color registry - single source of truth for all dye color definitions

/// Color category for UI grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCategory {
    Required,
    Recommended,
    Common,
}

/// Definition of a single color parameter
#[derive(Debug, Clone, Copy)]
pub struct ColorDef {
    /// Parameter name used in LSX files (e.g., "`Cloth_Primary`")
    pub name: &'static str,
    /// Category for UI grouping
    pub category: ColorCategory,
    /// Whether this color must be set (not left at default) for export
    pub is_required: bool,
}

/// Default hex color value (neutral gray)
pub const DEFAULT_HEX: &str = "808080";

/// Complete registry of all color parameters
/// Order determines display order within each category
pub const COLOR_REGISTRY: &[ColorDef] = &[
    // Required colors (14) - must be set before export
    ColorDef {
        name: "Cloth_Primary",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Cloth_Secondary",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Cloth_Tertiary",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Color_01",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Color_02",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Color_03",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Custom_1",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Custom_2",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Leather_Primary",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Leather_Secondary",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Leather_Tertiary",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Metal_Primary",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Metal_Secondary",
        category: ColorCategory::Required,
        is_required: true,
    },
    ColorDef {
        name: "Metal_Tertiary",
        category: ColorCategory::Required,
        is_required: true,
    },
    // Recommended colors (3) - optional but commonly needed
    ColorDef {
        name: "Accent_Color",
        category: ColorCategory::Recommended,
        is_required: false,
    },
    ColorDef {
        name: "GlowColor",
        category: ColorCategory::Recommended,
        is_required: false,
    },
    ColorDef {
        name: "GlowColour",
        category: ColorCategory::Recommended,
        is_required: false,
    },
    // Common colors (14) - used by many mods
    ColorDef {
        name: "AddedColor",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "Highlight_Color",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "BaseColor",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "InnerColor",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "OuterColor",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "PrimaryColor",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "SecondaryColor",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "TetriaryColor",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "Primary",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "Secondary",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "Tertiary",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "Primary_Color",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "Secondary_Color",
        category: ColorCategory::Common,
        is_required: false,
    },
    ColorDef {
        name: "Tertiary_Color",
        category: ColorCategory::Common,
        is_required: false,
    },
];

/// Get colors filtered by category
pub fn colors_by_category(category: ColorCategory) -> impl Iterator<Item = &'static ColorDef> {
    COLOR_REGISTRY
        .iter()
        .filter(move |c| c.category == category)
}

/// Get all required colors
pub fn required_colors() -> impl Iterator<Item = &'static ColorDef> {
    COLOR_REGISTRY.iter().filter(|c| c.is_required)
}

/// Find a color definition by name
#[must_use]
pub fn find_color(name: &str) -> Option<&'static ColorDef> {
    COLOR_REGISTRY.iter().find(|c| c.name == name)
}

/// Total number of colors
pub const COLOR_COUNT: usize = 31;
