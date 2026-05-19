//! Visual theme for MicroTube — palette, accent ramps, and color math.
//!
//! All color decisions live here so the aesthetic is tunable from one place.
//! Code outside this module should refer to colors by semantic name rather
//! than reaching for `Color::Rgb(...)` directly.

use ratatui::style::Color;

// --- Background gradient stops (deep-space → near-horizon) ----------------

pub const SPACE_DEEP: Color = Color::Rgb(3, 5, 12);
pub const SPACE_MID: Color = Color::Rgb(9, 12, 26);
pub const SPACE_NEAR: Color = Color::Rgb(14, 18, 34);
pub const HORIZON: Color = Color::Rgb(4, 9, 18);

// Panel surfaces sit on top of the backdrop; the shell-level background is
// SPACE_DEEP so panels read as a slightly raised layer.
pub const PANEL: Color = Color::Rgb(11, 14, 26);
pub const PANEL_RAISED: Color = Color::Rgb(16, 20, 34);
pub const SHADOW: Color = Color::Rgb(2, 3, 6);

// --- Neutral ink ramp (typography) ----------------------------------------
// Stepped luminance — anchored to a cool, slightly violet undertone so the
// chrome feels like part of the deep-space palette, not a separate grey.

pub const INK_0: Color = Color::Rgb(238, 242, 250); // brightest — primary readout
pub const INK_1: Color = Color::Rgb(206, 212, 228); // strong — secondary readout
pub const INK_2: Color = Color::Rgb(158, 166, 188); // calm body text
pub const INK_3: Color = Color::Rgb(118, 126, 150); // dim labels
pub const INK_4: Color = Color::Rgb(82, 90, 114); // very dim — separators, frame
pub const INK_5: Color = Color::Rgb(54, 60, 80); // near-background — inactive chrome

// --- Semantic accents (per-parameter, per-domain) -------------------------
// One struct so the scattered `Color::Rgb(...)` per-parameter colors are
// editable together. Values match the previous ui.rs literals 1:1 in Phase A.

pub struct SemanticPalette {
    pub base: Color,    // base carrier frequency
    pub gain: Color,    // volume
    pub warm: Color,    // harmonics / warmth
    pub life: Color,    // emergence
    pub drift: Color,   // shepard intensity
    pub d_base: Color,  // shepard base frequency
    pub mist: Color,    // noise / mist
    pub timbre: Color,  // timbre cycle
    pub savable: Color, // save-preset accent
    pub program: Color, // sequence / program
}

pub const SEMANTIC: SemanticPalette = SemanticPalette {
    base: Color::Rgb(80, 220, 245),
    gain: Color::Rgb(84, 240, 150),
    warm: Color::Rgb(250, 210, 92),
    life: Color::Rgb(210, 145, 255),
    drift: Color::Rgb(255, 170, 110),
    d_base: Color::Rgb(255, 205, 135),
    mist: Color::Rgb(120, 170, 255),
    timbre: Color::Rgb(170, 255, 120),
    savable: Color::Rgb(84, 240, 150),
    program: Color::Rgb(210, 145, 255),
};

// --- Accent ramp ----------------------------------------------------------
// Derived from the band-keyed accent (presets::freq_color). One struct so
// renderers can pick the right luminance level without doing color math.

#[derive(Clone, Copy)]
pub struct AccentRamp {
    /// The base accent — what the audio is "tuned to".
    pub base: Color,
    /// Brighter than base — leading edge of meters, active text.
    pub glow: Color,
    /// Lower-luminance variant — quiet rails, secondary fill.
    pub dim: Color,
    /// Very low-luminance — backdrop wash, inactive frames.
    pub veil: Color,
    /// A shifted hue — used for the right-ear waveform, etc.
    pub shifted: Color,
}

impl AccentRamp {
    pub fn from_accent(base: Color) -> Self {
        Self {
            base,
            glow: brighten(base, 1.18),
            dim: dim(base, 0.55),
            veil: dim(base, 0.22),
            shifted: shift(base, 34, 8, 44),
        }
    }
}

// --- Color math -----------------------------------------------------------

/// Multiply each RGB channel by `factor`, clamped to [0, 255].
/// Non-Rgb colors pass through unchanged.
pub fn dim(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 * factor).clamp(0.0, 255.0) as u8,
            (g as f32 * factor).clamp(0.0, 255.0) as u8,
            (b as f32 * factor).clamp(0.0, 255.0) as u8,
        ),
        other => other,
    }
}

/// Push each channel toward white by `(1 - factor)` for factor > 1,
/// or behave like `dim` for factor <= 1.
pub fn brighten(color: Color, factor: f32) -> Color {
    if factor <= 1.0 {
        return dim(color, factor);
    }
    let t = (factor - 1.0).clamp(0.0, 1.0);
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            (r as f32 + (255.0 - r as f32) * t) as u8,
            (g as f32 + (255.0 - g as f32) * t) as u8,
            (b as f32 + (255.0 - b as f32) * t) as u8,
        ),
        other => other,
    }
}

/// Saturating add per channel — used to nudge a color toward a sibling hue.
pub fn shift(color: Color, dr: u8, dg: u8, db: u8) -> Color {
    match color {
        Color::Rgb(r, g, b) => Color::Rgb(
            r.saturating_add(dr),
            g.saturating_add(dg),
            b.saturating_add(db),
        ),
        other => other,
    }
}

/// Linear mix between two colors. `t = 0` → start, `t = 1` → end.
pub fn mix(start: Color, end: Color, t: f32) -> Color {
    let (sr, sg, sb) = rgb(start);
    let (er, eg, eb) = rgb(end);
    let t = t.clamp(0.0, 1.0);
    Color::Rgb(
        (sr as f32 + (er as f32 - sr as f32) * t) as u8,
        (sg as f32 + (eg as f32 - sg as f32) * t) as u8,
        (sb as f32 + (eb as f32 - sb as f32) * t) as u8,
    )
}

pub fn rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128),
    }
}

/// A slow breathing modulation of `accent` — 0.6..0.84 luminance over ~8s.
pub fn breathing(accent: Color, elapsed: f64) -> Color {
    let pulse = 0.62 + 0.22 * (elapsed * 0.8).sin().abs() as f32;
    dim(accent, pulse)
}
