//! Emergence Engine
//!
//! A generative audio system inspired by Conway's Game of Life, Bach canons,
//! and Hofstadter's strange loops. Voices emerge from simple rules, interact
//! through harmonic relationships, and decay back into silence.
//!
//! Rules of the system:
//! - Voices spawn at harmonic/golden-ratio intervals of existing voices
//! - Constructive interference strengthens voices; dissonance causes decay
//! - A "canon" pattern repeats at different pitches and time offsets
//! - Total energy is conserved: as new voices emerge, others must fade
//! - The system tends toward consonance but is perturbed by randomness

use std::f64::consts::{PI, TAU};

use crate::penrose::{self, PenroseWalk, Tile};

/// Golden ratio - the fundamental proportion of emergence
const PHI: f64 = 1.618033988749895;

/// Maximum simultaneous voices
const MAX_VOICES: usize = 12;

/// Simple ratios used by [`consonance_score`]. Each entry is `(ratio, peak_score)`:
/// a perfect match scores `peak_score`, partial matches drop linearly within
/// a 0.1 log-distance neighborhood.
pub const CONSONANCE_RATIOS: &[(f64, f64)] = &[
    (1.0, 1.0),         // Unison
    (2.0, 0.95),        // Octave
    (3.0 / 2.0, 0.9),   // Fifth
    (4.0 / 3.0, 0.85),  // Fourth
    (5.0 / 4.0, 0.8),   // Major third
    (6.0 / 5.0, 0.75),  // Minor third
    (PHI, 0.7),         // Golden (special)
];

/// Score how consonant a frequency ratio is (0.0 = dissonant, 1.0 = perfect).
///
/// Each peak in [`CONSONANCE_RATIOS`] contributes a triangular kernel of
/// half-width 0.1 in log-ratio space; the highest contribution wins. Used by
/// the audio engine to weight voice lifetime, and by the Knowledge-tab
/// playground to plot the landscape — single source of truth for both.
pub fn consonance_score(ratio: f64) -> f64 {
    let mut best_score = 0.0;
    for &(r, score) in CONSONANCE_RATIOS {
        let distance = ((ratio / r).ln()).abs();
        if distance < 0.1 {
            let proximity = 1.0 - (distance / 0.1);
            let s = score * proximity;
            if s > best_score {
                best_score = s;
            }
        }
    }
    best_score
}

/// Harmonic ratios that voices can spawn at (just intonation + golden)
const SPAWN_RATIOS: &[f64] = &[
    1.0 / PHI, // Sub-golden
    0.5,       // Octave below
    2.0 / 3.0, // Perfect fifth below
    3.0 / 4.0, // Perfect fourth below
    1.0,       // Unison (canon)
    5.0 / 4.0, // Major third
    4.0 / 3.0, // Perfect fourth
    3.0 / 2.0, // Perfect fifth
    PHI,       // Golden ratio
    2.0,       // Octave
    PHI * PHI, // Golden squared (approaching next octave)
];

/// A single emergent voice in the system.
#[derive(Clone)]
pub struct Voice {
    pub freq_ratio: f64, // Ratio to base frequency
    pub amplitude: f64,  // Current amplitude (0.0 - 1.0)
    pub phase: f64,      // Phase accumulator
    pub age: f64,        // Seconds since birth
    pub lifetime: f64,   // Total expected lifetime
    pub generation: u8,  // How many ancestors spawned this voice
    pub pan: f64,        // -1.0 (left) to 1.0 (right)
    alive: bool,
}

impl Voice {
    fn new(freq_ratio: f64, lifetime: f64, generation: u8, pan: f64) -> Self {
        Self {
            freq_ratio,
            amplitude: 0.0, // Starts silent, fades in
            phase: 0.0,
            age: 0.0,
            lifetime,
            generation,
            pan,
            alive: true,
        }
    }

    /// Envelope: smooth attack/sustain/release
    fn envelope(&self) -> f64 {
        let t = self.age / self.lifetime;
        if t >= 1.0 {
            return 0.0;
        }
        // Bell curve envelope: sin^2 for smooth in/out
        let env = (PI * t).sin();
        env * env
    }

    fn is_alive(&self) -> bool {
        self.alive && self.age < self.lifetime
    }
}

/// How spawn ratios are chosen.
///
/// `Canon` follows the original repeating ratio table (a Bach-style fugue).
/// `Penrose` reads ratios from a Fibonacci-word walk — the 1D quasicrystal
/// that traces a Conway worm through a Penrose P3 tiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnMode {
    Canon,
    Penrose,
}

impl SpawnMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Canon => "canon",
            Self::Penrose => "penrose",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Canon => Self::Penrose,
            Self::Penrose => Self::Canon,
        }
    }

    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Penrose,
            _ => Self::Canon,
        }
    }
}

/// State shared with the visualization (snapshot of current voices).
#[derive(Clone)]
pub struct EmergenceSnapshot {
    pub voices: Vec<VoiceInfo>,
    pub total_energy: f32,
    pub generation_count: u8,
    pub epoch: u32, // How many spawn cycles have occurred
    pub spawn_mode: SpawnMode,
    /// Recent Fibonacci-word tiles (newest last). Empty when not in Penrose mode.
    pub recent_tiles: Vec<Tile>,
    pub walk_position: usize,
}

impl EmergenceSnapshot {
    pub fn empty() -> Self {
        Self {
            voices: Vec::new(),
            total_energy: 0.0,
            generation_count: 0,
            epoch: 0,
            spawn_mode: SpawnMode::Canon,
            recent_tiles: Vec::new(),
            walk_position: 0,
        }
    }
}

#[derive(Clone)]
pub struct VoiceInfo {
    pub freq_ratio: f32,
    pub amplitude: f32,
    pub age_normalized: f32, // 0.0 to 1.0
    pub generation: u8,
    pub pan: f32,
}

/// The emergence engine - runs in the audio thread.
pub struct EmergenceEngine {
    voices: Vec<Voice>,
    rng: u64,
    time: f64,
    sample_rate: f64,
    spawn_timer: f64,
    spawn_interval: f64, // Seconds between potential spawns
    epoch: u32,
    /// Canon pattern: a sequence of ratio indices that repeats
    canon_pattern: Vec<usize>,
    canon_position: usize,
    canon_offset: f64, // Pitch offset for canon repetition
    spawn_mode: SpawnMode,
    penrose_walk: PenroseWalk,
}

impl EmergenceEngine {
    pub fn new(sample_rate: f64) -> Self {
        let mut engine = Self {
            voices: Vec::with_capacity(MAX_VOICES),
            rng: 0xBAC0_CAFE_1685_1750,
            time: 0.0,
            sample_rate,
            spawn_timer: 0.0,
            spawn_interval: 3.0, // New voice every ~3 seconds
            epoch: 0,
            canon_pattern: vec![7, 5, 6, 8, 4, 9, 3, 10], // Fifths, thirds, golden
            canon_position: 0,
            canon_offset: 1.0,
            spawn_mode: SpawnMode::Canon,
            penrose_walk: PenroseWalk::new(),
        };
        // Seed with a root voice
        engine.voices.push(Voice::new(1.0, 8.0, 0, 0.0));
        engine
    }

    pub fn set_spawn_mode(&mut self, mode: SpawnMode) {
        self.spawn_mode = mode;
    }

    pub fn spawn_mode(&self) -> SpawnMode {
        self.spawn_mode
    }

    fn xorshift(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng as f64) / (u64::MAX as f64)
    }

    /// Advance the engine by one sample period and return (left, right) contribution.
    pub fn process(&mut self, base_freq: f64, intensity: f64) -> (f64, f64) {
        let dt = 1.0 / self.sample_rate;
        self.time += dt;
        self.spawn_timer += dt;

        // Spawn logic - check if it's time for a new voice
        let adjusted_interval = self.spawn_interval / (0.5 + intensity * 1.5);
        if self.spawn_timer >= adjusted_interval {
            self.spawn_timer = 0.0;
            self.try_spawn(intensity);
        }

        // Generate audio from all living voices
        let mut sum_l = 0.0;
        let mut sum_r = 0.0;
        let mut total_energy = 0.0;

        for voice in &mut self.voices {
            if !voice.is_alive() {
                continue;
            }

            voice.age += dt;
            let env = voice.envelope();
            voice.amplitude = env * intensity;

            if voice.amplitude < 0.001 {
                continue;
            }

            // Phase accumulator for this voice
            let freq = base_freq * voice.freq_ratio;
            voice.phase += freq / self.sample_rate;
            voice.phase -= voice.phase.floor();

            // Slightly warm tone (fundamental + soft 2nd harmonic)
            let sample = (TAU * voice.phase).sin()
                + 0.2 * (TAU * voice.phase * 2.0).sin()
                + 0.05 * (TAU * voice.phase * 3.0).sin();

            let amp = voice.amplitude * 0.15; // Scale down - these are background voices

            // Stereo panning (constant power)
            let pan_angle = (voice.pan + 1.0) * 0.25 * PI; // 0 to PI/2
            let gain_l = pan_angle.cos();
            let gain_r = pan_angle.sin();

            sum_l += sample * amp * gain_l;
            sum_r += sample * amp * gain_r;
            total_energy += voice.amplitude;
        }

        // Energy conservation: normalize if too loud
        if total_energy > 2.0 {
            let scale = 2.0 / total_energy;
            sum_l *= scale;
            sum_r *= scale;
        }

        // Remove dead voices
        self.voices.retain(|v| v.is_alive());

        (sum_l, sum_r)
    }

    fn try_spawn(&mut self, intensity: f64) {
        if self.voices.len() >= MAX_VOICES {
            // Kill the oldest voice to make room (conservation)
            if let Some(oldest) = self.voices.iter_mut().min_by(|a, b| {
                let a_remaining = a.lifetime - a.age;
                let b_remaining = b.lifetime - b.age;
                a_remaining.partial_cmp(&b_remaining).unwrap()
            }) {
                oldest.alive = false;
            }
            self.voices.retain(|v| v.alive);
        }

        if self.voices.is_empty() {
            // Always maintain a root
            self.voices.push(Voice::new(1.0, 8.0, 0, 0.0));
            return;
        }

        self.epoch += 1;

        // Choose the base ratio according to the active spawn mode.
        let base_ratio = match self.spawn_mode {
            SpawnMode::Canon => {
                let idx = self.canon_pattern[self.canon_position % self.canon_pattern.len()];
                self.canon_position += 1;
                SPAWN_RATIOS[idx % SPAWN_RATIOS.len()]
            }
            SpawnMode::Penrose => {
                // Each spawn advances the Conway worm by one rhomb; the
                // (previous, current) tile pair selects the harmonic move.
                let (prev, curr) = self.penrose_walk.step();
                self.canon_position += 1;
                penrose::pair_ratio(prev, curr)
            }
        };

        // Every 8 spawns, shift the canon offset (like a fugue answer).
        // Penrose mode reuses the same transposition cadence so the walk
        // sweeps across registers without losing its quasicrystal cadence.
        if self.canon_position.is_multiple_of(8) {
            self.canon_offset = SPAWN_RATIOS[(self.epoch as usize / 8) % SPAWN_RATIOS.len()];
        }

        // Apply canon offset and slight random mutation
        let mutation = 1.0 + (self.xorshift() - 0.5) * 0.02 * intensity;
        let final_ratio = base_ratio * self.canon_offset * mutation;

        // Clamp to musically useful range (0.25x to 4x base)
        let final_ratio = final_ratio.clamp(0.25, 4.0);

        // Lifetime varies: longer for consonant ratios, shorter for dissonant
        let consonance = self.consonance_score(final_ratio);
        let base_lifetime = 4.0 + consonance * 8.0; // 4-12 seconds
        let lifetime = base_lifetime * (0.8 + self.xorshift() * 0.4);

        // Generation: child of the strongest current voice
        let parent_gen = self
            .voices
            .iter()
            .max_by(|a, b| a.amplitude.partial_cmp(&b.amplitude).unwrap())
            .map(|v| v.generation)
            .unwrap_or(0);
        let generation = parent_gen.saturating_add(1).min(7);

        // Pan: spread voices across the stereo field
        let pan = (self.xorshift() * 2.0 - 1.0) * 0.7; // -0.7 to 0.7

        self.voices
            .push(Voice::new(final_ratio, lifetime, generation, pan));
    }

    /// Score how consonant a frequency ratio is (0.0 = dissonant, 1.0 = perfect).
    /// Uses the concept of harmonic distance.
    fn consonance_score(&self, ratio: f64) -> f64 {
        consonance_score(ratio)
    }

    /// Take a snapshot of current state for visualization.
    pub fn snapshot(&self) -> EmergenceSnapshot {
        let voices: Vec<VoiceInfo> = self
            .voices
            .iter()
            .filter(|v| v.is_alive() && v.amplitude > 0.001)
            .map(|v| VoiceInfo {
                freq_ratio: v.freq_ratio as f32,
                amplitude: v.amplitude as f32,
                age_normalized: (v.age / v.lifetime) as f32,
                generation: v.generation,
                pan: v.pan as f32,
            })
            .collect();

        let total_energy = voices.iter().map(|v| v.amplitude).sum();
        let generation_count = voices.iter().map(|v| v.generation).max().unwrap_or(0);

        let recent_tiles = if self.spawn_mode == SpawnMode::Penrose {
            self.penrose_walk.recent(24)
        } else {
            Vec::new()
        };

        EmergenceSnapshot {
            voices,
            total_energy,
            generation_count,
            epoch: self.epoch,
            spawn_mode: self.spawn_mode,
            recent_tiles,
            walk_position: self.penrose_walk.position(),
        }
    }
}
