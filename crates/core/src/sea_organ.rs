//! Sea Organ Driver
//!
//! A control-rate model of the Zadar Sea Organ (Morske orgulje, Nikola Bašić
//! and Ivica Stamać, 2005): 35 stopped flue pipes in 7 segments of 5 along a
//! stepped seawall. The sea pushes water up each segment's sloped pipe; the
//! water column compresses the trapped air and drives it through the pipe's
//! labium. Nature decides when and how loudly each pipe speaks; the tuning is
//! fixed by geometry.
//!
//! Model (after Krvavica et al., "Time-dependent numerical model for
//! simulating internal oscillations in a sea organ", Ocean Engineering 2020):
//!   η(t, x)  random-phase JONSWAP sea surface, 32 components
//!   l_s(t)   per-segment water-column displacement, a damped resonator
//!            l'' + 2ζω_r l' + ω_r² l = ω_r² η(t, x_s)
//!   pipes    speak when trim·l_s crosses a per-pipe threshold (hysteresis)
//!
//! Segment resonances and damping are Krvavica's Table 2. The chord layout is
//! verified (odd segments G major D–G–d–g–h, even segments C6 C–D–c–e–a); the
//! just ratios, thresholds, wave direction and pitch behaviour are inferred
//! and live here as named constants so they can be retuned by ear.

use std::f64::consts::{PI, TAU};

pub const SEGMENTS: usize = 7;
pub const PIPES_PER_SEGMENT: usize = 5;
pub const PIPES: usize = SEGMENTS * PIPES_PER_SEGMENT;

/// Sea-surface components in the random-phase sum.
const WAVE_COMPONENTS: usize = 32;

/// Audio samples between sea updates. Sea content is 0.1–3 Hz, so even at
/// 192 kHz this stays two orders of magnitude above the signal.
pub const CONTROL_INTERVAL: usize = 64;

/// Water-column resonance per segment, Hz (Krvavica et al., Table 2).
const SEGMENT_RESONANCE_HZ: [f64; SEGMENTS] = [0.215, 0.263, 0.308, 0.275, 0.323, 0.449, 0.528];
/// Damping ratio per segment (Krvavica et al., Table 2).
const SEGMENT_DAMPING: [f64; SEGMENTS] = [0.115, 0.101, 0.103, 0.087, 0.116, 0.137, 0.140];
/// Output trim per segment. A linear resonator over-ranks segment 6 in small
/// seas; 0.6 restores the paper's ordering without breaking the jugo case.
const SEGMENT_TRIM: [f64; SEGMENTS] = [1.0, 1.0, 1.0, 1.0, 1.0, 0.6, 1.0];

/// G-major segments (1, 3, 5, 7): D, G, d, g, h — ratios against d (D3).
const G_MAJOR_RATIOS: [f64; PIPES_PER_SEGMENT] = [1.0 / 2.0, 2.0 / 3.0, 1.0, 4.0 / 3.0, 5.0 / 3.0];
/// C6 segments (2, 4, 6): C, D, c, e, a — ratios against d (D3).
const C_SIXTH_RATIOS: [f64; PIPES_PER_SEGMENT] =
    [4.0 / 9.0, 1.0 / 2.0, 8.0 / 9.0, 9.0 / 8.0, 3.0 / 2.0];

/// JONSWAP peak enhancement and spectral widths.
const JONSWAP_GAMMA: f64 = 3.3;
const JONSWAP_SIGMA_LOW: f64 = 0.07;
const JONSWAP_SIGMA_HIGH: f64 = 0.09;
/// Component band, as multiples of the peak frequency.
const BAND_LOW: f64 = 0.5;
const BAND_HIGH: f64 = 3.0;

/// Significant wave height: Hs = MIN + SPAN · intensity² (m).
/// 0.5 → 0.34 m (light maestral), 1.0 → 1.2 m (beyond the 1.0 m jugo case).
const HS_MIN: f64 = 0.05;
const HS_SPAN: f64 = 1.15;
/// Peak period: Tp = MIN + SPAN · gravity (s). 0.2 → 2.9 s, 1.0 → 4.5 s.
const TP_MIN: f64 = 2.5;
const TP_SPAN: f64 = 2.0;

/// Wave obliquity. Maestral (short) and jugo (long) seas arrive from opposite
/// ends of the wall, so the sweep direction flips around this period.
const DIRECTION_MAX: f64 = PI / 4.0;
const DIRECTION_FLIP_TP: f64 = 3.7;
const DIRECTION_FLIP_WIDTH: f64 = 0.3;
const GRAVITY_ACCEL: f64 = 9.81;

/// Pipe geometry along the wall (m). Pipes are 1.5 m apart; segment centres
/// are spread so the whole organ spans about 70 m.
const SEGMENT_SPACING: f64 = 10.5;
const PIPE_SPACING: f64 = 1.5;
/// Listener stands centred, this far back from the wall (m).
const LISTENER_DISTANCE: f64 = 4.0;
/// Distance gain is (LISTENER_DISTANCE / r)^ROLLOFF; 1.0 would be free field,
/// softened so the far segments stay audible.
const DISTANCE_ROLLOFF: f64 = 0.5;
const MAX_PAN: f64 = 0.85;

/// A pipe speaks when trim·l crosses its threshold (m of water column) and
/// falls silent below OFF_RATIO of it. Within a segment the five thresholds
/// are graded so small crests sound one or two pipes and big ones all five.
const SPEAK_THRESHOLD: f64 = 0.20;
const THRESHOLD_GRADE: f64 = 0.75;
const THRESHOLD_JITTER: f64 = 0.10;
const OFF_RATIO: f64 = 0.7;
/// Displacement above the off threshold that takes a pipe to full voice.
const SPEAK_RANGE: f64 = 0.66;
/// Relative level while the water falls and air is drawn back in. Unknown at
/// Zadar; a quieter inflow keeps the per-wave "breath" audible.
const INFLOW_GAIN: f64 = 0.35;

/// Fixed seed so a given session always meets the same sea.
const SEA_SEED: u64 = 0x5EA0_7A0A_2005_0415;

/// Map the emergence intensity (0–1) to significant wave height (m).
pub fn wave_height(intensity: f64) -> f64 {
    let i = intensity.clamp(0.0, 1.0);
    HS_MIN + HS_SPAN * i * i
}

/// Map gravity (0–1), repurposed in Sea Organ mode, to peak period (s).
pub fn peak_period(gravity: f64) -> f64 {
    TP_MIN + TP_SPAN * gravity.clamp(0.0, 1.0)
}

/// Pipe frequency as a ratio of the base frequency.
pub fn pipe_ratio(pipe: usize) -> f64 {
    let segment = pipe / PIPES_PER_SEGMENT;
    let slot = pipe % PIPES_PER_SEGMENT;
    if segment.is_multiple_of(2) {
        G_MAJOR_RATIOS[slot]
    } else {
        C_SIXTH_RATIOS[slot]
    }
}

/// Pipe position along the wall (m), centred on the listener.
fn pipe_position(pipe: usize) -> f64 {
    let segment = (pipe / PIPES_PER_SEGMENT) as f64;
    let slot = (pipe % PIPES_PER_SEGMENT) as f64;
    (segment - 3.0) * SEGMENT_SPACING + (slot - 2.0) * PIPE_SPACING
}

/// Stereo pan for a pipe heard from the centre of the promenade.
pub fn pipe_pan(pipe: usize) -> f64 {
    let x = pipe_position(pipe);
    ((x / LISTENER_DISTANCE).atan() / (PI * 0.5)).clamp(-MAX_PAN, MAX_PAN)
}

/// Distance attenuation for a pipe heard from the centre of the promenade.
pub fn pipe_distance_gain(pipe: usize) -> f64 {
    let x = pipe_position(pipe);
    let r = (x * x + LISTENER_DISTANCE * LISTENER_DISTANCE).sqrt();
    (LISTENER_DISTANCE / r).powf(DISTANCE_ROLLOFF)
}

/// Normalised JONSWAP shape at r = f / fp.
fn jonswap_shape(r: f64) -> f64 {
    let sigma = if r <= 1.0 {
        JONSWAP_SIGMA_LOW
    } else {
        JONSWAP_SIGMA_HIGH
    };
    let peak = (-(r - 1.0).powi(2) / (2.0 * sigma * sigma)).exp();
    r.powi(-5) * (-1.25 * r.powi(-4)).exp() * JONSWAP_GAMMA.powf(peak)
}

#[derive(Clone, Copy, Default)]
struct WaveComponent {
    /// Frequency as a multiple of the peak frequency.
    ratio: f64,
    /// Amplitude per metre of Hs.
    amplitude: f64,
    phase: f64,
}

#[derive(Clone, Copy, Default)]
struct Segment {
    displacement: f64,
    velocity: f64,
}

#[derive(Clone, Copy, Default)]
struct Pipe {
    threshold: f64,
    latched: bool,
    onset: bool,
}

/// Fixed-size state for the sea and its 35 pipes. No allocation after `new`.
pub struct SeaOrganState {
    components: [WaveComponent; WAVE_COMPONENTS],
    segments: [Segment; SEGMENTS],
    pipes: [Pipe; PIPES],
    rng: u64,
}

impl SeaOrganState {
    pub fn new() -> Self {
        let mut state = Self {
            components: [WaveComponent::default(); WAVE_COMPONENTS],
            segments: [Segment::default(); SEGMENTS],
            pipes: [Pipe::default(); PIPES],
            rng: SEA_SEED,
        };
        state.seed();
        state
    }

    pub fn reset(&mut self) {
        self.rng = SEA_SEED;
        self.segments = [Segment::default(); SEGMENTS];
        self.pipes = [Pipe::default(); PIPES];
        self.seed();
    }

    fn xorshift(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng as f64) / (u64::MAX as f64)
    }

    /// Draw component frequencies, phases and pipe thresholds.
    fn seed(&mut self) {
        // Jittered frequencies keep the sum from repeating; amplitudes follow
        // a_i = √(2 S(f_i) Δf) normalised so Σ a_i²/2 = (Hs/4)² at Hs = 1.
        let width = (BAND_HIGH - BAND_LOW) / WAVE_COMPONENTS as f64;
        let mut variance = 0.0;
        for i in 0..WAVE_COMPONENTS {
            let ratio = BAND_LOW + width * (i as f64 + self.xorshift());
            let energy = jonswap_shape(ratio) * width;
            self.components[i] = WaveComponent {
                ratio,
                amplitude: energy,
                phase: self.xorshift() * TAU,
            };
            variance += energy;
        }
        let target_variance = 1.0 / 16.0;
        for component in &mut self.components {
            component.amplitude = (2.0 * target_variance * component.amplitude / variance).sqrt();
        }

        for segment in 0..SEGMENTS {
            // A seeded permutation decides which pipe in each segment is the
            // most eager to speak.
            let mut ranks = [0, 1, 2, 3, 4];
            for i in (1..PIPES_PER_SEGMENT).rev() {
                let j = (self.xorshift() * (i + 1) as f64) as usize;
                ranks.swap(i, j.min(i));
            }
            for (slot, &rank) in ranks.iter().enumerate() {
                let grade = 1.0 + THRESHOLD_GRADE * rank as f64 / (PIPES_PER_SEGMENT - 1) as f64;
                let jitter = 1.0 + THRESHOLD_JITTER * self.xorshift();
                self.pipes[segment * PIPES_PER_SEGMENT + slot].threshold =
                    SPEAK_THRESHOLD * grade * jitter;
            }
        }
    }

    /// Advance the sea by `dt` seconds with significant wave height `hs` (m)
    /// and peak period `tp` (s). Updates every pipe's latch and onset flag.
    pub fn tick(&mut self, dt: f64, hs: f64, tp: f64) {
        let fp = 1.0 / tp.max(0.5);
        let theta = DIRECTION_MAX * ((tp - DIRECTION_FLIP_TP) / DIRECTION_FLIP_WIDTH).tanh();
        let obliquity = theta.sin();

        let mut surface = [0.0; SEGMENTS];
        for component in &mut self.components {
            let omega = TAU * fp * component.ratio;
            component.phase = (component.phase + omega * dt) % TAU;
            // Deep-water dispersion: k = ω²/g.
            let k = omega * omega / GRAVITY_ACCEL;
            let amplitude = component.amplitude * hs;
            for (segment, eta) in surface.iter_mut().enumerate() {
                let x = (segment as f64 - 3.0) * SEGMENT_SPACING;
                *eta += amplitude * (component.phase - k * x * obliquity).cos();
            }
        }

        for (segment, state) in self.segments.iter_mut().enumerate() {
            let omega_r = TAU * SEGMENT_RESONANCE_HZ[segment];
            let zeta = SEGMENT_DAMPING[segment];
            // Semi-implicit Euler: stable for ω_r·dt ≪ 1, which holds here.
            let accel = omega_r * omega_r * (surface[segment] - state.displacement)
                - 2.0 * zeta * omega_r * state.velocity;
            state.velocity += accel * dt;
            state.displacement += state.velocity * dt;
        }

        for pipe in 0..PIPES {
            let level = self.segment_level(pipe / PIPES_PER_SEGMENT);
            let state = &mut self.pipes[pipe];
            let was_latched = state.latched;
            if level >= state.threshold {
                state.latched = true;
            } else if level < state.threshold * OFF_RATIO {
                state.latched = false;
            }
            state.onset = state.latched && !was_latched;
        }
    }

    /// Trimmed water-column displacement for a segment (m).
    pub fn segment_level(&self, segment: usize) -> f64 {
        self.segments[segment].displacement * SEGMENT_TRIM[segment]
    }

    pub fn pipe_latched(&self, pipe: usize) -> bool {
        self.pipes[pipe].latched
    }

    /// True on the tick a pipe starts speaking.
    pub fn pipe_onset(&self, pipe: usize) -> bool {
        self.pipes[pipe].onset
    }

    /// Normalised drive (0–1) above the pipe's off threshold. Tone follows a
    /// smoothstep of it; breath follows its square root.
    pub fn pipe_drive(&self, pipe: usize) -> f64 {
        let segment = pipe / PIPES_PER_SEGMENT;
        let off = self.pipes[pipe].threshold * OFF_RATIO;
        let drive = ((self.segment_level(segment) - off) / SPEAK_RANGE).clamp(0.0, 1.0);
        // Rising water pushes air out through the labium; falling water draws
        // it back in, more quietly.
        let velocity = self.segments[segment].velocity * SEGMENT_TRIM[segment];
        let omega_r = TAU * SEGMENT_RESONANCE_HZ[segment];
        let rising = 0.5 + 0.5 * (velocity / (omega_r * 0.05)).tanh();
        drive * (INFLOW_GAIN + (1.0 - INFLOW_GAIN) * rising)
    }
}

impl Default for SeaOrganState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DT: f64 = 0.01;

    /// RMS trimmed displacement and speaking fraction per segment.
    fn run_sea(hs: f64, tp: f64, seconds: f64) -> ([f64; SEGMENTS], [f64; SEGMENTS]) {
        let mut sea = SeaOrganState::new();
        let warmup = (60.0 / TEST_DT) as usize;
        let steps = (seconds / TEST_DT) as usize;
        let mut sum_sq = [0.0; SEGMENTS];
        let mut speaking = [0.0; SEGMENTS];
        for step in 0..warmup + steps {
            sea.tick(TEST_DT, hs, tp);
            if step < warmup {
                continue;
            }
            for segment in 0..SEGMENTS {
                sum_sq[segment] += sea.segment_level(segment).powi(2);
                let base = segment * PIPES_PER_SEGMENT;
                if (base..base + PIPES_PER_SEGMENT).any(|pipe| sea.pipe_latched(pipe)) {
                    speaking[segment] += 1.0;
                }
            }
        }
        let n = steps as f64;
        (sum_sq.map(|s| (s / n).sqrt()), speaking.map(|s| s / n))
    }

    #[test]
    fn surface_matches_requested_wave_height() {
        let mut sea = SeaOrganState::new();
        let hs = 0.4;
        let mut sum_sq = 0.0;
        let steps = (600.0 / TEST_DT) as usize;
        for _ in 0..steps {
            sea.tick(TEST_DT, hs, 2.9);
            let eta: f64 = sea
                .components
                .iter()
                .map(|c| c.amplitude * hs * c.phase.cos())
                .sum();
            sum_sq += eta * eta;
        }
        let measured_hs = 4.0 * (sum_sq / steps as f64).sqrt();
        assert!((measured_hs - hs).abs() < 0.05, "Hs {measured_hs}");
    }

    #[test]
    fn maestral_sea_favours_middle_segments_over_the_first() {
        let (rms, speaking) = run_sea(0.4, 2.9, 600.0);
        assert!(rms[2] > rms[0] && rms[4] > rms[0], "rms {rms:?}");
        let quietest = (0..SEGMENTS)
            .min_by(|&a, &b| rms[a].total_cmp(&rms[b]))
            .unwrap();
        assert_eq!(quietest, 0, "rms {rms:?}");
        assert!(
            speaking[0] < 0.05,
            "segment 1 should be nearly silent: {speaking:?}"
        );
    }

    #[test]
    fn jugo_sea_wakes_the_first_segment_and_quiets_the_last() {
        let (rms, speaking) = run_sea(1.0, 4.5, 600.0);
        let loudest = (0..SEGMENTS)
            .max_by(|&a, &b| rms[a].total_cmp(&rms[b]))
            .unwrap();
        assert_eq!(loudest, 3, "rms {rms:?}");
        for quiet in [5, 6] {
            for louder in 0..5 {
                assert!(rms[quiet] < rms[louder], "rms {rms:?}");
            }
        }
        assert!(
            speaking[0] > 0.2,
            "segment 1 should speak in jugo: {speaking:?}"
        );
    }

    #[test]
    fn calm_sea_keeps_every_pipe_silent() {
        let (_, speaking) = run_sea(wave_height(0.1), peak_period(0.5), 600.0);
        assert!(speaking.iter().all(|&s| s == 0.0), "{speaking:?}");
    }

    #[test]
    fn pipe_layout_alternates_g_major_and_c_sixth() {
        let d3 = 146.83;
        let g_segment: Vec<f64> = (0..5).map(|p| d3 * pipe_ratio(p)).collect();
        let c_segment: Vec<f64> = (5..10).map(|p| d3 * pipe_ratio(p)).collect();
        // D2 G2 D3 G3 B3 and C2 D2 C3 E3 A3, within 16 cents of 12-TET (just B is the outlier).
        let g_expected = [73.42, 97.99, 146.83, 196.0, 246.94];
        let c_expected = [65.41, 73.42, 130.81, 164.81, 220.0];
        for (actual, expected) in g_segment
            .iter()
            .zip(g_expected)
            .chain(c_segment.iter().zip(c_expected))
        {
            let cents = 1200.0 * (actual / expected).log2();
            assert!(cents.abs() < 16.0, "{actual} vs {expected}");
        }
        assert_eq!(pipe_ratio(30), pipe_ratio(0));
    }

    #[test]
    fn mappings_hit_the_reference_sea_states() {
        assert!((wave_height(0.5) - 0.3375).abs() < 1.0e-9);
        assert!((peak_period(0.2) - 2.9).abs() < 1.0e-9);
        assert!((peak_period(1.0) - 4.5).abs() < 1.0e-9);
    }

    #[test]
    fn far_pipes_pan_wide_and_sound_quieter() {
        assert!(pipe_pan(0) < -0.8 && pipe_pan(PIPES - 1) > 0.8);
        assert!(pipe_pan(17).abs() < 1.0e-9);
        assert!(pipe_distance_gain(0) < pipe_distance_gain(17));
        assert!((pipe_distance_gain(17) - 1.0).abs() < 1.0e-9);
    }
}
