//! Shepard-Risset Glissando
//!
//! A stack of octave-spaced sine oscillators sweeping in parallel through
//! a raised-cosine amplitude window — Risset's continuous variant of
//! Shepard's auditory illusion of a tone that rises (or falls) forever.
//!
//! Each oscillator carries:
//!   - a log-frequency offset x_i ∈ [0, NUM_OCTAVES) that drifts at `rate`
//!     octaves per second (sign chosen by `Direction`),
//!   - a phase accumulator for the corresponding sine wave at
//!     base_freq · 2^x_i.
//!
//! When x_i crosses the upper bound it wraps to the lower bound (and
//! vice-versa for descending). Because the bell-shaped envelope is
//! ~zero at the edges, the wrap is inaudible — that is the illusion.
//!
//! The N oscillators start evenly spaced across the range and stay so
//! at every instant (they all share the same drift), so the raised-cosine
//! window has stable total energy and we can normalize once at construction.
//!
//! The output is summed mono — the binaural beat does the stereo work,
//! and adding stereo motion to the Shepard layer would only blur the
//! difference frequency.

use std::f64::consts::TAU;

const NUM_OCTAVES: usize = 7;
/// C1. Seven octaves from here tops out just above C8, keeping the
/// illusion present without a 6-7 kHz whistle.
pub const DEFAULT_BASE_FREQ_HZ: f64 = 32.703_195_662_574_83;
/// C0 through C3 keeps the stack in a musically useful range.
pub const MIN_BASE_FREQ_HZ: f64 = DEFAULT_BASE_FREQ_HZ * 0.5;
pub const MAX_BASE_FREQ_HZ: f64 = DEFAULT_BASE_FREQ_HZ * 4.0;
const RANGE_OCTAVES: f64 = NUM_OCTAVES as f64;
/// Default sweep rate — 36 seconds per octave.
pub const DEFAULT_RATE_OCT_PER_SEC: f64 = 1.0 / 36.0;
/// Output gain applied after envelope normalization. Keeps the layer
/// audible at intensity=1.0 without dominating the binaural carrier.
const OUTPUT_GAIN: f64 = 0.30;
const PHASE_STEP: f64 = 0.618_033_988_749_894_9;

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Up => "rising",
            Self::Down => "falling",
        }
    }

    pub fn glyph(self) -> &'static str {
        match self {
            Self::Up => "\u{2191}",   // ↑
            Self::Down => "\u{2193}", // ↓
        }
    }

    pub fn flipped(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }

    pub fn from_u32(v: u32) -> Self {
        if v == 1 { Self::Down } else { Self::Up }
    }
}

pub struct ShepardEngine {
    sample_rate: f64,
    /// Per-oscillator log-frequency offset, in [0, RANGE_OCTAVES).
    log_offsets: [f64; NUM_OCTAVES],
    /// Per-oscillator phase accumulator (turns, mod 1).
    phases: [f64; NUM_OCTAVES],
    /// Octaves per second drift (positive; sign comes from Direction).
    rate: f64,
    /// Equal-power normalizer for the evenly-spaced raised-cosine windows.
    normalizer: f64,
}

impl ShepardEngine {
    pub fn new(sample_rate: f64) -> Self {
        let mut log_offsets = [0.0; NUM_OCTAVES];
        let mut phases = [0.0; NUM_OCTAVES];
        for (i, slot) in log_offsets.iter_mut().enumerate() {
            *slot = i as f64;
            phases[i] = ((i as f64 + 0.5) * PHASE_STEP).fract();
        }

        let mut energy_sum = 0.0;
        for i in 0..NUM_OCTAVES {
            let env = envelope(i as f64);
            energy_sum += env * env;
        }
        let normalizer = 1.0 / energy_sum.max(1e-6).sqrt();

        Self {
            sample_rate,
            log_offsets,
            phases,
            rate: DEFAULT_RATE_OCT_PER_SEC,
            normalizer,
        }
    }

    /// Advance every oscillator by one sample period and return the
    /// summed mono Shepard contribution scaled by `intensity ∈ [0, 1]`.
    pub fn process(&mut self, direction: Direction, intensity: f64, base_freq: f64) -> f64 {
        if intensity <= 0.0 {
            return 0.0;
        }
        let base_freq = base_freq.clamp(MIN_BASE_FREQ_HZ, MAX_BASE_FREQ_HZ);
        let dt = 1.0 / self.sample_rate;
        let drift = match direction {
            Direction::Up => self.rate * dt,
            Direction::Down => -self.rate * dt,
        };

        let mut sum = 0.0;
        for i in 0..NUM_OCTAVES {
            self.log_offsets[i] += drift;
            if self.log_offsets[i] >= RANGE_OCTAVES {
                self.log_offsets[i] -= RANGE_OCTAVES;
            } else if self.log_offsets[i] < 0.0 {
                self.log_offsets[i] += RANGE_OCTAVES;
            }

            let freq = base_freq * (2.0_f64).powf(self.log_offsets[i]);
            self.phases[i] += freq * dt;
            self.phases[i] -= self.phases[i].floor();

            let env = envelope(self.log_offsets[i]);

            sum += (TAU * self.phases[i]).sin() * env;
        }

        sum * self.normalizer * OUTPUT_GAIN * intensity
    }
}

/// Raised-cosine spectral window: `sin⁴(π · log_offset / N)`.
///
/// Returns 0 at `log_offset = 0` and `log_offset = RANGE_OCTAVES`, peaks at
/// the middle. Public so the Knowledge-tab playground can plot it.
pub fn envelope(log_offset: f64) -> f64 {
    let phase = (std::f64::consts::PI * log_offset / RANGE_OCTAVES).sin();
    phase * phase * phase * phase
}

/// Number of octave-spaced oscillators stacked in the Shepard layer.
pub const fn num_octaves() -> usize {
    NUM_OCTAVES
}

/// Width of the log-frequency window the oscillators sweep through.
pub const fn range_octaves() -> f64 {
    RANGE_OCTAVES
}

/// Default lowest oscillator frequency in Hz (when `log_offset == 0`).
pub const fn f_min() -> f64 {
    DEFAULT_BASE_FREQ_HZ
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offsets_stay_in_range() {
        let mut engine = ShepardEngine::new(48_000.0);
        for _ in 0..(48_000 * 60) {
            // 1 minute — multiple wrap cycles
            let _ = engine.process(Direction::Up, 1.0, DEFAULT_BASE_FREQ_HZ);
        }
        for &x in engine.log_offsets.iter() {
            assert!((0.0..RANGE_OCTAVES).contains(&x), "log offset escaped: {x}");
        }
    }

    #[test]
    fn descending_wraps_through_zero() {
        let mut engine = ShepardEngine::new(48_000.0);
        // Drive descending for long enough to cross the lower wrap.
        for _ in 0..(48_000 * 60) {
            let _ = engine.process(Direction::Down, 1.0, DEFAULT_BASE_FREQ_HZ);
        }
        for &x in engine.log_offsets.iter() {
            assert!((0.0..RANGE_OCTAVES).contains(&x));
        }
    }

    #[test]
    fn output_bounded_at_full_intensity() {
        let mut engine = ShepardEngine::new(48_000.0);
        let mut peak: f64 = 0.0;
        for _ in 0..(48_000 * 4) {
            let s = engine
                .process(Direction::Up, 1.0, DEFAULT_BASE_FREQ_HZ)
                .abs();
            if s > peak {
                peak = s;
            }
        }
        // Should sit comfortably below clipping; OUTPUT_GAIN keeps headroom.
        assert!(peak < 1.0, "peak too hot: {peak}");
    }

    #[test]
    fn zero_intensity_silent() {
        let mut engine = ShepardEngine::new(48_000.0);
        for _ in 0..1024 {
            assert_eq!(
                engine.process(Direction::Up, 0.0, DEFAULT_BASE_FREQ_HZ),
                0.0
            );
        }
    }

    #[test]
    fn offsets_remain_evenly_spaced() {
        // The ring of offsets should stay an arithmetic progression
        // (mod RANGE_OCTAVES) at every instant.
        let mut engine = ShepardEngine::new(48_000.0);
        for _ in 0..7919 {
            let _ = engine.process(Direction::Up, 0.5, DEFAULT_BASE_FREQ_HZ);
        }
        let mut diffs: Vec<f64> = Vec::new();
        for i in 0..NUM_OCTAVES {
            let next = (engine.log_offsets[(i + 1) % NUM_OCTAVES] - engine.log_offsets[i]
                + RANGE_OCTAVES)
                % RANGE_OCTAVES;
            diffs.push(next);
        }
        let mean: f64 = diffs.iter().sum::<f64>() / diffs.len() as f64;
        for d in diffs {
            assert!((d - mean).abs() < 1e-9, "offsets drifted apart");
        }
    }

    #[test]
    fn envelope_is_silent_at_wrap_points() {
        assert!(envelope(0.0).abs() < 1e-12);
        assert!(envelope(RANGE_OCTAVES).abs() < 1e-12);
        assert!(envelope(RANGE_OCTAVES * 0.5) > 0.999);
    }

    #[test]
    fn envelope_energy_is_stable_while_drifting() {
        let baseline: f64 = (0..NUM_OCTAVES)
            .map(|i| {
                let env = envelope(i as f64);
                env * env
            })
            .sum();

        for step in 0..64 {
            let drift = step as f64 / 64.0;
            let energy: f64 = (0..NUM_OCTAVES)
                .map(|i| {
                    let env = envelope((i as f64 + drift) % RANGE_OCTAVES);
                    env * env
                })
                .sum();
            assert!(
                (energy - baseline).abs() < 1e-12,
                "energy changed at drift {drift}: {energy} vs {baseline}"
            );
        }
    }

    #[test]
    fn octave_phases_are_decorrelated() {
        let engine = ShepardEngine::new(48_000.0);
        for i in 0..NUM_OCTAVES {
            for j in (i + 1)..NUM_OCTAVES {
                let distance = (engine.phases[i] - engine.phases[j]).abs();
                let wrapped = distance.min(1.0 - distance);
                assert!(wrapped > 0.05, "phases too close: {i}, {j}");
            }
        }
    }
}
