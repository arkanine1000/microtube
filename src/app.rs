use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::emergence::{EmergenceSnapshot, SpawnMode};
use crate::presets::{PRESETS, SEQUENCES};

/// Lock-free audio parameters shared between UI and audio thread.
pub struct AudioParams {
    pub base_freq: AtomicU32,
    pub beat_freq: AtomicU32,
    pub volume: AtomicU32,
    pub playing: AtomicBool,
    pub noise_level: AtomicU32,
    pub mist_type: AtomicU32,
    pub harmonics: AtomicU32,
    pub emergence: AtomicU32, // Emergence intensity 0.0-1.0
    pub spawn_mode: AtomicU32,
}

impl AudioParams {
    pub fn new() -> Self {
        Self {
            base_freq: AtomicU32::new(220.0_f32.to_bits()),
            beat_freq: AtomicU32::new(10.0_f32.to_bits()),
            volume: AtomicU32::new(0.7_f32.to_bits()),
            playing: AtomicBool::new(true),
            noise_level: AtomicU32::new(0.0_f32.to_bits()),
            mist_type: AtomicU32::new(MistType::Pink as u32),
            harmonics: AtomicU32::new(0.3_f32.to_bits()),
            emergence: AtomicU32::new(0.0_f32.to_bits()),
            spawn_mode: AtomicU32::new(SpawnMode::Canon as u32),
        }
    }

    pub fn get_base_freq(&self) -> f32 {
        f32::from_bits(self.base_freq.load(Ordering::Relaxed))
    }

    pub fn get_beat_freq(&self) -> f32 {
        f32::from_bits(self.beat_freq.load(Ordering::Relaxed))
    }

    pub fn get_volume(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }

    pub fn get_noise_level(&self) -> f32 {
        f32::from_bits(self.noise_level.load(Ordering::Relaxed))
    }

    pub fn get_mist_type(&self) -> MistType {
        MistType::from_u32(self.mist_type.load(Ordering::Relaxed))
    }

    pub fn get_harmonics(&self) -> f32 {
        f32::from_bits(self.harmonics.load(Ordering::Relaxed))
    }

    pub fn get_emergence(&self) -> f32 {
        f32::from_bits(self.emergence.load(Ordering::Relaxed))
    }

    pub fn set_base_freq(&self, v: f32) {
        self.base_freq.store(v.to_bits(), Ordering::Relaxed);
    }

    pub fn set_beat_freq(&self, v: f32) {
        self.beat_freq.store(v.to_bits(), Ordering::Relaxed);
    }

    pub fn set_volume(&self, v: f32) {
        self.volume
            .store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn set_noise_level(&self, v: f32) {
        self.noise_level
            .store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn set_mist_type(&self, v: MistType) {
        self.mist_type.store(v as u32, Ordering::Relaxed);
    }

    pub fn set_harmonics(&self, v: f32) {
        self.harmonics
            .store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn set_emergence(&self, v: f32) {
        self.emergence
            .store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn get_spawn_mode(&self) -> SpawnMode {
        SpawnMode::from_u32(self.spawn_mode.load(Ordering::Relaxed))
    }

    pub fn set_spawn_mode(&self, mode: SpawnMode) {
        self.spawn_mode.store(mode as u32, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum MistType {
    Pink = 0,
    White = 1,
    Brown = 2,
    Blue = 3,
    Velvet = 4,
}

impl MistType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::White,
            2 => Self::Brown,
            3 => Self::Blue,
            4 => Self::Velvet,
            _ => Self::Pink,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Pink => Self::White,
            Self::White => Self::Brown,
            Self::Brown => Self::Blue,
            Self::Blue => Self::Velvet,
            Self::Velvet => Self::Pink,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pink => "Pink",
            Self::White => "White",
            Self::Brown => "Brown",
            Self::Blue => "Blue",
            Self::Velvet => "Velvet",
        }
    }

    pub fn texture(self) -> &'static str {
        match self {
            Self::Pink => "warm",
            Self::White => "air",
            Self::Brown => "surf",
            Self::Blue => "glass",
            Self::Velvet => "sparks",
        }
    }
}

/// Ring buffer for passing audio samples to the visualization.
pub struct VizBuffer {
    pub samples_l: Vec<f32>,
    pub samples_r: Vec<f32>,
    pub write_pos: usize,
}

impl VizBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            samples_l: vec![0.0; size],
            samples_r: vec![0.0; size],
            write_pos: 0,
        }
    }

    pub fn push(&mut self, l: f32, r: f32) {
        let len = self.samples_l.len();
        self.samples_l[self.write_pos % len] = l;
        self.samples_r[self.write_pos % len] = r;
        self.write_pos = (self.write_pos + 1) % len;
    }

    pub fn read_ordered(&self) -> (Vec<f32>, Vec<f32>) {
        let len = self.samples_l.len();
        let pos = self.write_pos;
        let mut l = Vec::with_capacity(len);
        let mut r = Vec::with_capacity(len);
        for i in 0..len {
            let idx = (pos + i) % len;
            l.push(self.samples_l[idx]);
            r.push(self.samples_r[idx]);
        }
        (l, r)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    PresetSelect,
    SequenceSelect,
    Help,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ActiveParam {
    BaseFreq,
    BeatFreq,
    Volume,
    Harmonics,
    Emergence,
    NoiseLevel,
}

impl ActiveParam {
    pub fn next(self) -> Self {
        match self {
            Self::BaseFreq => Self::BeatFreq,
            Self::BeatFreq => Self::Volume,
            Self::Volume => Self::Harmonics,
            Self::Harmonics => Self::Emergence,
            Self::Emergence => Self::NoiseLevel,
            Self::NoiseLevel => Self::BaseFreq,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::BaseFreq => Self::NoiseLevel,
            Self::BeatFreq => Self::BaseFreq,
            Self::Volume => Self::BeatFreq,
            Self::Harmonics => Self::Volume,
            Self::Emergence => Self::Harmonics,
            Self::NoiseLevel => Self::Emergence,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::BaseFreq => "Base Freq",
            Self::BeatFreq => "Beat Freq",
            Self::Volume => "Volume",
            Self::NoiseLevel => "Noise",
            Self::Harmonics => "Warmth",
            Self::Emergence => "Emergence",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum VizMode {
    Waveform,
    Spectrum,
    Harmonics,
    Envelope,
    Penrose,
    Emergence,
}

impl VizMode {
    pub fn next(self) -> Self {
        match self {
            Self::Waveform => Self::Spectrum,
            Self::Spectrum => Self::Harmonics,
            Self::Harmonics => Self::Envelope,
            Self::Envelope => Self::Penrose,
            Self::Penrose => Self::Emergence,
            Self::Emergence => Self::Waveform,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Waveform => Self::Emergence,
            Self::Spectrum => Self::Waveform,
            Self::Harmonics => Self::Spectrum,
            Self::Envelope => Self::Harmonics,
            Self::Penrose => Self::Envelope,
            Self::Emergence => Self::Penrose,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Waveform => "Wave",
            Self::Spectrum => "Spectrum",
            Self::Harmonics => "Harmonics",
            Self::Envelope => "Envelope",
            Self::Penrose => "Penrose",
            Self::Emergence => "Emergence",
        }
    }
}

pub struct App {
    pub params: Arc<AudioParams>,
    pub viz_buffer: Arc<Mutex<VizBuffer>>,
    pub emergence_snapshot: Arc<Mutex<EmergenceSnapshot>>,
    pub mode: AppMode,
    pub active_param: ActiveParam,
    pub viz_mode: VizMode,
    pub current_preset: Option<usize>,
    pub current_sequence: Option<usize>,
    pub sequence_start: Option<Instant>,
    pub should_quit: bool,
    pub menu_index: usize,
    pub spectrum_bars: Vec<f32>,
    pub start_time: Instant,
    pub frame_count: u64,
}

impl App {
    pub fn new(
        params: Arc<AudioParams>,
        viz_buffer: Arc<Mutex<VizBuffer>>,
        emergence_snapshot: Arc<Mutex<EmergenceSnapshot>>,
    ) -> Self {
        Self {
            params,
            viz_buffer,
            emergence_snapshot,
            mode: AppMode::Normal,
            active_param: ActiveParam::BeatFreq,
            viz_mode: VizMode::Waveform,
            current_preset: Some(2), // Alpha
            current_sequence: None,
            sequence_start: None,
            should_quit: false,
            menu_index: 0,
            spectrum_bars: vec![0.0; 32],
            start_time: Instant::now(),
            frame_count: 0,
        }
    }

    pub fn apply_preset(&mut self, idx: usize) {
        if idx >= PRESETS.len() {
            return;
        }
        let preset = &PRESETS[idx];
        self.params.set_base_freq(preset.base_freq);
        self.params.set_beat_freq(preset.beat_freq);
        self.params.set_noise_level(preset.noise_mix);
        self.current_preset = Some(idx);
        self.current_sequence = None;
        self.sequence_start = None;
    }

    pub fn start_sequence(&mut self, idx: usize) {
        if idx >= SEQUENCES.len() {
            return;
        }
        self.current_sequence = Some(idx);
        self.sequence_start = Some(Instant::now());
        self.current_preset = None;
        let seq = &SEQUENCES[idx];
        if let Some(step) = seq.steps.first() {
            self.params.set_base_freq(step.base_freq);
            self.params.set_beat_freq(step.beat_freq);
        }
    }

    pub fn update_sequence(&mut self) {
        let (seq_idx, start) = match (self.current_sequence, self.sequence_start) {
            (Some(i), Some(s)) => (i, s),
            _ => return,
        };

        let seq = &SEQUENCES[seq_idx];
        let elapsed = start.elapsed().as_secs_f32();

        if elapsed >= seq.total_duration_secs {
            self.current_sequence = None;
            self.sequence_start = None;
            return;
        }

        let mut acc = 0.0_f32;
        for (i, step) in seq.steps.iter().enumerate() {
            if elapsed < acc + step.duration_secs {
                let progress = (elapsed - acc) / step.duration_secs;
                let next = if i + 1 < seq.steps.len() {
                    &seq.steps[i + 1]
                } else {
                    step
                };
                let beat = step.beat_freq + (next.beat_freq - step.beat_freq) * progress;
                let base = step.base_freq + (next.base_freq - step.base_freq) * progress;
                self.params.set_beat_freq(beat);
                self.params.set_base_freq(base);
                break;
            }
            acc += step.duration_secs;
        }
    }

    pub fn sequence_elapsed(&self) -> Option<f32> {
        self.sequence_start.map(|s| s.elapsed().as_secs_f32())
    }

    pub fn session_elapsed(&self) -> f32 {
        self.start_time.elapsed().as_secs_f32()
    }

    pub fn clear_emergence_snapshot(&self) {
        if let Ok(mut snapshot) = self.emergence_snapshot.try_lock() {
            *snapshot = EmergenceSnapshot::empty();
        }
    }

    pub fn toggle_spawn_mode(&mut self) {
        let next = self.params.get_spawn_mode().toggled();
        self.params.set_spawn_mode(next);
        if next == SpawnMode::Penrose
            && self.params.get_emergence() > 0.01
            && self.viz_mode != VizMode::Penrose
        {
            self.viz_mode = VizMode::Penrose;
        }
    }

    pub fn cycle_mist_type(&mut self) {
        let next = self.params.get_mist_type().next();
        self.params.set_mist_type(next);
        if self.params.get_noise_level() <= 0.01 {
            self.params.set_noise_level(0.15);
        }
        self.current_preset = None;
    }

    pub fn adjust_param(&mut self, delta: f32) {
        match self.active_param {
            ActiveParam::BaseFreq => {
                let v = (self.params.get_base_freq() + delta * 5.0).clamp(50.0, 500.0);
                self.params.set_base_freq(v);
            }
            ActiveParam::BeatFreq => {
                let v = (self.params.get_beat_freq() + delta * 0.5).clamp(0.5, 100.0);
                self.params.set_beat_freq(v);
            }
            ActiveParam::Volume => {
                let v = (self.params.get_volume() + delta * 0.05).clamp(0.0, 1.0);
                self.params.set_volume(v);
            }
            ActiveParam::NoiseLevel => {
                let v = (self.params.get_noise_level() + delta * 0.05).clamp(0.0, 1.0);
                self.params.set_noise_level(v);
            }
            ActiveParam::Harmonics => {
                let v = (self.params.get_harmonics() + delta * 0.05).clamp(0.0, 1.0);
                self.params.set_harmonics(v);
            }
            ActiveParam::Emergence => {
                let v = (self.params.get_emergence() + delta * 0.05).clamp(0.0, 1.0);
                self.params.set_emergence(v);
                if v <= 0.01 {
                    self.clear_emergence_snapshot();
                }
            }
        }
        self.current_preset = None;
    }
}
