//! Native (cpal) audio backend.
//!
//! All synthesis now lives in `microtube-core`; this file is just the glue
//! between the lock-free [`AudioParams`] the UI writes and the shared
//! [`Engine`]. Each callback mirrors the atomics into the engine's target
//! parameters, then pumps frames one at a time so the visualisation ring
//! buffer and emergence snapshot stay sample-accurate.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, SampleRate, Stream, StreamConfig};

use crate::app::{AudioParams, VizBuffer};
use crate::emergence::EmergenceSnapshot;
use microtube_core::engine::{Engine, Params};

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

        let mut engine = Engine::new(sample_rate);

        // Snapshot update rate: ~30 times per second.
        let snapshot_interval = (sample_rate / 30.0) as u32;
        let mut viz_counter: u32 = 0;
        let mut snapshot_counter: u32 = 0;

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    // Mirror the UI's lock-free atomics into the engine.
                    *engine.targets_mut() = read_params(&params);

                    for frame in data.chunks_mut(channels) {
                        let (sample_l, sample_r) = engine.process_frame();
                        frame[0] = sample_l;
                        if channels > 1 {
                            frame[1] = sample_r;
                        }

                        // Visualisation ring buffer (decimated 4:1).
                        viz_counter += 1;
                        if viz_counter.is_multiple_of(4)
                            && let Ok(mut buf) = viz_buffer.try_lock()
                        {
                            buf.push(sample_l, sample_r);
                        }

                        // Emergence snapshot (periodic).
                        snapshot_counter += 1;
                        if snapshot_counter >= snapshot_interval {
                            snapshot_counter = 0;
                            if let Ok(mut snap) = emergence_snapshot.try_lock() {
                                *snap = engine.emergence_snapshot();
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

/// Snapshot the lock-free UI parameters into a plain `Params` struct.
fn read_params(params: &AudioParams) -> Params {
    Params {
        playing: params.playing.load(Ordering::Relaxed),
        base_freq: params.get_base_freq(),
        beat_freq: params.get_beat_freq(),
        volume: params.get_volume(),
        noise_level: params.get_noise_level(),
        mist_type: params.get_mist_type(),
        harmonics: params.get_harmonics(),
        emergence: params.get_emergence(),
        spawn_mode: params.get_spawn_mode(),
        shepard: params.get_shepard(),
        shepard_base_freq: params.get_shepard_base_freq(),
        shepard_direction: params.get_shepard_direction(),
        timbre: params.get_timbre(),
    }
}
