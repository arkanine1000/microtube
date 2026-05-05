use ratatui::style::Color;

pub struct Preset {
    pub name: &'static str,
    pub description: &'static str,
    pub beat_freq: f32,
    pub base_freq: f32,
    pub noise_mix: f32,
    pub color: Color,
}

pub struct SequenceStep {
    pub beat_freq: f32,
    pub base_freq: f32,
    pub duration_secs: f32,
}

pub struct Sequence {
    pub name: &'static str,
    pub description: &'static str,
    pub steps: &'static [SequenceStep],
    pub total_duration_secs: f32,
}

pub static PRESETS: &[Preset] = &[
    Preset {
        name: "Deep Sleep",
        description: "Delta waves (2 Hz) - deep dreamless sleep",
        beat_freq: 2.0,
        base_freq: 180.0,
        noise_mix: 0.15,
        color: Color::Rgb(180, 120, 255), // Bright lavender - readable on dark bg
    },
    Preset {
        name: "Meditation",
        description: "Theta waves (6 Hz) - deep meditation, creativity",
        beat_freq: 6.0,
        base_freq: 200.0,
        noise_mix: 0.1,
        color: Color::Rgb(140, 100, 255), // Soft violet
    },
    Preset {
        name: "Relaxation",
        description: "Alpha waves (10 Hz) - calm, relaxed awareness",
        beat_freq: 10.0,
        base_freq: 220.0,
        noise_mix: 0.0,
        color: Color::Rgb(80, 230, 230), // Bright cyan
    },
    Preset {
        name: "Focus",
        description: "Beta waves (18 Hz) - concentration, alertness",
        beat_freq: 18.0,
        base_freq: 250.0,
        noise_mix: 0.0,
        color: Color::Rgb(80, 255, 140), // Bright mint
    },
    Preset {
        name: "Flow State",
        description: "Gamma waves (40 Hz) - peak performance, insight",
        beat_freq: 40.0,
        base_freq: 300.0,
        noise_mix: 0.0,
        color: Color::Rgb(255, 220, 80), // Warm gold
    },
];

// Static sequence steps stored as const arrays
static DEEP_FOCUS_STEPS: &[SequenceStep] = &[
    SequenceStep { beat_freq: 18.0, base_freq: 250.0, duration_secs: 600.0 },
    SequenceStep { beat_freq: 10.0, base_freq: 220.0, duration_secs: 600.0 },
    SequenceStep { beat_freq: 6.0, base_freq: 200.0, duration_secs: 300.0 },
];

static WAKE_UP_STEPS: &[SequenceStep] = &[
    SequenceStep { beat_freq: 2.0, base_freq: 180.0, duration_secs: 120.0 },
    SequenceStep { beat_freq: 6.0, base_freq: 200.0, duration_secs: 180.0 },
    SequenceStep { beat_freq: 10.0, base_freq: 220.0, duration_secs: 180.0 },
    SequenceStep { beat_freq: 15.0, base_freq: 240.0, duration_secs: 120.0 },
];

static POWER_NAP_STEPS: &[SequenceStep] = &[
    SequenceStep { beat_freq: 10.0, base_freq: 220.0, duration_secs: 300.0 },
    SequenceStep { beat_freq: 5.0, base_freq: 200.0, duration_secs: 600.0 },
    SequenceStep { beat_freq: 10.0, base_freq: 220.0, duration_secs: 180.0 },
    SequenceStep { beat_freq: 14.0, base_freq: 240.0, duration_secs: 120.0 },
];

static DEEP_MEDITATION_STEPS: &[SequenceStep] = &[
    SequenceStep { beat_freq: 10.0, base_freq: 220.0, duration_secs: 300.0 },
    SequenceStep { beat_freq: 6.0, base_freq: 200.0, duration_secs: 900.0 },
    SequenceStep { beat_freq: 4.0, base_freq: 190.0, duration_secs: 300.0 },
    SequenceStep { beat_freq: 10.0, base_freq: 220.0, duration_secs: 300.0 },
];

static PENROSE_STEPS: &[SequenceStep] = &[
    SequenceStep { beat_freq: 40.0, base_freq: 280.0, duration_secs: 300.0 },
    SequenceStep { beat_freq: 7.83, base_freq: 220.0, duration_secs: 600.0 }, // Schumann resonance
    SequenceStep { beat_freq: 40.0, base_freq: 280.0, duration_secs: 300.0 },
    SequenceStep { beat_freq: 6.0, base_freq: 200.0, duration_secs: 300.0 },
];

pub static SEQUENCES: &[Sequence] = &[
    Sequence {
        name: "Deep Focus",
        description: "25 min: Beta \u{2192} Alpha \u{2192} Theta",
        steps: DEEP_FOCUS_STEPS,
        total_duration_secs: 1500.0,
    },
    Sequence {
        name: "Wake Up",
        description: "10 min: Delta \u{2192} Theta \u{2192} Alpha \u{2192} Beta",
        steps: WAKE_UP_STEPS,
        total_duration_secs: 600.0,
    },
    Sequence {
        name: "Power Nap",
        description: "20 min: Alpha \u{2192} Theta \u{2192} Alpha \u{2192} Beta",
        steps: POWER_NAP_STEPS,
        total_duration_secs: 1200.0,
    },
    Sequence {
        name: "Deep Meditation",
        description: "30 min: Alpha \u{2192} Theta \u{2192} Deep \u{2192} Alpha",
        steps: DEEP_MEDITATION_STEPS,
        total_duration_secs: 1800.0,
    },
    Sequence {
        name: "Orch-OR",
        description: "25 min: Gamma \u{2192} Schumann \u{2192} Gamma \u{2192} Theta",
        steps: PENROSE_STEPS,
        total_duration_secs: 1500.0,
    },
];

/// Get the color for a given beat frequency band.
/// Tuned for readability on dark backgrounds (~#383c4a).
pub fn freq_color(beat_freq: f32) -> Color {
    if beat_freq < 4.0 {
        Color::Rgb(180, 120, 255) // Delta: lavender
    } else if beat_freq < 8.0 {
        Color::Rgb(140, 100, 255) // Theta: violet
    } else if beat_freq < 13.0 {
        Color::Rgb(80, 230, 230) // Alpha: cyan
    } else if beat_freq < 30.0 {
        Color::Rgb(80, 255, 140) // Beta: mint
    } else {
        Color::Rgb(255, 220, 80) // Gamma: gold
    }
}

/// Get the band name for a given beat frequency.
pub fn freq_band_name(beat_freq: f32) -> &'static str {
    if beat_freq < 4.0 {
        "\u{0394} Delta"
    } else if beat_freq < 8.0 {
        "\u{03B8} Theta"
    } else if beat_freq < 13.0 {
        "\u{03B1} Alpha"
    } else if beat_freq < 30.0 {
        "\u{03B2} Beta"
    } else {
        "\u{03B3} Gamma"
    }
}
