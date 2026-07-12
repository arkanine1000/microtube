//! The sample-accurate synthesis engine.
//!
//! [`Engine`] owns every oscillator, accumulator and sub-engine and turns a
//! set of target [`Params`] into stereo audio. Front-ends only ever push
//! target values and pull samples — the smoothing, voice management and
//! limiting all happen here, identically on native and Wasm.

use std::f64::consts::TAU;

use crate::emergence::{EmergenceEngine, EmergenceSnapshot, SpawnMode};
use crate::shepard::{DEFAULT_BASE_FREQ_HZ, Direction, ShepardEngine};
use crate::synth::{MistType, NoiseGen, Timbre, mist_gain, soft_clip};

/// All audio parameters a front-end can drive. Plain `Copy` data — the
/// CLI mirrors its lock-free atomics into this, the worklet mutates it
/// directly from `postMessage` events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub playing: bool,
    pub base_freq: f32,
    pub beat_freq: f32,
    pub volume: f32,
    pub noise_level: f32,
    pub mist_type: MistType,
    pub harmonics: f32,
    pub emergence: f32,
    pub gravity: f32,
    pub spawn_mode: SpawnMode,
    pub shepard: f32,
    pub shepard_base_freq: f32,
    pub shepard_direction: Direction,
    pub timbre: Timbre,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            playing: true,
            base_freq: 220.0,
            beat_freq: 10.0,
            volume: 0.5,
            noise_level: 0.0,
            mist_type: MistType::Brown,
            harmonics: 0.3,
            emergence: 0.0,
            gravity: 0.5,
            spawn_mode: SpawnMode::Canon,
            shepard: 0.0,
            shepard_base_freq: DEFAULT_BASE_FREQ_HZ as f32,
            shepard_direction: Direction::Down,
            timbre: Timbre::Organ,
        }
    }
}

/// The synthesis engine. Construct once per audio stream with the device
/// sample rate, push [`Params`], then pull frames.
pub struct Engine {
    sample_rate: f64,
    smooth_alpha: f64,
    targets: Params,

    // Smoothed ("current") parameter values, chased toward `targets`.
    current_base: f64,
    current_beat: f64,
    current_vol: f64,
    current_noise: f64,
    current_harmonics: f64,
    current_emergence: f64,
    current_gravity: f64,
    current_shepard: f64,
    current_shepard_base: f64,
    current_harm_weights: [f64; 5],

    // Carrier + harmonic phase accumulators.
    phase_l: f64,
    phase_r: f64,
    harm_phase_l: [f64; 5],
    harm_phase_r: [f64; 5],

    noise: NoiseGen,
    emergence: EmergenceEngine,
    shepard: ShepardEngine,
    emergence_was_active: bool,
}

impl Engine {
    /// Create an engine for the given device sample rate (Hz).
    pub fn new(sample_rate: f64) -> Self {
        // ~50 ms exponential smoothing time constant.
        let smooth_alpha = 1.0 - (-1.0 / (sample_rate * 0.05)).exp();
        Self {
            sample_rate,
            smooth_alpha,
            targets: Params::default(),
            current_base: 220.0,
            current_beat: 10.0,
            current_vol: 0.5,
            current_noise: 0.0,
            current_harmonics: 0.3,
            current_emergence: 0.0,
            current_gravity: 0.5,
            current_shepard: 0.0,
            current_shepard_base: DEFAULT_BASE_FREQ_HZ,
            current_harm_weights: Timbre::Organ.weights(),
            phase_l: 0.0,
            phase_r: 0.0,
            harm_phase_l: [0.0; 5],
            harm_phase_r: [0.0; 5],
            noise: NoiseGen::new(),
            emergence: EmergenceEngine::new(sample_rate),
            shepard: ShepardEngine::new(sample_rate),
            emergence_was_active: false,
        }
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Mutable access to the target parameters. Cheap to call every block.
    pub fn targets_mut(&mut self) -> &mut Params {
        &mut self.targets
    }

    /// Replace all target parameters at once.
    pub fn set_targets(&mut self, params: Params) {
        self.targets = params;
    }

    pub fn targets(&self) -> &Params {
        &self.targets
    }

    /// Snapshot of the emergence engine, for visualisation.
    pub fn emergence_snapshot(&self) -> EmergenceSnapshot {
        if self.current_emergence > 0.01 {
            self.emergence.snapshot()
        } else {
            EmergenceSnapshot::empty()
        }
    }

    /// Render `out_l.len()` stereo frames into the two buffers.
    pub fn process_block(&mut self, out_l: &mut [f32], out_r: &mut [f32]) {
        let n = out_l.len().min(out_r.len());
        for i in 0..n {
            let (l, r) = self.process_frame();
            out_l[i] = l;
            out_r[i] = r;
        }
    }

    /// Render one stereo frame. This is the single source of truth for the
    /// signal flow; `process_block` and the CLI callback both ride it.
    pub fn process_frame(&mut self) -> (f32, f32) {
        if !self.targets.playing {
            return (0.0, 0.0);
        }

        // Keep the emergence sub-engine's spawn mode in sync.
        if self.emergence.spawn_mode() != self.targets.spawn_mode {
            self.emergence.set_spawn_mode(self.targets.spawn_mode);
        }

        let a = self.smooth_alpha;
        let t = self.targets; // `Params` is `Copy` — take a snapshot.

        // Smooth parameter transitions.
        self.current_base += (t.base_freq as f64 - self.current_base) * a;
        self.current_beat += (t.beat_freq as f64 - self.current_beat) * a;
        self.current_vol += (t.volume as f64 - self.current_vol) * a;
        self.current_noise += (t.noise_level as f64 - self.current_noise) * a;
        self.current_harmonics += (t.harmonics as f64 - self.current_harmonics) * a;
        self.current_emergence += (t.emergence as f64 - self.current_emergence) * a;
        self.current_gravity += (t.gravity as f64 - self.current_gravity) * a;
        self.current_shepard += (t.shepard as f64 - self.current_shepard) * a;
        self.current_shepard_base += (t.shepard_base_freq as f64 - self.current_shepard_base) * a;

        let target_weights = t.timbre.weights();
        for (i, &tw) in target_weights.iter().enumerate() {
            self.current_harm_weights[i] += (tw - self.current_harm_weights[i]) * a;
        }

        let freq_l = self.current_base;
        let freq_r = self.current_base + self.current_beat;

        // Primary binaural tone.
        let (mut sample_l, mut sample_r) = self.generate_tone(freq_l, freq_r);
        sample_l *= self.current_vol;
        sample_r *= self.current_vol;

        // Emergence voices.
        if self.current_emergence > 0.01 {
            self.emergence_was_active = true;
            let (em_l, em_r) = self.emergence.process(
                self.current_base,
                self.current_emergence,
                self.current_gravity,
                &self.current_harm_weights,
                self.current_harmonics,
            );
            sample_l += em_l * self.current_vol;
            sample_r += em_r * self.current_vol;
        } else if (t.emergence as f64) <= 0.01 && self.emergence_was_active {
            self.emergence.reset();
            self.emergence_was_active = false;
        }

        // Shepard-Risset glissando — mono, mixed equally L/R so it doesn't
        // disturb the binaural phase difference.
        if self.current_shepard > 0.001 {
            let s = self.shepard.process(
                t.shepard_direction,
                self.current_shepard,
                self.current_shepard_base,
            ) * self.current_vol;
            sample_l += s;
            sample_r += s;
        }

        // Mist / noise layer.
        if self.current_noise > 0.001 {
            let noise =
                self.noise.sample(t.mist_type) * self.current_noise * mist_gain(t.mist_type);
            sample_l += noise;
            sample_r += noise;
        }

        // Soft clip to prevent harsh distortion.
        (soft_clip(sample_l) as f32, soft_clip(sample_r) as f32)
    }

    fn generate_tone(&mut self, freq_l: f64, freq_r: f64) -> (f64, f64) {
        self.phase_l += freq_l / self.sample_rate;
        self.phase_r += freq_r / self.sample_rate;
        self.phase_l -= self.phase_l.floor();
        self.phase_r -= self.phase_r.floor();

        let mut sample_l = (TAU * self.phase_l).sin();
        let mut sample_r = (TAU * self.phase_r).sin();

        if self.current_harmonics > 0.01 {
            let mut total_weight = 0.0;
            for i in 0..5 {
                let weight = self.current_harm_weights[i];
                if weight < 0.001 {
                    continue;
                }
                let mult = (i + 2) as f64;
                self.harm_phase_l[i] += (freq_l * mult) / self.sample_rate;
                self.harm_phase_r[i] += (freq_r * mult) / self.sample_rate;
                self.harm_phase_l[i] -= self.harm_phase_l[i].floor();
                self.harm_phase_r[i] -= self.harm_phase_r[i].floor();

                sample_l += (TAU * self.harm_phase_l[i]).sin() * weight * self.current_harmonics;
                sample_r += (TAU * self.harm_phase_r[i]).sin() * weight * self.current_harmonics;
                total_weight += weight;
            }
            let norm = 1.0 + self.current_harmonics * total_weight;
            sample_l /= norm;
            sample_r /= norm;
        }

        (sample_l, sample_r)
    }
}
