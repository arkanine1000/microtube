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
use crate::sea_organ::{self, SeaOrganState, smoothstep};

/// Golden ratio - the fundamental proportion of emergence
const PHI: f64 = 1.618033988749895;

/// Maximum simultaneous voices
const MAX_VOICES: usize = 12;
/// Sea Organ storms layer many pipes at once; it gets a larger budget.
const SEA_MAX_VOICES: usize = 20;

/// Built-in binaural spatialization constants for emergence voices.
///
/// This is an HRTF-style approximation, not measured HRIR convolution: short
/// interaural delay, constant-power level difference, and far-ear softening.
const HRTF_DELAY_BUFFER_LEN: usize = 256;
const HRTF_MAX_ITD_SECONDS: f64 = 0.00065;
const HRTF_SHADOW_ATTENUATION: f64 = 0.28;
const HRTF_NEAR_CUTOFF_HZ: f64 = 11_000.0;
const HRTF_FAR_CUTOFF_HZ: f64 = 2_400.0;

/// Simple ratios used by [`consonance_score`]. Each entry is `(ratio, peak_score)`:
/// a perfect match scores `peak_score`, partial matches drop linearly within
/// a 0.1 log-distance neighborhood.
pub const CONSONANCE_RATIOS: &[(f64, f64)] = &[
    (1.0, 1.0),          // Unison
    (2.0, 0.95),         // Octave
    (3.0 / 2.0, 0.9),    // Fifth
    (4.0 / 3.0, 0.85),   // Fourth
    (5.0 / 4.0, 0.8),    // Major third
    (6.0 / 5.0, 0.75),   // Minor third
    (9.0 / 8.0, 0.68),   // Major second
    (16.0 / 15.0, 0.62), // Minor second
    (PHI, 0.7),          // Golden (special)
    (PHI * PHI, 0.65),   // Golden squared
];

/// Score how consonant a frequency ratio is (0.0 = dissonant, 1.0 = perfect).
///
/// Each peak in [`CONSONANCE_RATIOS`] contributes a triangular kernel of
/// half-width 0.1 in log-ratio space; the highest contribution wins. Used by
/// the audio engine to weight voice lifetime, and by the Knowledge-tab
/// playground to plot the landscape — single source of truth for both.
pub fn consonance_score(mut ratio: f64) -> f64 {
    if !ratio.is_finite() || ratio <= 0.0 {
        return 0.0;
    }
    if ratio < 1.0 {
        ratio = 1.0 / ratio; // Treat sub-harmonics symmetrically
    }
    let mut best_score = 0.0;
    for &(r, score) in CONSONANCE_RATIOS {
        let distance = consonance_distance(ratio, r);
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

fn consonance_distance(ratio: f64, target: f64) -> f64 {
    let diff = (ratio / target).ln();
    if target == 1.0 {
        diff.abs()
    } else {
        let octave = (diff / std::f64::consts::LN_2).round();
        (diff - octave * std::f64::consts::LN_2).abs()
    }
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
    pub interval_from_root: f64, // Ratio to base frequency
    pub trajectory: f64,         // Child/parent interval ratio; >1 rises, <1 falls
    pub amplitude: f64,          // Current amplitude (0.0 - 1.0)
    pub phase: f64,              // Phase accumulator
    pub age: f64,                // Seconds since birth
    pub lifetime: f64,           // Total expected lifetime
    pub generation: u8,          // How many ancestors spawned this voice
    pub pan: f64,                // -1.0 (left) to 1.0 (right)
    spatial: HrtfState,
    pipe: Option<PipeVoice>, // Sea Organ pipe driven by the sea, not the bell envelope
    alive: bool,
}

impl Voice {
    fn new(
        interval_from_root: f64,
        trajectory: f64,
        lifetime: f64,
        generation: u8,
        pan: f64,
    ) -> Self {
        Self {
            interval_from_root,
            trajectory,
            amplitude: 0.0, // Starts silent, fades in
            phase: 0.0,
            age: 0.0,
            lifetime,
            generation,
            pan,
            spatial: HrtfState::default(),
            pipe: None,
            alive: true,
        }
    }

    fn root() -> Self {
        Self::new(1.0, 1.0, 8.0, 0, 0.0)
    }

    /// A Sea Organ pipe at its fixed place on the wall.
    fn pipe(index: usize) -> Self {
        let mut voice = Self::new(
            sea_organ::pipe_ratio(index),
            1.0,
            PIPE_MAX_LIFETIME + PIPE_RELEASE_SECONDS * 2.0,
            (index / sea_organ::PIPES_PER_SEGMENT) as u8,
            sea_organ::pipe_pan(index),
        );
        voice.pipe = Some(PipeVoice::new(index as u8));
        voice
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

/// Per-voice state for lightweight HRTF-style spatialization.
#[derive(Clone)]
struct HrtfState {
    delay: [f64; HRTF_DELAY_BUFFER_LEN],
    write_idx: usize,
    low_l: f64,
    low_r: f64,
}

impl Default for HrtfState {
    fn default() -> Self {
        Self {
            delay: [0.0; HRTF_DELAY_BUFFER_LEN],
            write_idx: 0,
            low_l: 0.0,
            low_r: 0.0,
        }
    }
}

impl HrtfState {
    fn reset(&mut self) {
        self.delay = [0.0; HRTF_DELAY_BUFFER_LEN];
        self.write_idx = 0;
        self.low_l = 0.0;
        self.low_r = 0.0;
    }

    fn process(&mut self, sample: f64, pan: f64, sample_rate: f64) -> (f64, f64) {
        let pan = pan.clamp(-1.0, 1.0);
        let azimuth = pan * PI * 0.5;
        let side = azimuth.sin();
        let shadow = side.abs();

        self.delay[self.write_idx] = sample;

        let max_delay = (HRTF_DELAY_BUFFER_LEN - 2) as f64;
        let itd_samples = (HRTF_MAX_ITD_SECONDS * sample_rate * side).clamp(-max_delay, max_delay);
        let delayed_l = self.read_delay(itd_samples.max(0.0));
        let delayed_r = self.read_delay((-itd_samples).max(0.0));
        self.write_idx = (self.write_idx + 1) % HRTF_DELAY_BUFFER_LEN;

        let pan_angle = (pan + 1.0) * 0.25 * PI;
        let mut gain_l = pan_angle.cos();
        let mut gain_r = pan_angle.sin();

        let shadow_l = side.max(0.0);
        let shadow_r = (-side).max(0.0);
        gain_l *= 1.0 - HRTF_SHADOW_ATTENUATION * shadow_l;
        gain_r *= 1.0 - HRTF_SHADOW_ATTENUATION * shadow_r;

        let shaped_l = shape_far_ear(delayed_l, &mut self.low_l, shadow_l, sample_rate);
        let shaped_r = shape_far_ear(delayed_r, &mut self.low_r, shadow_r, sample_rate);

        // A gentle front-facing pinna cue: off-center voices lose a little
        // high-frequency directness even in the near ear.
        let presence = 1.0 - 0.04 * shadow;
        (shaped_l * gain_l * presence, shaped_r * gain_r * presence)
    }

    fn read_delay(&self, delay_samples: f64) -> f64 {
        let base = delay_samples.floor() as usize;
        let frac = delay_samples - base as f64;
        let near = self.delay[wrapped_delay_index(self.write_idx, base)];
        let far = self.delay[wrapped_delay_index(self.write_idx, base + 1)];
        near * (1.0 - frac) + far * frac
    }
}

fn wrapped_delay_index(write_idx: usize, offset: usize) -> usize {
    (write_idx + HRTF_DELAY_BUFFER_LEN - (offset % HRTF_DELAY_BUFFER_LEN)) % HRTF_DELAY_BUFFER_LEN
}

fn shape_far_ear(sample: f64, low_state: &mut f64, shadow: f64, sample_rate: f64) -> f64 {
    if shadow <= f64::EPSILON {
        *low_state = sample;
        return sample;
    }

    let cutoff = HRTF_NEAR_CUTOFF_HZ + (HRTF_FAR_CUTOFF_HZ - HRTF_NEAR_CUTOFF_HZ) * shadow;
    let alpha = one_pole_alpha(cutoff, sample_rate);
    *low_state += (sample - *low_state) * alpha;
    sample * (1.0 - shadow) + *low_state * shadow
}

fn one_pole_alpha(cutoff_hz: f64, sample_rate: f64) -> f64 {
    let omega = 2.0 * PI * cutoff_hz.max(1.0);
    (omega / (omega + sample_rate.max(1.0))).clamp(0.0, 1.0)
}

/// Sea Organ pipe voices: speaking threshold and drive come from the sea;
/// timbre is a clean stopped flue pipe (odd partials). Air and water noise
/// belong to the mist layer, so pipes carry no breath, chiff or scoop.
const PIPE_MAX_LIFETIME: f64 = 20.0; // Safety cap for one continuous swell, then release
const PIPE_RELEASE_SECONDS: f64 = 0.5; // ~6 smoothing time constants after unlatching
/// Slow enough that onsets swell in rather than strike like a note-on.
const PIPE_SMOOTHING_SECONDS: f64 = 0.08;
const PIPE_PITCH_RISE_MIN: f64 = 0.005; // Sharpening at the edge of speech
const PIPE_PITCH_RISE_SPAN: f64 = 0.02; // Extra sharpening at full drive
const PIPE_PARTIAL3_MIN: f64 = 0.15;
const PIPE_PARTIAL3_SPAN: f64 = 0.35;
const PIPE_PARTIAL5_SPAN: f64 = 0.125;
const PIPE_PARTIAL7_SPAN: f64 = 0.06;
/// Overblown register: the pipe's twelfth (its own slightly mistuned
/// resonance) takes over, with a little of its own 3rd for presence. Sea pressure never holds steady, so the pipe
/// keeps a share of its fundamental, which also keeps the storm's low body.
const PIPE_OVERBLOW_MIX: f64 = 0.7;
const PIPE_OVERBLOWN_PARTIAL9: f64 = 0.15;
/// Register changes glide over this long rather than switching.
const PIPE_REGISTER_SECONDS: f64 = 0.15;
/// Below this emergence level the sea organ fades out, so turning emergence
/// off never cuts a sounding pipe.
const PIPE_FADE_INTENSITY: f64 = 0.15;

#[derive(Clone, Copy)]
struct PipeVoice {
    index: u8,
    held: bool,
    target: f64,
    drive_target: f64,
    drive: f64,
    register_target: f64,
    register: f64,
    /// Overblown register runs on its own phase, near but not at 3×.
    upper_ratio: f64,
    upper_phase: f64,
    released_for: f64,
}

impl PipeVoice {
    fn new(index: u8) -> Self {
        Self {
            index,
            held: true,
            target: 0.0,
            drive_target: 0.0,
            drive: 0.0,
            register_target: 0.0,
            register: 0.0,
            upper_ratio: 3.0 * (1.0 + sea_organ::pipe_overblow_detune(index as usize)),
            upper_phase: 0.0,
            released_for: 0.0,
        }
    }
}

/// How spawn ratios are chosen.
///
/// `Canon` follows the original repeating ratio table (a Bach-style fugue).
/// `Penrose` reads ratios from a Fibonacci-word walk — the 1D quasicrystal
/// that traces a Conway worm through a Penrose P3 tiling.
/// `Fuxian` filters a consonance pool through counterpoint-style constraints.
/// `SeaOrgan` replaces the spawn timer with a simulated sea driving the 35
/// tuned pipes of the Zadar Sea Organ; gravity sets the wave period.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnMode {
    Canon = 0,
    Penrose = 1,
    Fuxian = 2,
    SeaOrgan = 3,
}

impl SpawnMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Canon => "canon",
            Self::Penrose => "penrose",
            Self::Fuxian => "fuxian",
            Self::SeaOrgan => "sea_organ",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Canon => Self::Penrose,
            Self::Penrose => Self::Fuxian,
            Self::Fuxian => Self::SeaOrgan,
            Self::SeaOrgan => Self::Canon,
        }
    }

    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Penrose,
            2 => Self::Fuxian,
            3 => Self::SeaOrgan,
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
    sea: SeaOrganState,
    sea_counter: usize,
}

#[derive(Clone, Copy)]
struct ParentInfo {
    interval_from_root: f64,
    trajectory: f64,
    generation: u8,
}

#[derive(Clone, Copy)]
struct FuxianCandidate {
    movement: f64,
    interval_from_root: f64,
    score: f64,
}

const EMPTY_FUXIAN_CANDIDATE: FuxianCandidate = FuxianCandidate {
    movement: 0.0,
    interval_from_root: 0.0,
    score: 0.0,
};

const FUXIAN_POOL_CAPACITY: usize = 32;
const MINOR_THIRD: f64 = 6.0 / 5.0;
const MAJOR_SECOND: f64 = 9.0 / 8.0;
const MINOR_SECOND: f64 = 16.0 / 15.0;
const FUXIAN_GRAVITY_STRENGTH: f64 = 6.0;
const RATIO_EPSILON: f64 = 1.0e-9;

impl EmergenceEngine {
    pub fn new(sample_rate: f64) -> Self {
        let mut engine = Self {
            voices: Vec::with_capacity(SEA_MAX_VOICES),
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
            sea: SeaOrganState::new(),
            sea_counter: 0,
        };
        // Seed with a root voice
        engine.voices.push(Voice::root());
        engine
    }

    pub fn reset(&mut self) {
        self.voices.clear();
        self.time = 0.0;
        self.spawn_timer = 0.0;
        self.epoch = 0;
        self.canon_position = 0;
        self.canon_offset = 1.0;
        self.penrose_walk = PenroseWalk::new();
        self.sea.reset();
        self.sea_counter = 0;
        // A calm sea is silent: the Sea Organ has no root drone.
        if self.spawn_mode != SpawnMode::SeaOrgan {
            self.voices.push(Voice::root());
        }
    }

    pub fn set_spawn_mode(&mut self, mode: SpawnMode) {
        // Entering Sea Organ drops voices that have not started sounding
        // (such as the seed root), so a preset that starts at sea starts in
        // silence; anything audible finishes its own envelope.
        if mode == SpawnMode::SeaOrgan && self.spawn_mode != mode {
            self.voices.retain(|voice| voice.amplitude >= 0.001);
        }
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
    pub fn process(
        &mut self,
        base_freq: f64,
        intensity: f64,
        gravity: f64,
        harm_weights: &[f64; 5],
        harm_intensity: f64,
    ) -> (f64, f64) {
        let dt = 1.0 / self.sample_rate;
        self.time += dt;
        self.spawn_timer += dt;

        let sea_mode = self.spawn_mode == SpawnMode::SeaOrgan;
        if sea_mode {
            // The sea, not a timer, decides when pipes speak.
            self.sea_counter += 1;
            if self.sea_counter >= sea_organ::CONTROL_INTERVAL {
                self.sea_counter = 0;
                let control_dt = sea_organ::CONTROL_INTERVAL as f64 * dt;
                self.sea.tick(
                    control_dt,
                    sea_organ::wave_height(intensity),
                    sea_organ::peak_period(gravity),
                );
                self.update_pipes(intensity);
            }
        } else {
            // Spawn logic - check if it's time for a new voice
            let adjusted_interval = self.spawn_interval / (0.5 + intensity * 1.5);
            if self.spawn_timer >= adjusted_interval {
                self.spawn_timer = 0.0;
                self.try_spawn(intensity, gravity);
            }
        }
        let pipe_rates = PipeRates {
            level: 1.0 - (-dt / PIPE_SMOOTHING_SECONDS).exp(),
            register: 1.0 - (-dt / PIPE_REGISTER_SECONDS).exp(),
        };

        // Generate audio from all living voices
        let mut sum_l = 0.0;
        let mut sum_r = 0.0;
        let mut total_energy = 0.0;

        for voice in &mut self.voices {
            if !voice.is_alive() {
                continue;
            }

            voice.age += dt;

            if let Some(pipe) = voice.pipe.as_mut() {
                let sample = pipe_sample(
                    pipe,
                    &mut voice.amplitude,
                    &mut voice.phase,
                    base_freq * voice.interval_from_root,
                    // Past the safety cap the pipe breathes out like any release.
                    sea_mode && voice.age < PIPE_MAX_LIFETIME,
                    pipe_rates,
                    dt,
                    self.sample_rate,
                );
                if pipe.released_for > PIPE_RELEASE_SECONDS {
                    voice.alive = false;
                }
                let (voice_l, voice_r) =
                    voice
                        .spatial
                        .process(sample * 0.15, voice.pan, self.sample_rate);
                sum_l += voice_l;
                sum_r += voice_r;
                total_energy += voice.amplitude;
                continue;
            }

            let env = voice.envelope();
            voice.amplitude = env * intensity;

            if voice.amplitude < 0.001 {
                continue;
            }

            // Phase accumulator for this voice
            let freq = base_freq * voice.interval_from_root;
            voice.phase += freq / self.sample_rate;
            voice.phase -= voice.phase.floor();

            // Dynamic timbre matching the carrier wave
            let mut sample = (TAU * voice.phase).sin();
            if harm_intensity > 0.01 {
                let mut total_weight = 0.0;
                for i in 0..5 {
                    let weight = harm_weights[i];
                    if weight < 0.001 {
                        continue;
                    }
                    let mult = (i + 2) as f64;
                    sample += (TAU * voice.phase * mult).sin() * weight * harm_intensity;
                    total_weight += weight;
                }
                let norm = 1.0 + harm_intensity * total_weight;
                sample /= norm;
            }

            let voice_sample = sample * voice.amplitude * 0.15; // Background voice scale
            let (voice_l, voice_r) =
                voice
                    .spatial
                    .process(voice_sample, voice.pan, self.sample_rate);

            sum_l += voice_l;
            sum_r += voice_r;
            total_energy += voice.amplitude;
        }

        // Energy conservation: normalize if too loud. Sea Organ pipes are
        // independent sources, so their layers add in power, not amplitude.
        if total_energy > 2.0 {
            let scale = if sea_mode {
                (2.0 / total_energy).sqrt()
            } else {
                2.0 / total_energy
            };
            sum_l *= scale;
            sum_r *= scale;
        }

        // Remove dead voices
        self.voices.retain(|v| v.is_alive());

        (sum_l, sum_r)
    }

    /// Control-rate Sea Organ update: start pipes whose segment just crossed
    /// their threshold and retarget every pipe voice from the sea.
    fn update_pipes(&mut self, intensity: f64) {
        for index in 0..sea_organ::PIPES {
            if self.sea.pipe_onset(index) {
                self.start_pipe(index);
            }
        }

        let fade = (intensity / PIPE_FADE_INTENSITY).min(1.0);
        for voice in &mut self.voices {
            let Some(pipe) = voice.pipe.as_mut() else {
                continue;
            };
            let index = pipe.index as usize;
            let drive = self.sea.pipe_drive(index);
            let gain = sea_organ::pipe_distance_gain(index) * fade;
            pipe.held = self.sea.pipe_latched(index);
            pipe.drive_target = drive;
            pipe.target = smoothstep(drive) * gain;
            pipe.register_target = self.sea.pipe_register(index);
        }
    }

    fn start_pipe(&mut self, index: usize) {
        if let Some(voice) = self
            .voices
            .iter_mut()
            .find(|voice| voice.pipe.is_some_and(|pipe| pipe.index as usize == index))
        {
            // Still sounding from the last wave: carry on.
            if let Some(pipe) = voice.pipe.as_mut() {
                pipe.released_for = 0.0;
            }
            voice.age = 0.0;
            voice.alive = true;
            return;
        }

        if self.voices.len() >= SEA_MAX_VOICES {
            // The sea keeps the loudest pipes.
            if let Some(quietest) = self
                .voices
                .iter_mut()
                .min_by(|a, b| a.amplitude.total_cmp(&b.amplitude))
            {
                quietest.alive = false;
            }
            self.voices.retain(|v| v.alive);
        }

        self.epoch += 1;
        self.voices.push(Voice::pipe(index));
    }

    fn try_spawn(&mut self, intensity: f64, gravity: f64) {
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
            self.voices.push(Voice::root());
            return;
        }

        self.epoch += 1;

        let parent = self
            .voices
            .iter()
            .filter(|voice| voice.is_alive())
            .max_by(|a, b| a.amplitude.total_cmp(&b.amplitude))
            .map(|voice| ParentInfo {
                interval_from_root: voice.interval_from_root,
                trajectory: voice.trajectory,
                generation: voice.generation,
            })
            .unwrap_or(ParentInfo {
                interval_from_root: 1.0,
                trajectory: 1.0,
                generation: 0,
            });

        // Choose the interval according to the active spawn mode. Canon and
        // Penrose produce parent-relative moves; Fuxian chooses the child's
        // root interval directly from a constrained counterpoint pool.
        let final_ratio = match self.spawn_mode {
            SpawnMode::Canon => {
                let idx = self.canon_pattern[self.canon_position % self.canon_pattern.len()];
                self.canon_position += 1;
                let base_ratio = SPAWN_RATIOS[idx % SPAWN_RATIOS.len()];
                self.canon_child_ratio(parent.interval_from_root, base_ratio, intensity)
            }
            SpawnMode::Penrose => {
                // Each spawn advances the Conway worm by one rhomb; the
                // (previous, current) tile pair selects the harmonic move.
                let (prev, curr) = self.penrose_walk.step();
                self.canon_position += 1;
                let base_ratio = penrose::pair_ratio(prev, curr);
                self.canon_child_ratio(parent.interval_from_root, base_ratio, intensity)
            }
            SpawnMode::Fuxian => self.fuxian_child_ratio(parent, gravity),
            // Sea Organ pipes start from the sea in `update_pipes`.
            SpawnMode::SeaOrgan => return,
        };

        // Lifetime varies with the interval from parent to child: longer for
        // consonant relationships, shorter for dissonant ones.
        let realized_interval = final_ratio / parent.interval_from_root;
        let consonance = self.consonance_score(realized_interval);
        let base_lifetime = 4.0 + consonance * 8.0; // 4-12 seconds
        let lifetime = base_lifetime * (0.8 + self.xorshift() * 0.4);

        let generation = parent.generation.saturating_add(1).min(7);

        // Pan: spread voices across the stereo field
        let pan = (self.xorshift() * 2.0 - 1.0) * 0.7; // -0.7 to 0.7

        self.voices.push(Voice::new(
            final_ratio,
            realized_interval,
            lifetime,
            generation,
            pan,
        ));
    }

    fn canon_child_ratio(&mut self, parent_ratio: f64, base_ratio: f64, intensity: f64) -> f64 {
        // Every 8 spawns, shift the canon offset (like a fugue answer).
        // Penrose mode reuses the same transposition cadence so the walk
        // sweeps across registers without losing its quasicrystal cadence.
        if self.canon_position.is_multiple_of(8) {
            self.canon_offset = SPAWN_RATIOS[(self.epoch as usize / 8) % SPAWN_RATIOS.len()];
        }

        // Apply canon offset and slight random mutation. The resulting interval
        // is attached to the strongest current voice, so generations are
        // audible parent-child branches rather than labels only.
        let mutation = 1.0 + (self.xorshift() - 0.5) * 0.02 * intensity;
        let interval = base_ratio * self.canon_offset * mutation;
        fold_voice_ratio(parent_ratio * interval)
    }

    fn fuxian_child_ratio(&mut self, parent: ParentInfo, gravity: f64) -> f64 {
        let mut pool = [EMPTY_FUXIAN_CANDIDATE; FUXIAN_POOL_CAPACITY];
        let len = build_fuxian_pool(&mut pool, parent.interval_from_root);
        let parent_motion = safe_ln(parent.trajectory).unwrap_or(0.0);
        let require_step = parent_motion.abs() > MINOR_THIRD.ln();

        if let Some(selected) =
            self.weighted_fuxian_selection(&pool[..len], parent, gravity, require_step)
        {
            return selected;
        }

        if require_step {
            let mut recovery = [EMPTY_FUXIAN_CANDIDATE; 2];
            let len =
                build_step_recovery_pool(&mut recovery, parent.interval_from_root, parent_motion);
            if let Some(selected) =
                self.weighted_fuxian_selection(&recovery[..len], parent, gravity, false)
            {
                return selected;
            }
        }

        1.0
    }

    fn weighted_fuxian_selection(
        &mut self,
        candidates: &[FuxianCandidate],
        parent: ParentInfo,
        gravity: f64,
        require_step: bool,
    ) -> Option<f64> {
        let gravity = gravity.clamp(0.0, 1.0);
        let mut total = 0.0;

        for &candidate in candidates {
            if !fuxian_candidate_allowed(candidate, parent, require_step) {
                continue;
            }
            total += fuxian_weight(candidate, parent.interval_from_root, gravity);
        }

        if total <= f64::EPSILON {
            return None;
        }

        let mut ticket = self.xorshift() * total;
        for &candidate in candidates {
            if !fuxian_candidate_allowed(candidate, parent, require_step) {
                continue;
            }
            let weight = fuxian_weight(candidate, parent.interval_from_root, gravity);
            if ticket <= weight {
                return Some(candidate.interval_from_root);
            }
            ticket -= weight;
        }

        candidates
            .iter()
            .rev()
            .copied()
            .find(|&candidate| fuxian_candidate_allowed(candidate, parent, require_step))
            .map(|candidate| candidate.interval_from_root)
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
            .filter(|v| v.is_alive())
            .map(|v| VoiceInfo {
                freq_ratio: v.interval_from_root as f32,
                amplitude: v.amplitude as f32,
                age_normalized: (v.age / v.lifetime).clamp(0.0, 1.0) as f32,
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

/// Per-sample one-pole coefficients for Sea Organ pipes.
#[derive(Clone, Copy)]
struct PipeRates {
    level: f64,
    register: f64,
}

/// Render one sample of a Sea Organ pipe and advance its state.
#[allow(clippy::too_many_arguments)]
fn pipe_sample(
    pipe: &mut PipeVoice,
    amplitude: &mut f64,
    phase: &mut f64,
    freq: f64,
    driven: bool,
    rates: PipeRates,
    dt: f64,
    sample_rate: f64,
) -> f64 {
    if !driven {
        // Left Sea Organ mode or held too long: let the pipe fade out.
        pipe.held = false;
        pipe.target = 0.0;
    }
    if pipe.held {
        pipe.released_for = 0.0;
    } else {
        pipe.released_for += dt;
    }
    *amplitude += (pipe.target - *amplitude) * rates.level;
    pipe.drive += (pipe.drive_target - pipe.drive) * rates.level;
    pipe.register += (pipe.register_target - pipe.register) * rates.register;

    if *amplitude < 0.001 {
        return 0.0;
    }

    // Pitch rises slightly with pressure.
    let bend = 1.0 + PIPE_PITCH_RISE_MIN + PIPE_PITCH_RISE_SPAN * pipe.drive;
    *phase += freq * bend / sample_rate;
    *phase -= phase.floor();
    pipe.upper_phase += freq * bend * pipe.upper_ratio / sample_rate;
    pipe.upper_phase -= pipe.upper_phase.floor();

    // Stopped pipe: odd partials, brightening with drive.
    let w3 = PIPE_PARTIAL3_MIN + PIPE_PARTIAL3_SPAN * pipe.drive;
    let w5 = PIPE_PARTIAL5_SPAN * pipe.drive;
    let w7 = PIPE_PARTIAL7_SPAN * pipe.drive;
    // Odd partials by angle addition: one sin/cos, then steps of 2p.
    let (s1, c1) = (TAU * *phase).sin_cos();
    let (s2, c2) = (2.0 * s1 * c1, 1.0 - 2.0 * s1 * s1);
    let (s3, c3) = (s1 * c2 + c1 * s2, c1 * c2 - s1 * s2);
    let (s5, c5) = (s3 * c2 + c3 * s2, c3 * c2 - s3 * s2);
    let s7 = s5 * c2 + c5 * s2;
    let fundamental = (s1 + w3 * s3 + w5 * s5 + w7 * s7) / (1.0 + w3 + w5 + w7);

    // Overblown: the twelfth takes over as the water rises. It is not phase
    // locked to the fundamental, so while both sound its slight mistuning
    // beats against the fundamental's 3rd harmonic: the flutter.
    let blend = PIPE_OVERBLOW_MIX * pipe.register;
    let tone = if blend > 1.0e-4 {
        let su = (TAU * pipe.upper_phase).sin();
        let su3 = su * (3.0 - 4.0 * su * su);
        let overblown = (su + PIPE_OVERBLOWN_PARTIAL9 * su3) / (1.0 + PIPE_OVERBLOWN_PARTIAL9);
        fundamental * (1.0 - blend) + overblown * blend
    } else {
        fundamental
    };

    tone * *amplitude
}

fn build_fuxian_pool(
    pool: &mut [FuxianCandidate; FUXIAN_POOL_CAPACITY],
    parent_interval_from_root: f64,
) -> usize {
    let mut len = 0;
    for &(ratio, score) in CONSONANCE_RATIOS {
        push_fuxian_candidate(pool, &mut len, parent_interval_from_root, ratio, score);
        if !ratio_close(ratio, 1.0) {
            push_fuxian_candidate(
                pool,
                &mut len,
                parent_interval_from_root,
                1.0 / ratio,
                score,
            );
        }
    }
    len
}

fn build_step_recovery_pool(
    pool: &mut [FuxianCandidate; 2],
    parent_interval_from_root: f64,
    parent_motion: f64,
) -> usize {
    let mut len = 0;
    let steps = if parent_motion.is_sign_positive() {
        [1.0 / MAJOR_SECOND, 1.0 / MINOR_SECOND]
    } else {
        [MAJOR_SECOND, MINOR_SECOND]
    };

    for step in steps {
        let interval_from_root = fold_voice_ratio(parent_interval_from_root * step);
        pool[len] = FuxianCandidate {
            movement: step,
            interval_from_root,
            score: consonance_score(step),
        };
        len += 1;
    }

    len
}

fn push_fuxian_candidate(
    pool: &mut [FuxianCandidate; FUXIAN_POOL_CAPACITY],
    len: &mut usize,
    parent_interval_from_root: f64,
    movement: f64,
    score: f64,
) {
    if *len >= pool.len() || !movement.is_finite() || movement <= 0.0 {
        return;
    }
    if ratio_close(movement, 1.0) {
        return;
    }
    let interval_from_root = fold_voice_ratio(parent_interval_from_root * movement);
    if ratio_close(interval_from_root, parent_interval_from_root) {
        return;
    }
    if pool[..*len]
        .iter()
        .any(|candidate| ratio_close(candidate.interval_from_root, interval_from_root))
    {
        return;
    }
    pool[*len] = FuxianCandidate {
        movement,
        interval_from_root,
        score,
    };
    *len += 1;
}

fn fuxian_candidate_allowed(
    candidate: FuxianCandidate,
    parent: ParentInfo,
    require_step: bool,
) -> bool {
    if candidate.interval_from_root <= 0.0 || !candidate.interval_from_root.is_finite() {
        return false;
    }
    if ratio_close(candidate.interval_from_root, parent.interval_from_root) {
        return false;
    }
    if fuxian_parallel(candidate.interval_from_root, parent.interval_from_root) {
        return false;
    }
    if require_step {
        return fuxian_opposite_step(candidate.interval_from_root, parent);
    }
    true
}

fn fuxian_parallel(candidate_interval_from_root: f64, parent_interval_from_root: f64) -> bool {
    if is_perfect_fifth(parent_interval_from_root) {
        return is_perfect_fifth(candidate_interval_from_root);
    }
    if is_octave(parent_interval_from_root) {
        return is_octave(candidate_interval_from_root);
    }
    false
}

fn fuxian_opposite_step(candidate_interval_from_root: f64, parent: ParentInfo) -> bool {
    let parent_motion = match safe_ln(parent.trajectory) {
        Some(value) if value.abs() > MINOR_THIRD.ln() => value,
        _ => return true,
    };
    let child_motion = match safe_ln(candidate_interval_from_root / parent.interval_from_root) {
        Some(value) => value,
        None => return false,
    };

    child_motion.signum() != parent_motion.signum()
        && child_motion.abs() > RATIO_EPSILON
        && child_motion.abs() <= MAJOR_SECOND.ln() + RATIO_EPSILON
}

fn fuxian_weight(candidate: FuxianCandidate, parent_interval_from_root: f64, gravity: f64) -> f64 {
    let Some(parent_distance) = safe_ln(parent_interval_from_root) else {
        return 0.0;
    };
    let Some(candidate_distance) = safe_ln(candidate.interval_from_root) else {
        return 0.0;
    };
    let pull = parent_distance.abs() - candidate_distance.abs();
    let gravity_bias = (pull * gravity * FUXIAN_GRAVITY_STRENGTH).exp();
    (0.05 + candidate.score.max(0.0)) * gravity_bias
}

fn is_perfect_fifth(ratio: f64) -> bool {
    ratio_close(ratio, 3.0 / 2.0) || ratio_close(ratio, 2.0 / 3.0)
}

fn is_octave(ratio: f64) -> bool {
    ratio_close(ratio, 2.0) || ratio_close(ratio, 0.5)
}

fn ratio_close(a: f64, b: f64) -> bool {
    safe_ln(a / b)
        .map(|distance| distance.abs() <= RATIO_EPSILON)
        .unwrap_or(false)
}

fn safe_ln(value: f64) -> Option<f64> {
    if value.is_finite() && value > 0.0 {
        Some(value.ln())
    } else {
        None
    }
}

fn fold_voice_ratio(mut ratio: f64) -> f64 {
    if !ratio.is_finite() || ratio <= 0.0 {
        return 1.0;
    }

    while ratio < 0.25 {
        ratio *= 2.0;
    }
    while ratio > 4.0 {
        ratio *= 0.5;
    }

    ratio.clamp(0.25, 4.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1.0e-9,
            "actual {actual} expected {expected}"
        );
    }

    fn assert_near(actual: f64, expected: f64, epsilon: f64) {
        assert!(
            (actual - expected).abs() <= epsilon,
            "actual {actual} expected {expected} epsilon {epsilon}"
        );
    }

    #[test]
    fn consonance_score_handles_inversions_and_invalid_ratios() {
        assert_close(consonance_score(2.0 / 3.0), consonance_score(3.0 / 2.0));
        assert_close(consonance_score(0.5), consonance_score(2.0));
        assert_eq!(consonance_score(0.0), 0.0);
        assert_eq!(consonance_score(-1.0), 0.0);
    }

    #[test]
    fn consonance_score_respects_octave_equivalent_intervals() {
        assert_close(consonance_score(3.0), consonance_score(3.0 / 2.0));
        assert_close(consonance_score(4.0), consonance_score(2.0));
    }

    #[test]
    fn spawned_voice_uses_strongest_parent_ratio() {
        let mut engine = EmergenceEngine::new(100.0);
        engine.voices.clear();
        engine.voices.push(Voice {
            interval_from_root: 2.0,
            trajectory: 1.0,
            amplitude: 1.0,
            phase: 0.0,
            age: 1.0,
            lifetime: 10.0,
            generation: 2,
            pan: 0.0,
            spatial: HrtfState::default(),
            pipe: None,
            alive: true,
        });
        engine.canon_pattern = vec![7]; // 3:2 above the parent

        engine.try_spawn(0.0, 0.5);

        let spawned = engine.voices.last().expect("voice should spawn");
        assert_close(spawned.interval_from_root, 3.0);
        assert_close(spawned.trajectory, 3.0 / 2.0);
        assert_eq!(spawned.generation, 3);
    }

    #[test]
    fn reset_returns_engine_to_single_root_voice() {
        let mut engine = EmergenceEngine::new(100.0);
        engine.try_spawn(0.0, 0.5);
        assert!(engine.voices.len() > 1);

        engine.reset();

        assert_eq!(engine.epoch, 0);
        assert_eq!(engine.canon_position, 0);
        assert_eq!(engine.voices.len(), 1);
        assert_close(engine.voices[0].interval_from_root, 1.0);
        assert_close(engine.voices[0].trajectory, 1.0);
        assert_eq!(engine.snapshot().voices.len(), 1);
    }

    #[test]
    fn fuxian_spawn_moves_from_the_parent_instead_of_duplication() {
        let mut engine = EmergenceEngine::new(100.0);
        engine.set_spawn_mode(SpawnMode::Fuxian);

        engine.try_spawn(0.0, 0.5);

        let spawned = engine.voices.last().expect("voice should spawn");
        assert_ne!(spawned.interval_from_root, 1.0);
        assert_ne!(spawned.trajectory, 1.0);
        assert_eq!(spawned.generation, 1);
    }

    #[test]
    fn fuxian_pool_builds_parent_relative_candidates() {
        let mut pool = [EMPTY_FUXIAN_CANDIDATE; FUXIAN_POOL_CAPACITY];
        let len = build_fuxian_pool(&mut pool, 3.0 / 2.0);

        assert!(pool[..len].iter().any(|candidate| {
            ratio_close(candidate.movement, 9.0 / 8.0)
                && ratio_close(candidate.interval_from_root, 27.0 / 16.0)
        }));
        assert!(
            pool[..len]
                .iter()
                .all(|candidate| !ratio_close(candidate.interval_from_root, 3.0 / 2.0))
        );
    }

    #[test]
    fn fuxian_parallel_filter_rejects_repeated_fifths_and_octaves() {
        let fifth_parent = ParentInfo {
            interval_from_root: 3.0 / 2.0,
            trajectory: 1.0,
            generation: 1,
        };
        let octave_parent = ParentInfo {
            interval_from_root: 2.0,
            trajectory: 1.0,
            generation: 1,
        };

        assert!(!fuxian_candidate_allowed(
            FuxianCandidate {
                movement: 1.0,
                interval_from_root: 3.0 / 2.0,
                score: 0.9,
            },
            fifth_parent,
            false,
        ));
        assert!(!fuxian_candidate_allowed(
            FuxianCandidate {
                movement: 1.0,
                interval_from_root: 2.0,
                score: 0.95,
            },
            octave_parent,
            false,
        ));
        assert!(fuxian_candidate_allowed(
            FuxianCandidate {
                movement: 8.0 / 9.0,
                interval_from_root: 4.0 / 3.0,
                score: 0.85,
            },
            fifth_parent,
            false,
        ));
    }

    #[test]
    fn fuxian_leap_rule_requires_opposite_stepwise_motion() {
        let parent = ParentInfo {
            interval_from_root: 3.0 / 2.0,
            trajectory: 3.0 / 2.0,
            generation: 1,
        };

        assert!(fuxian_candidate_allowed(
            FuxianCandidate {
                movement: 8.0 / 9.0,
                interval_from_root: 4.0 / 3.0,
                score: 0.85,
            },
            parent,
            true,
        ));
        assert!(!fuxian_candidate_allowed(
            FuxianCandidate {
                movement: 4.0 / 3.0,
                interval_from_root: 2.0,
                score: 0.95,
            },
            parent,
            true,
        ));
        assert!(!fuxian_candidate_allowed(
            FuxianCandidate {
                movement: 2.0 / 3.0,
                interval_from_root: 1.0,
                score: 1.0,
            },
            parent,
            true,
        ));
    }

    #[test]
    fn fuxian_gravity_exponentially_biases_toward_root() {
        let near_root = FuxianCandidate {
            movement: 0.5,
            interval_from_root: 1.0,
            score: 1.0,
        };
        let far_from_root = FuxianCandidate {
            movement: 3.0 / 2.0,
            interval_from_root: 3.0,
            score: 0.95,
        };
        let parent = 2.0;

        let neutral_near = fuxian_weight(near_root, parent, 0.0);
        let neutral_far = fuxian_weight(far_from_root, parent, 0.0);
        let weighted_near = fuxian_weight(near_root, parent, 1.0);
        let weighted_far = fuxian_weight(far_from_root, parent, 1.0);

        assert!(neutral_near > neutral_far);
        assert!(weighted_near / weighted_far > neutral_near / neutral_far);
    }

    #[test]
    fn fold_voice_ratio_wraps_by_octave_instead_of_hard_clamping() {
        assert_close(fold_voice_ratio(8.0), 4.0);
        assert_close(fold_voice_ratio(6.0), 3.0);
        assert_close(fold_voice_ratio(0.125), 0.25);
        assert_close(fold_voice_ratio(0.1875), 0.375);
    }

    #[test]
    fn spawn_mode_round_trips_and_cycles_through_sea_organ() {
        assert_eq!(SpawnMode::from_u32(3), SpawnMode::SeaOrgan);
        assert_eq!(SpawnMode::from_u32(SpawnMode::SeaOrgan as u32), SpawnMode::SeaOrgan);
        assert_eq!(SpawnMode::Fuxian.toggled(), SpawnMode::SeaOrgan);
        assert_eq!(SpawnMode::SeaOrgan.toggled(), SpawnMode::Canon);
        assert_eq!(SpawnMode::SeaOrgan.label(), "sea_organ");
    }

    fn sea_engine(sample_rate: f64) -> EmergenceEngine {
        let mut engine = EmergenceEngine::new(sample_rate);
        engine.set_spawn_mode(SpawnMode::SeaOrgan);
        engine
    }

    #[test]
    fn sea_organ_starts_and_resets_without_a_root_voice() {
        let mut engine = sea_engine(8_000.0);
        assert!(engine.voices.is_empty());

        engine.voices.push(Voice::pipe(17));
        engine.reset();
        assert!(engine.voices.is_empty());
    }

    #[test]
    fn calm_sea_organ_stays_silent() {
        let mut engine = sea_engine(8_000.0);
        let weights = [0.0; 5];
        for _ in 0..(8_000 * 60) {
            let (left, right) = engine.process(146.83, 0.1, 0.5, &weights, 0.0);
            assert_eq!((left, right), (0.0, 0.0));
        }
        assert!(engine.voices.is_empty());
        assert_eq!(engine.epoch, 0);
    }

    #[test]
    fn stormy_sea_organ_stays_bounded_within_the_voice_cap() {
        let mut engine = sea_engine(48_000.0);
        let weights = [0.0; 5];
        let mut peak = 0.0_f64;
        let mut most_voices = 0;
        for _ in 0..(48_000 * 60) {
            let (left, right) = engine.process(146.83, 1.0, 1.0, &weights, 0.0);
            assert!(left.is_finite() && right.is_finite());
            peak = peak.max(left.abs()).max(right.abs());
            most_voices = most_voices.max(engine.voices.len());
        }
        assert!(engine.epoch > 0, "the storm should sound pipes");
        assert!(most_voices <= SEA_MAX_VOICES);
        assert!(engine.voices.iter().all(|voice| voice.pipe.is_some()));
        assert!(peak > 0.01 && peak < 1.0, "peak {peak}");
    }

    #[test]
    fn leaving_sea_organ_lets_pipes_fade_out() {
        let mut engine = sea_engine(8_000.0);
        engine.voices.push(Voice::pipe(17));
        engine.set_spawn_mode(SpawnMode::Canon);
        let weights = [0.0; 5];
        for _ in 0..8_000 {
            engine.process(146.83, 0.5, 0.5, &weights, 0.0);
        }
        assert!(engine.voices.iter().all(|voice| voice.pipe.is_none()));
    }

    /// Magnitude of `samples` at `freq` (single-bin DFT).
    fn magnitude_at(samples: &[f64], freq: f64, sample_rate: f64) -> f64 {
        let (mut re, mut im) = (0.0, 0.0);
        for (n, &x) in samples.iter().enumerate() {
            let angle = TAU * freq * n as f64 / sample_rate;
            re += x * angle.cos();
            im += x * angle.sin();
        }
        (re * re + im * im).sqrt() / samples.len() as f64
    }

    fn render_pipe(register: f64) -> Vec<f64> {
        render_pipe_for(register, 1.0)
    }

    fn render_pipe_for(register: f64, seconds: f64) -> Vec<f64> {
        let mut pipe = PipeVoice::new(17);
        pipe.target = 1.0;
        pipe.register_target = register;
        pipe.register = register;
        let (mut amplitude, mut phase) = (1.0, 0.0);
        let rates = PipeRates {
            level: 1.0,
            register: 1.0,
        };
        (0..(8_000.0 * seconds) as usize)
            .map(|_| {
                pipe_sample(
                    &mut pipe, &mut amplitude, &mut phase, 100.0, true, rates, 1.0 / 8_000.0,
                    8_000.0,
                )
            })
            .collect()
    }

    #[test]
    fn overblown_pipe_moves_up_a_twelfth() {
        // Pitch sits 0.5 % sharp at zero drive.
        let f = 100.0 * (1.0 + PIPE_PITCH_RISE_MIN);
        let upper = f * PipeVoice::new(17).upper_ratio;
        let low = render_pipe(0.0);
        let high = render_pipe(1.0);
        assert!(magnitude_at(&low, f, 8_000.0) > 4.0 * magnitude_at(&low, 3.0 * f, 8_000.0));
        assert!(magnitude_at(&high, upper, 8_000.0) > 2.0 * magnitude_at(&high, f, 8_000.0));
        assert!(magnitude_at(&high, f, 8_000.0) > 0.0);
    }

    #[test]
    fn partly_overblown_pipe_flutters_at_trill_rate() {
        let f = 100.0 * (1.0 + PIPE_PITCH_RISE_MIN);
        let upper = f * PipeVoice::new(17).upper_ratio;
        let beat = (upper - 3.0 * f).abs();
        assert!((1.0..=4.0).contains(&beat), "beat {beat} Hz at a 100 Hz pipe");

        // Both near-twelfths sound together, so they beat at `beat` Hz.
        let mixed = render_pipe_for(0.35, 4.0);
        let harmonic = magnitude_at(&mixed, 3.0 * f, 8_000.0);
        let twelfth = magnitude_at(&mixed, upper, 8_000.0);
        assert!(harmonic > 0.03 && twelfth > 0.03, "{harmonic} {twelfth}");
        assert!(harmonic.min(twelfth) / harmonic.max(twelfth) > 0.25);
    }

    #[test]
    fn pipe_held_past_the_cap_fades_before_it_dies() {
        let mut engine = sea_engine(8_000.0);
        let mut voice = Voice::pipe(17);
        voice.age = PIPE_MAX_LIFETIME;
        voice.amplitude = 0.5;
        engine.voices.push(voice);
        let weights = [0.0; 5];
        let mut last = 0.5;
        while let Some(voice) = engine.voices.first() {
            assert!(voice.amplitude <= last + 1.0e-12, "release must not jump");
            last = voice.amplitude;
            engine.process(146.83, 0.05, 0.5, &weights, 0.0);
        }
        assert!(last < 0.01, "voice died at amplitude {last}");
    }

    #[test]
    fn entering_sea_organ_keeps_sounding_voices() {
        let mut engine = EmergenceEngine::new(8_000.0);
        engine.voices[0].amplitude = 0.4;
        engine.set_spawn_mode(SpawnMode::SeaOrgan);
        assert_eq!(engine.voices.len(), 1);
    }

    #[test]
    fn hrtf_center_pan_is_balanced() {
        let mut hrtf = HrtfState::default();
        let (left, right) = hrtf.process(1.0, 0.0, 48_000.0);
        let expected = 0.5_f64.sqrt();

        assert_near(left, expected, 1.0e-12);
        assert_near(right, expected, 1.0e-12);
    }

    #[test]
    fn hrtf_side_pan_delays_the_far_ear() {
        let mut right_source = HrtfState::default();
        let (left, right) = right_source.process(1.0, 1.0, 48_000.0);
        assert_near(left, 0.0, 1.0e-12);
        assert!(right > 0.9);

        let mut left_source = HrtfState::default();
        let (left, right) = left_source.process(1.0, -1.0, 48_000.0);
        assert!(left > 0.9);
        assert_near(right, 0.0, 1.0e-12);
    }

    #[test]
    fn hrtf_output_stays_bounded() {
        let mut hrtf = HrtfState::default();
        let mut peak = 0.0_f64;

        for n in 0_usize..4096 {
            let sample = if n.is_multiple_of(2) { 1.0 } else { -1.0 };
            let pan = (n % 201) as f64 / 100.0 - 1.0;
            let (left, right) = hrtf.process(sample, pan, 48_000.0);
            peak = peak.max(left.abs()).max(right.abs());
        }

        assert!(peak <= 1.0, "peak {peak}");
    }

    #[test]
    fn hrtf_reset_clears_delay_and_filter_history() {
        let mut hrtf = HrtfState::default();
        for _ in 0..64 {
            hrtf.process(1.0, 1.0, 48_000.0);
        }

        hrtf.reset();
        let (left, right) = hrtf.process(1.0, 1.0, 48_000.0);

        assert_near(left, 0.0, 1.0e-12);
        assert!(right > 0.9);
    }
}
