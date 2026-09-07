//! Built-in color palettes: ten light and ten dark themes.

use crate::color::Rgba;
use crate::theme::Palette;

#[allow(clippy::too_many_arguments)]
const fn pal(
    is_dark: bool,
    background: u32,
    surface: u32,
    surface_variant: u32,
    text: u32,
    text_secondary: u32,
    text_muted: u32,
    accent: u32,
    on_accent: u32,
    outline_rgb: u32,
    outline_a: f32,
    outline_strong_a: f32,
    danger: u32,
    success: u32,
    warning: u32,
    shadow: u32,
) -> Palette {
    Palette {
        is_dark,
        background: Rgba::hex(background),
        surface: Rgba::hex(surface),
        surface_variant: Rgba::hex(surface_variant),
        text: Rgba::hex(text),
        text_secondary: Rgba::hex(text_secondary),
        text_muted: Rgba::hex(text_muted),
        accent: Rgba::hex(accent),
        on_accent: Rgba::hex(on_accent),
        outline: Rgba::hexa(outline_rgb, outline_a),
        outline_strong: Rgba::hexa(outline_rgb, outline_strong_a),
        danger: Rgba::hex(danger),
        success: Rgba::hex(success),
        warning: Rgba::hex(warning),
        shadow: Rgba::hex(shadow),
    }
}

// ---- light ---------------------------------------------------------------------------------

pub const INDIGO: Palette = pal(
    false, 0xF4F5F8, 0xFFFFFF, 0xEEF0F4, 0x171A21, 0x5B6472, 0x98A1B0, 0x5B5FEF, 0xFFFFFF,
    0x171A21, 0.10, 0.22, 0xE5484D, 0x2FA46B, 0xF0A020, 0x101420,
);
pub const OCEAN: Palette = pal(
    false, 0xF2F7FA, 0xFFFFFF, 0xE3EEF5, 0x0F2A3A, 0x4A6478, 0x8FA5B5, 0x0284C7, 0xFFFFFF,
    0x0F2A3A, 0.10, 0.22, 0xDC2626, 0x059669, 0xD97706, 0x0A1E2C,
);
pub const FOREST: Palette = pal(
    false, 0xF3F7F2, 0xFFFFFF, 0xE4EEE1, 0x14261A, 0x4E6656, 0x90A594, 0x16A34A, 0xFFFFFF,
    0x14261A, 0.10, 0.22, 0xDC2626, 0x15803D, 0xCA8A04, 0x0B1A10,
);
pub const SUNSET: Palette = pal(
    false, 0xFFF6F0, 0xFFFFFF, 0xFFE9DC, 0x2E1A12, 0x6F5246, 0xB39A8F, 0xEA580C, 0xFFFFFF,
    0x2E1A12, 0.10, 0.22, 0xDC2626, 0x16A34A, 0xD97706, 0x2A1208,
);
pub const ROSE: Palette = pal(
    false, 0xFDF2F5, 0xFFFFFF, 0xFBE3EA, 0x2C1220, 0x6E4A5A, 0xB08E9C, 0xE11D48, 0xFFFFFF,
    0x2C1220, 0.10, 0.22, 0xB91C1C, 0x16A34A, 0xD97706, 0x2A0F1A,
);
pub const LAVENDER: Palette = pal(
    false, 0xF6F3FB, 0xFFFFFF, 0xECE6F7, 0x241A36, 0x5E5275, 0x9C91B3, 0x7C3AED, 0xFFFFFF,
    0x241A36, 0.10, 0.22, 0xDC2626, 0x16A34A, 0xD97706, 0x1A1030,
);
pub const SAND: Palette = pal(
    false, 0xF9F5EC, 0xFFFDF8, 0xF0E8D8, 0x2A241A, 0x6B6151, 0xA69B87, 0xB45309, 0xFFFFFF,
    0x2A241A, 0.10, 0.22, 0xB91C1C, 0x4D7C0F, 0xA16207, 0x221A0E,
);
pub const SLATE: Palette = pal(
    false, 0xF1F3F5, 0xFFFFFF, 0xE2E6EA, 0x111827, 0x4B5563, 0x9CA3AF, 0x334155, 0xFFFFFF,
    0x111827, 0.10, 0.22, 0xDC2626, 0x16A34A, 0xD97706, 0x0F172A,
);
pub const MINT: Palette = pal(
    false, 0xF0FAF7, 0xFFFFFF, 0xDDF3EC, 0x0F2A24, 0x476B62, 0x8FB0A6, 0x0D9488, 0xFFFFFF,
    0x0F2A24, 0.10, 0.22, 0xDC2626, 0x059669, 0xD97706, 0x082019,
);
pub const MONO: Palette = pal(
    false, 0xFBFBFA, 0xFFFFFF, 0xF0F0EE, 0x1A1A1A, 0x555555, 0x9A9A9A, 0x111111, 0xFFFFFF,
    0x1A1A1A, 0.10, 0.24, 0xC62828, 0x2E7D32, 0xB26A00, 0x000000,
);

// ---- dark ----------------------------------------------------------------------------------

pub const MIDNIGHT: Palette = pal(
    true, 0x0E1014, 0x181B21, 0x23272F, 0xEDEFF3, 0xA3AAB8, 0x6C7482, 0x7B7FF7, 0xFFFFFF, 0xFFFFFF,
    0.10, 0.24, 0xF26B6F, 0x4CC38A, 0xF8B84E, 0x000000,
);
pub const NORD: Palette = pal(
    true, 0x2E3440, 0x3B4252, 0x434C5E, 0xECEFF4, 0xD8DEE9, 0x9AA5B8, 0x88C0D0, 0x2E3440, 0xECEFF4,
    0.10, 0.24, 0xBF616A, 0xA3BE8C, 0xEBCB8B, 0x000000,
);
pub const DRACULA: Palette = pal(
    true, 0x282A36, 0x2F3140, 0x44475A, 0xF8F8F2, 0xBFC3D9, 0x6272A4, 0xBD93F9, 0x1E1F29, 0xF8F8F2,
    0.10, 0.24, 0xFF5555, 0x50FA7B, 0xF1FA8C, 0x000000,
);
pub const OCEAN_DARK: Palette = pal(
    true, 0x0B1620, 0x122130, 0x1B2E40, 0xE6F0F7, 0x9FB6C8, 0x5F7A8F, 0x38BDF8, 0x082032, 0xE6F0F7,
    0.10, 0.24, 0xF87171, 0x34D399, 0xFBBF24, 0x000000,
);
pub const FOREST_DARK: Palette = pal(
    true, 0x0E1512, 0x16211B, 0x1F2E25, 0xE7F0EA, 0xA3B8AA, 0x667A6D, 0x4ADE80, 0x052E16, 0xE7F0EA,
    0.10, 0.24, 0xF87171, 0x86EFAC, 0xFACC15, 0x000000,
);
pub const EMBER: Palette = pal(
    true, 0x171210, 0x221A17, 0x2E2320, 0xF5EDE8, 0xBFA9A0, 0x7D6A62, 0xFB923C, 0x2A1206, 0xF5EDE8,
    0.10, 0.24, 0xF87171, 0x4ADE80, 0xFBBF24, 0x000000,
);
pub const ROSE_DARK: Palette = pal(
    true, 0x1A1116, 0x241820, 0x31212B, 0xF6EBF0, 0xC4A9B7, 0x7F6A75, 0xFB7185, 0x2D0A12, 0xF6EBF0,
    0.10, 0.24, 0xF87171, 0x4ADE80, 0xFBBF24, 0x000000,
);
pub const AMETHYST: Palette = pal(
    true, 0x130F1D, 0x1C1629, 0x281F3A, 0xEFEAF7, 0xB4A8CB, 0x736A88, 0xA78BFA, 0x1E1033, 0xEFEAF7,
    0.10, 0.24, 0xF87171, 0x4ADE80, 0xFBBF24, 0x000000,
);
pub const GRAPHITE: Palette = pal(
    true, 0x1B1B1D, 0x242427, 0x2E2E32, 0xEDEDEF, 0xA9A9AF, 0x6F6F76, 0xE5E7EB, 0x111113, 0xEDEDEF,
    0.10, 0.24, 0xF87171, 0x4ADE80, 0xFBBF24, 0x000000,
);
pub const CARBON: Palette = pal(
    true, 0x000000, 0x0E0E0E, 0x1A1A1A, 0xF4F4F5, 0xA1A1AA, 0x62626B, 0x22D3EE, 0x04262B, 0xF4F4F5,
    0.12, 0.26, 0xF87171, 0x34D399, 0xFBBF24, 0x000000,
);

/// Every built-in palette, light ones first.
pub const BUILTIN: &[(&str, Palette)] = &[
    ("Indigo", INDIGO),
    ("Ocean", OCEAN),
    ("Forest", FOREST),
    ("Sunset", SUNSET),
    ("Rose", ROSE),
    ("Lavender", LAVENDER),
    ("Sand", SAND),
    ("Slate", SLATE),
    ("Mint", MINT),
    ("Mono", MONO),
    ("Midnight", MIDNIGHT),
    ("Nord", NORD),
    ("Dracula", DRACULA),
    ("Ocean Dark", OCEAN_DARK),
    ("Forest Dark", FOREST_DARK),
    ("Ember", EMBER),
    ("Rose Dark", ROSE_DARK),
    ("Amethyst", AMETHYST),
    ("Graphite", GRAPHITE),
    ("Carbon", CARBON),
];

/// Look up a built-in palette by name (case-insensitive).
pub fn builtin(name: &str) -> Option<Palette> {
    let want = name.trim().to_ascii_lowercase();
    BUILTIN
        .iter()
        .find(|(n, _)| n.to_ascii_lowercase() == want)
        .map(|(_, p)| *p)
}
