use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, Stream, StreamConfig};

use crate::app::{AudioParams, MistType, Timbre, VizBuffer};
use crate::emergence::{EmergenceEngine, EmergenceSnapshot, SpawnMode};
use crate::shepard::{Direction, ShepardEngine};

struct SynthState {
    phase_l: f64,
    phase_r: f64,
    harm_phase_l: [f64; 5],
    harm_phase_r: [f64; 5],
    current_harm_weights: [f64; 5],
    sample_rate: f64,
    current_base: f64,
    current_beat: f64,
    current_vol: f64,
    current_noise: f64,
    current_harmonics: f64,
    current_emergence: f64,
    current_shepard: f64,
    pink_state: [f64; 7],
    pink_counter: u32,
    brown_state: f64,
    last_white: f64,
    last_white_2: f64,
    velvet_state: f64,
    rng: u64,
    viz_counter: u32,
    emergence: EmergenceEngine,
    shepard: ShepardEngine,
    snapshot_counter: u32,
    emergence_was_active: bool,
}

impl SynthState {
    fn new(sample_rate: f64) -> Self {
        Self {
            phase_l: 0.0,
            phase_r: 0.0,
            harm_phase_l: [0.0; 5],
            harm_phase_r: [0.0; 5],
            current_harm_weights: Timbre::Organ.weights(),
            sample_rate,
            current_base: 220.0,
            current_beat: 10.0,
            current_vol: 0.7,
            current_noise: 0.0,
            current_harmonics: 0.3,
            current_emergence: 0.0,
            current_shepard: 0.0,
            pink_state: [0.0; 7],
            pink_counter: 0,
            brown_state: 0.0,
            last_white: 0.0,
            last_white_2: 0.0,
            velvet_state: 0.0,
            rng: 0xDEADBEEFCAFE1234,
            viz_counter: 0,
            emergence: EmergenceEngine::new(sample_rate),
            shepard: ShepardEngine::new(sample_rate),
            snapshot_counter: 0,
            emergence_was_active: false,
        }
    }

    fn xorshift64(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng as f64) / (u64::MAX as f64) * 2.0 - 1.0
    }

    fn pink_noise(&mut self) -> f64 {
        self.pink_counter = self.pink_counter.wrapping_add(1);
        let mut sum = 0.0;
        for i in 0..7 {
            if self.pink_counter & (1 << i) == 0 {
                self.pink_state[i] = self.xorshift64();
            }
            sum += self.pink_state[i];
        }
        sum / 7.0
    }

    fn mist_sample(&mut self, mist_type: MistType) -> f64 {
        match mist_type {
            MistType::Pink => self.pink_noise(),
            MistType::White => self.xorshift64() * 0.58,
            MistType::Brown => {
                let white = self.xorshift64();
                self.brown_state = (self.brown_state * 0.996 + white * 0.035).clamp(-1.0, 1.0);
                self.brown_state * 1.4
            }
            MistType::Blue => {
                let white = self.xorshift64();
                let blue = white - self.last_white * 0.72 + self.last_white_2 * 0.12;
                self.last_white_2 = self.last_white;
                self.last_white = white;
                blue * 0.42
            }
            MistType::Velvet => {
                let trigger = (self.xorshift64() + 1.0) * 0.5;
                if trigger < 0.0025 {
                    self.velvet_state = if self.xorshift64() >= 0.0 { 1.0 } else { -1.0 };
                }
                let sample = self.velvet_state;
                self.velvet_state *= 0.88;
                sample * 0.75
            }
        }
    }

    fn generate_tone(&mut self, freq_l: f64, freq_r: f64) -> (f64, f64) {
        use std::f64::consts::TAU;

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

pub struct AudioEngine {
    _stream: Stream,
}

impl AudioEngine {
    pub fn new(
        params: Arc<AudioParams>,
        viz_buffer: Arc<Mutex<VizBuffer>>,
        emergence_snapshot: Arc<Mutex<EmergenceSnapshot>>,
    ) -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No audio output device found")?;

        let supported = device
            .supported_output_configs()
            .map_err(|e| format!("Failed to query audio configs: {e}"))?
            .find(|c| c.channels() == 2 && c.sample_format() == SampleFormat::F32)
            .or_else(|| {
                device
                    .supported_output_configs()
                    .ok()?
                    .find(|c| c.channels() == 2)
            })
            .ok_or("No suitable stereo output config found")?;

        let config: StreamConfig =
            if supported.min_sample_rate().0 <= 48_000 && supported.max_sample_rate().0 >= 48_000 {
                supported.with_sample_rate(SampleRate(48_000)).into()
            } else {
                supported.with_max_sample_rate().into()
            };
        let sample_rate = config.sample_rate.0 as f64;
        let channels = config.channels as usize;

        let mut state = SynthState::new(sample_rate);
        let smooth_alpha = 1.0 - (-1.0 / (sample_rate * 0.05)).exp();

        // Snapshot update rate: ~30 times per second
        let snapshot_interval = (sample_rate / 30.0) as u32;

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let playing = params.playing.load(Ordering::Relaxed);
                    if !playing {
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                        return;
                    }

                    let target_base =
                        f32::from_bits(params.base_freq.load(Ordering::Relaxed)) as f64;
                    let target_beat =
                        f32::from_bits(params.beat_freq.load(Ordering::Relaxed)) as f64;
                    let target_vol = f32::from_bits(params.volume.load(Ordering::Relaxed)) as f64;
                    let target_noise =
                        f32::from_bits(params.noise_level.load(Ordering::Relaxed)) as f64;
                    let target_mist_type =
                        MistType::from_u32(params.mist_type.load(Ordering::Relaxed));
                    let target_harmonics =
                        f32::from_bits(params.harmonics.load(Ordering::Relaxed)) as f64;
                    let target_emergence =
                        f32::from_bits(params.emergence.load(Ordering::Relaxed)) as f64;
                    let target_spawn_mode =
                        SpawnMode::from_u32(params.spawn_mode.load(Ordering::Relaxed));
                    if state.emergence.spawn_mode() != target_spawn_mode {
                        state.emergence.set_spawn_mode(target_spawn_mode);
                    }
                    let target_shepard =
                        f32::from_bits(params.shepard.load(Ordering::Relaxed)) as f64;
                    let shepard_direction =
                        Direction::from_u32(params.shepard_direction.load(Ordering::Relaxed));
                    let target_timbre = params.get_timbre();
                    let target_weights = target_timbre.weights();

                    for frame in data.chunks_mut(channels) {
                        // Smooth parameter transitions
                        state.current_base += (target_base - state.current_base) * smooth_alpha;
                        state.current_beat += (target_beat - state.current_beat) * smooth_alpha;
                        state.current_vol += (target_vol - state.current_vol) * smooth_alpha;
                        state.current_noise += (target_noise - state.current_noise) * smooth_alpha;
                        state.current_harmonics +=
                            (target_harmonics - state.current_harmonics) * smooth_alpha;
                        state.current_emergence +=
                            (target_emergence - state.current_emergence) * smooth_alpha;
                        state.current_shepard +=
                            (target_shepard - state.current_shepard) * smooth_alpha;

                        for (i, &target_weight) in target_weights.iter().enumerate() {
                            state.current_harm_weights[i] +=
                                (target_weight - state.current_harm_weights[i]) * smooth_alpha;
                        }

                        let freq_l = state.current_base;
                        let freq_r = state.current_base + state.current_beat;

                        // Primary binaural tone
                        let (mut sample_l, mut sample_r) = state.generate_tone(freq_l, freq_r);
                        sample_l *= state.current_vol;
                        sample_r *= state.current_vol;

                        // Emergence voices
                        if state.current_emergence > 0.01 {
                            state.emergence_was_active = true;
                            let (em_l, em_r) = state.emergence.process(
                                state.current_base,
                                state.current_emergence,
                                &state.current_harm_weights,
                                state.current_harmonics,
                            );
                            sample_l += em_l * state.current_vol;
                            sample_r += em_r * state.current_vol;
                        } else if target_emergence <= 0.01 && state.emergence_was_active {
                            state.emergence.reset();
                            state.emergence_was_active = false;
                        }

                        // Shepard-Risset glissando — mono, mixed equally L/R
                        // so it doesn't disturb the binaural phase difference.
                        if state.current_shepard > 0.001 {
                            let s = state
                                .shepard
                                .process(shepard_direction, state.current_shepard)
                                * state.current_vol;
                            sample_l += s;
                            sample_r += s;
                        }

                        // Mist/noise layer
                        if state.current_noise > 0.001 {
                            let noise = state.mist_sample(target_mist_type)
                                * state.current_noise
                                * mist_gain(target_mist_type);
                            sample_l += noise;
                            sample_r += noise;
                        }

                        // Soft clip to prevent harsh distortion
                        sample_l = soft_clip(sample_l);
                        sample_r = soft_clip(sample_r);

                        frame[0] = sample_l as f32;
                        if channels > 1 {
                            frame[1] = sample_r as f32;
                        }

                        // Viz buffer
                        state.viz_counter += 1;
                        if state.viz_counter.is_multiple_of(4)
                            && let Ok(mut buf) = viz_buffer.try_lock()
                        {
                            buf.push(sample_l as f32, sample_r as f32);
                        }

                        // Emergence snapshot (periodic)
                        state.snapshot_counter += 1;
                        if state.snapshot_counter >= snapshot_interval {
                            state.snapshot_counter = 0;
                            if let Ok(mut snap) = emergence_snapshot.try_lock() {
                                if state.current_emergence > 0.01 {
                                    *snap = state.emergence.snapshot();
                                } else if !snap.voices.is_empty() || snap.total_energy > 0.0 {
                                    *snap = EmergenceSnapshot::empty();
                                }
                            }
                        }
                    }
                },
                |err| {
                    eprintln!("Audio stream error: {err}");
                },
                None,
            )
            .map_err(|e| format!("Failed to build audio stream: {e}"))?;

        stream
            .play()
            .map_err(|e| format!("Failed to start audio: {e}"))?;

        Ok(Self { _stream: stream })
    }
}

/// Soft clipper using tanh - prevents harsh digital clipping.
#[inline]
fn soft_clip(x: f64) -> f64 {
    if x.abs() < 0.9 {
        x // Fast path: no processing needed for normal levels
    } else {
        x.tanh()
    }
}

#[inline]
fn mist_gain(mist_type: MistType) -> f64 {
    match mist_type {
        MistType::Pink => 0.30,
        MistType::White => 0.24,
        MistType::Brown => 0.22,
        MistType::Blue => 0.20,
        MistType::Velvet => 0.26,
    }
}
