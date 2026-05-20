use ratatui::style::Color;

use crate::app::{MistType, Timbre, VizMode};
use crate::emergence::SpawnMode;
use crate::shepard::Direction;

pub struct Preset {
    pub name: &'static str,
    pub description: &'static str,
    pub beat_freq: f32,
    pub base_freq: f32,
    pub noise_mix: f32,
    pub color: Color,
}

/// One entry in a timed sequence.
///
/// `beat_freq`, `base_freq`, and `duration_secs` are always automated.
/// Every other field is optional: `Some(value)` automates it, `None` leaves
/// the parameter under manual control. Continuous fields are linearly
/// interpolated toward the next step's value over the step's duration;
/// discrete fields (timbre, mist, direction, spawn mode, viz mode) snap
/// at step entry — the audio thread's 50 ms exponential smoothing absorbs
/// the boundary.
pub struct SequenceStep {
    pub name: Option<&'static str>,
    pub beat_freq: f32,
    pub base_freq: f32,
    pub duration_secs: f32,
    pub volume: Option<f32>,
    pub noise_level: Option<f32>,
    pub harmonics: Option<f32>,
    pub emergence: Option<f32>,
    pub shepard: Option<f32>,
    pub timbre: Option<Timbre>,
    pub mist_type: Option<MistType>,
    pub shepard_direction: Option<Direction>,
    pub spawn_mode: Option<SpawnMode>,
    pub viz_mode: Option<VizMode>,
}

pub struct Sequence {
    pub name: &'static str,
    pub description: &'static str,
    pub steps: &'static [SequenceStep],
    pub total_duration_secs: f32,
}

/// Legacy two-frequency step. Leaves all other parameters under manual control.
const fn legacy_step(beat_freq: f32, base_freq: f32, duration_secs: f32) -> SequenceStep {
    SequenceStep {
        name: None,
        beat_freq,
        base_freq,
        duration_secs,
        volume: None,
        noise_level: None,
        harmonics: None,
        emergence: None,
        shepard: None,
        timbre: None,
        mist_type: None,
        shepard_direction: None,
        spawn_mode: None,
        viz_mode: None,
    }
}

/// Fully-automated step for narrative sequences. Every audible & visible
/// parameter is set; the listener just rides.
#[allow(clippy::too_many_arguments)]
const fn epoch(
    name: &'static str,
    beat_freq: f32,
    base_freq: f32,
    duration_secs: f32,
    volume: f32,
    noise_level: f32,
    harmonics: f32,
    emergence: f32,
    shepard: f32,
    timbre: Timbre,
    mist_type: MistType,
    shepard_direction: Direction,
    spawn_mode: SpawnMode,
    viz_mode: VizMode,
) -> SequenceStep {
    SequenceStep {
        name: Some(name),
        beat_freq,
        base_freq,
        duration_secs,
        volume: Some(volume),
        noise_level: Some(noise_level),
        harmonics: Some(harmonics),
        emergence: Some(emergence),
        shepard: Some(shepard),
        timbre: Some(timbre),
        mist_type: Some(mist_type),
        shepard_direction: Some(shepard_direction),
        spawn_mode: Some(spawn_mode),
        viz_mode: Some(viz_mode),
    }
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
    legacy_step(18.0, 250.0, 600.0),
    legacy_step(10.0, 220.0, 600.0),
    legacy_step(6.0, 200.0, 300.0),
];

static WAKE_UP_STEPS: &[SequenceStep] = &[
    legacy_step(2.0, 180.0, 120.0),
    legacy_step(6.0, 200.0, 180.0),
    legacy_step(10.0, 220.0, 180.0),
    legacy_step(15.0, 240.0, 120.0),
];

static POWER_NAP_STEPS: &[SequenceStep] = &[
    legacy_step(10.0, 220.0, 300.0),
    legacy_step(5.0, 200.0, 600.0),
    legacy_step(10.0, 220.0, 180.0),
    legacy_step(14.0, 240.0, 120.0),
];

static DEEP_MEDITATION_STEPS: &[SequenceStep] = &[
    legacy_step(10.0, 220.0, 300.0),
    legacy_step(6.0, 200.0, 900.0),
    legacy_step(4.0, 190.0, 300.0),
    legacy_step(10.0, 220.0, 300.0),
];

static PENROSE_STEPS: &[SequenceStep] = &[
    legacy_step(40.0, 280.0, 300.0),
    legacy_step(7.83, 220.0, 600.0), // Schumann resonance
    legacy_step(40.0, 280.0, 300.0),
    legacy_step(6.0, 200.0, 300.0),
];

// Journey Through the Cosmos
//
// A 25½-minute strange loop. We begin inside a microtubule (Penrose-Hameroff
// Orch-OR) and zoom outward through nested scales of consciousness and physics
// — synapse, brain, body, Schumann cavity, lunar tide, solar wind, stellar
// fields, the galactic disc, the cosmic web, the CMB, and the primordial
// singularity. Then the loop closes: we find ourselves back inside the
// microtubule. The cosmos *is* the microtubule.
//
// Step durations follow the Fibonacci sequence (21, 34, 55, 89, 144, 233, 377)
// rising to Solar Wind (the still point, 377 s) and descending symmetrically
// — a quasicrystal in time. Beat frequency traces a U: gamma at the
// microtubule, delta at the stellar dream-depth, gamma again at strange-loop
// closure. Base frequency descends monotonically from 432 Hz (high quantum
// coherence) to 55 Hz (cosmic bass at the singularity); the strange-loop
// step's lerp from 55 → 432 over 34 s *is* the loop closing.
//
// Total: 1 529 s ≈ 25 min 29 s.
#[rustfmt::skip]
static JOURNEY_THROUGH_COSMOS_STEPS: &[SequenceStep] = &[
    //    name                       beat   base     dur    vol  noise  harm  emer  shep  timbre         mist              direction         spawn               viz
    epoch("Microtubule",          40.0, 432.00,  21.0,  0.40, 0.10, 0.85, 0.55, 0.00, Timbre::Bell,  MistType::Velvet, Direction::Up,    SpawnMode::Penrose, VizMode::Emergence),
    epoch("Synapse",              22.0, 384.00,  34.0,  0.50, 0.15, 0.70, 0.60, 0.10, Timbre::Bell,  MistType::Velvet, Direction::Up,    SpawnMode::Canon,   VizMode::Waveform),
    epoch("Neural Awareness",     14.0, 320.00,  55.0,  0.60, 0.20, 0.55, 0.45, 0.20, Timbre::Flute, MistType::Pink,   Direction::Up,    SpawnMode::Canon,   VizMode::Harmonics),
    epoch("Body",                 10.0, 256.00,  89.0,  0.65, 0.25, 0.50, 0.35, 0.25, Timbre::Flute, MistType::Pink,   Direction::Up,    SpawnMode::Canon,   VizMode::Envelope),
    epoch("Earth \u{00B7} Schumann", 7.83, 196.00, 144.0, 0.70, 0.40, 0.45, 0.30, 0.30, Timbre::Organ, MistType::Brown,  Direction::Up,    SpawnMode::Canon,   VizMode::Envelope),
    epoch("Lunar Tide",            5.0, 165.00, 233.0,  0.70, 0.35, 0.50, 0.40, 0.45, Timbre::Organ, MistType::Pink,   Direction::Up,    SpawnMode::Canon,   VizMode::Penrose),
    epoch("Solar Wind",            3.0, 130.81, 377.0,  0.70, 0.30, 0.60, 0.55, 0.60, Timbre::Organ, MistType::Pink,   Direction::Up,    SpawnMode::Penrose, VizMode::Penrose),
    epoch("Stellar Bells",         2.0, 110.00, 233.0,  0.65, 0.25, 0.70, 0.70, 0.70, Timbre::Bell,  MistType::Blue,   Direction::Up,    SpawnMode::Penrose, VizMode::Emergence),
    epoch("Galactic",              4.0,  87.31, 144.0,  0.60, 0.30, 0.80, 0.80, 0.80, Timbre::Bell,  MistType::Blue,   Direction::Up,    SpawnMode::Penrose, VizMode::Emergence),
    epoch("Cosmic Web",            8.0,  73.42,  89.0,  0.55, 0.45, 0.85, 0.90, 0.85, Timbre::Saw,   MistType::White,  Direction::Up,    SpawnMode::Penrose, VizMode::Emergence),
    epoch("Background Radiation",18.0,  65.41,  55.0,  0.45, 0.75, 0.50, 0.55, 0.70, Timbre::Saw,   MistType::White,  Direction::Up,    SpawnMode::Penrose, VizMode::Spectrum),
    epoch("Singularity",          60.0,  55.00,  34.0,  0.25, 0.85, 0.35, 0.25, 0.40, Timbre::Saw,   MistType::Velvet, Direction::Down,  SpawnMode::Penrose, VizMode::Spectrum),
    epoch("Strange Loop",         40.0, 432.00,  21.0,  0.50, 0.10, 0.85, 0.55, 0.00, Timbre::Bell,  MistType::Velvet, Direction::Up,    SpawnMode::Penrose, VizMode::Emergence),
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
    Sequence {
        name: "Journey Through the Cosmos",
        description: "25 min: Microtubule \u{2192} Cosmos \u{2192} Strange Loop",
        steps: JOURNEY_THROUGH_COSMOS_STEPS,
        total_duration_secs: 1529.0,
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
