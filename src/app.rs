use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::{io, path::PathBuf};

use crate::emergence::{EmergenceSnapshot, SpawnMode};
use crate::knowledge::KnowledgeState;
use crate::local_presets::{self, LocalPreset};
use crate::presets::{PRESETS, SEQUENCES, SequenceStep};
use crate::shepard::{DEFAULT_BASE_FREQ_HZ, Direction, MAX_BASE_FREQ_HZ, MIN_BASE_FREQ_HZ};

/// Top-level UI tab. The Studio tab is the live synth; Knowledge is the
/// in-app wiki / glossary / playground. The two are orthogonal axes; the
/// Studio modal-mode (`AppMode`) is dispatched only when `tab == Studio`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Studio,
    Knowledge,
}

impl Tab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Studio => "Studio",
            Self::Knowledge => "Knowledge",
        }
    }

    pub fn flipped(self) -> Self {
        match self {
            Self::Studio => Self::Knowledge,
            Self::Knowledge => Self::Studio,
        }
    }
}

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
    pub shepard: AtomicU32, // Shepard intensity 0.0-1.0
    pub shepard_base_freq: AtomicU32,
    pub shepard_direction: AtomicU32,
    pub timbre: AtomicU32,
}

impl AudioParams {
    pub fn new() -> Self {
        Self {
            base_freq: AtomicU32::new(220.0_f32.to_bits()),
            beat_freq: AtomicU32::new(10.0_f32.to_bits()),
            volume: AtomicU32::new(0.5_f32.to_bits()),
            playing: AtomicBool::new(true),
            noise_level: AtomicU32::new(0.0_f32.to_bits()),
            mist_type: AtomicU32::new(MistType::Brown as u32),
            harmonics: AtomicU32::new(0.3_f32.to_bits()),
            emergence: AtomicU32::new(0.0_f32.to_bits()),
            spawn_mode: AtomicU32::new(SpawnMode::Canon as u32),
            shepard: AtomicU32::new(0.0_f32.to_bits()),
            shepard_base_freq: AtomicU32::new((DEFAULT_BASE_FREQ_HZ as f32).to_bits()),
            shepard_direction: AtomicU32::new(Direction::Down as u32),
            timbre: AtomicU32::new(Timbre::Organ as u32),
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

    pub fn get_shepard(&self) -> f32 {
        f32::from_bits(self.shepard.load(Ordering::Relaxed))
    }

    pub fn set_shepard(&self, v: f32) {
        self.shepard
            .store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn get_shepard_base_freq(&self) -> f32 {
        f32::from_bits(self.shepard_base_freq.load(Ordering::Relaxed))
    }

    pub fn set_shepard_base_freq(&self, v: f32) {
        self.shepard_base_freq.store(
            v.clamp(MIN_BASE_FREQ_HZ as f32, MAX_BASE_FREQ_HZ as f32)
                .to_bits(),
            Ordering::Relaxed,
        );
    }

    pub fn get_shepard_direction(&self) -> Direction {
        Direction::from_u32(self.shepard_direction.load(Ordering::Relaxed))
    }

    pub fn set_shepard_direction(&self, d: Direction) {
        self.shepard_direction.store(d as u32, Ordering::Relaxed);
    }

    pub fn get_timbre(&self) -> Timbre {
        Timbre::from_u32(self.timbre.load(Ordering::Relaxed))
    }

    pub fn set_timbre(&self, t: Timbre) {
        self.timbre.store(t as u32, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Timbre {
    Organ = 0,
    Flute = 1,
    Bell = 2,
    Saw = 3,
}

impl Timbre {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Flute,
            2 => Self::Bell,
            3 => Self::Saw,
            _ => Self::Organ,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Organ => Self::Flute,
            Self::Flute => Self::Bell,
            Self::Bell => Self::Saw,
            Self::Saw => Self::Organ,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Organ => "Organ",
            Self::Flute => "Flute",
            Self::Bell => "Bell",
            Self::Saw => "Saw",
        }
    }

    pub fn weights(self) -> [f64; 5] {
        match self {
            Self::Organ => [0.5, 0.25, 0.125, 0.0625, 0.03125],
            Self::Flute => [0.0, 0.5, 0.0, 0.125, 0.0],
            Self::Bell => [1.0, 0.0, 0.5, 0.0, 0.25],
            Self::Saw => [0.5, 0.333, 0.25, 0.2, 0.166],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    PresetName,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresetSelection {
    BuiltIn(usize),
    Local(usize),
}

#[derive(Clone, Copy, PartialEq)]
pub enum ActiveParam {
    BaseFreq,
    BeatFreq,
    Volume,
    Harmonics,
    Emergence,
    Shepard,
    ShepardBase,
    NoiseLevel,
}

impl ActiveParam {
    pub fn next(self) -> Self {
        match self {
            Self::BaseFreq => Self::BeatFreq,
            Self::BeatFreq => Self::Volume,
            Self::Volume => Self::Harmonics,
            Self::Harmonics => Self::Emergence,
            Self::Emergence => Self::Shepard,
            Self::Shepard => Self::ShepardBase,
            Self::ShepardBase => Self::NoiseLevel,
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
            Self::Shepard => Self::Emergence,
            Self::ShepardBase => Self::Shepard,
            Self::NoiseLevel => Self::ShepardBase,
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
            Self::Shepard => "Shepard",
            Self::ShepardBase => "Drift Base",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub tab: Tab,
    pub knowledge: KnowledgeState,
    pub mode: AppMode,
    pub active_param: ActiveParam,
    pub viz_mode: VizMode,
    pub current_preset: Option<PresetSelection>,
    pub local_presets: Vec<LocalPreset>,
    pub current_sequence: Option<usize>,
    pub sequence_start: Option<Instant>,
    pub should_quit: bool,
    pub menu_index: usize,
    pub preset_name_input: String,
    pub preset_storage_path: Option<PathBuf>,
    pub status_message: Option<String>,
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
        let preset_storage_path = local_presets::default_path();
        let loaded_local_presets = preset_storage_path
            .as_deref()
            .map(local_presets::load)
            .unwrap_or_default();
        Self::with_local_presets(
            params,
            viz_buffer,
            emergence_snapshot,
            loaded_local_presets,
            preset_storage_path,
        )
    }

    fn with_local_presets(
        params: Arc<AudioParams>,
        viz_buffer: Arc<Mutex<VizBuffer>>,
        emergence_snapshot: Arc<Mutex<EmergenceSnapshot>>,
        local_presets: Vec<LocalPreset>,
        preset_storage_path: Option<PathBuf>,
    ) -> Self {
        Self {
            params,
            viz_buffer,
            emergence_snapshot,
            tab: Tab::Studio,
            knowledge: KnowledgeState::new(),
            mode: AppMode::Normal,
            active_param: ActiveParam::BeatFreq,
            viz_mode: VizMode::Waveform,
            current_preset: Some(PresetSelection::BuiltIn(2)), // Alpha
            local_presets,
            current_sequence: None,
            sequence_start: None,
            should_quit: false,
            menu_index: 0,
            preset_name_input: String::new(),
            preset_storage_path,
            status_message: None,
            spectrum_bars: vec![0.0; 32],
            start_time: Instant::now(),
            frame_count: 0,
        }
    }

    pub fn total_preset_count(&self) -> usize {
        PRESETS.len() + self.local_presets.len()
    }

    pub fn preset_menu_index(&self) -> usize {
        let selected = match self.current_preset {
            Some(PresetSelection::BuiltIn(idx)) => idx,
            Some(PresetSelection::Local(idx)) => PRESETS.len() + idx,
            None => 0,
        };
        selected.min(self.total_preset_count().saturating_sub(1))
    }

    pub fn current_preset_name(&self) -> Option<&str> {
        match self.current_preset {
            Some(PresetSelection::BuiltIn(idx)) => PRESETS.get(idx).map(|preset| preset.name),
            Some(PresetSelection::Local(idx)) => self
                .local_presets
                .get(idx)
                .map(|preset| preset.name.as_str()),
            None => None,
        }
    }

    pub fn begin_preset_save(&mut self) {
        self.preset_name_input = self.default_local_preset_name();
        self.mode = AppMode::PresetName;
    }

    pub fn finish_preset_save(&mut self) {
        let name = self.preset_name_input.clone();
        self.save_local_preset(&name);
        self.preset_name_input.clear();
        self.mode = AppMode::Normal;
    }

    pub fn cancel_preset_save(&mut self) {
        self.preset_name_input.clear();
        self.mode = AppMode::Normal;
    }

    pub fn save_local_preset(&mut self, name: &str) {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            self.status_message = Some("Preset name is empty".to_string());
            return;
        }

        let preset = self.capture_local_preset(trimmed.to_string());
        let previous = self.local_presets.clone();
        let index = if let Some(idx) = self
            .local_presets
            .iter()
            .position(|existing| existing.name == trimmed)
        {
            self.local_presets[idx] = preset;
            idx
        } else {
            self.local_presets.push(preset);
            self.local_presets.len() - 1
        };

        match self.persist_local_presets() {
            Ok(()) => {
                if self.current_sequence.is_none() {
                    self.current_preset = Some(PresetSelection::Local(index));
                }
                self.status_message = Some(format!("Saved preset {trimmed}"));
            }
            Err(err) => {
                self.local_presets = previous;
                self.status_message = Some(format!("Preset save failed: {err}"));
            }
        }
    }

    pub fn apply_preset_menu_index(&mut self, idx: usize) {
        if idx < PRESETS.len() {
            self.apply_preset(idx);
        } else {
            self.apply_local_preset(idx - PRESETS.len());
        }
    }

    pub fn apply_local_preset(&mut self, idx: usize) {
        let Some(preset) = self.local_presets.get(idx).cloned() else {
            return;
        };

        self.params
            .set_base_freq(preset.base_freq.clamp(50.0, 500.0));
        self.params
            .set_beat_freq(preset.beat_freq.clamp(0.5, 100.0));
        self.params.set_volume(preset.volume);
        self.params.set_noise_level(preset.noise_level);
        self.params.set_mist_type(preset.mist_type);
        self.params.set_harmonics(preset.harmonics);
        self.params.set_emergence(preset.emergence);
        self.params.set_spawn_mode(preset.spawn_mode);
        self.params.set_shepard(preset.shepard);
        self.params.set_shepard_base_freq(preset.shepard_base_freq);
        self.params.set_shepard_direction(preset.shepard_direction);
        self.params.set_timbre(preset.timbre);
        self.viz_mode = preset.viz_mode;
        if preset.emergence <= 0.01 {
            self.clear_emergence_snapshot();
        }
        self.current_preset = Some(PresetSelection::Local(idx));
        self.current_sequence = None;
        self.sequence_start = None;
    }

    pub fn delete_preset_menu_index(&mut self, idx: usize) -> bool {
        let Some(local_idx) = self.local_index_for_menu(idx) else {
            return false;
        };

        let removed_name = self.local_presets[local_idx].name.clone();
        let previous = self.local_presets.clone();
        self.local_presets.remove(local_idx);

        match self.persist_local_presets() {
            Ok(()) => {
                self.current_preset = match self.current_preset {
                    Some(PresetSelection::Local(current)) if current == local_idx => None,
                    Some(PresetSelection::Local(current)) if current > local_idx => {
                        Some(PresetSelection::Local(current - 1))
                    }
                    other => other,
                };
                self.menu_index = self
                    .menu_index
                    .min(self.total_preset_count().saturating_sub(1));
                self.status_message = Some(format!("Deleted preset {removed_name}"));
                true
            }
            Err(err) => {
                self.local_presets = previous;
                self.status_message = Some(format!("Preset delete failed: {err}"));
                false
            }
        }
    }

    fn local_index_for_menu(&self, idx: usize) -> Option<usize> {
        idx.checked_sub(PRESETS.len())
            .filter(|local_idx| *local_idx < self.local_presets.len())
    }

    fn persist_local_presets(&self) -> io::Result<()> {
        let path = self.preset_storage_path.as_deref().ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, "preset storage path unavailable")
        })?;
        local_presets::save(path, &self.local_presets)
    }

    fn capture_local_preset(&self, name: String) -> LocalPreset {
        LocalPreset {
            name,
            base_freq: self.params.get_base_freq(),
            beat_freq: self.params.get_beat_freq(),
            volume: self.params.get_volume(),
            noise_level: self.params.get_noise_level(),
            mist_type: self.params.get_mist_type(),
            harmonics: self.params.get_harmonics(),
            emergence: self.params.get_emergence(),
            spawn_mode: self.params.get_spawn_mode(),
            shepard: self.params.get_shepard(),
            shepard_base_freq: self.params.get_shepard_base_freq(),
            shepard_direction: self.params.get_shepard_direction(),
            timbre: self.params.get_timbre(),
            viz_mode: self.viz_mode,
        }
    }

    fn default_local_preset_name(&self) -> String {
        if let Some(PresetSelection::Local(idx)) = self.current_preset
            && let Some(preset) = self.local_presets.get(idx)
        {
            return preset.name.clone();
        }

        let base = match self.current_preset {
            Some(PresetSelection::BuiltIn(idx)) => PRESETS
                .get(idx)
                .map(|preset| format!("{} Custom", preset.name))
                .unwrap_or_else(|| "Custom".to_string()),
            _ => "Custom".to_string(),
        };
        self.unique_local_preset_name(&base)
    }

    fn unique_local_preset_name(&self, base: &str) -> String {
        if !self.local_presets.iter().any(|preset| preset.name == base) {
            return base.to_string();
        }

        let mut suffix = 2;
        loop {
            let candidate = format!("{base} {suffix}");
            if !self
                .local_presets
                .iter()
                .any(|preset| preset.name == candidate)
            {
                return candidate;
            }
            suffix += 1;
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
        if preset.noise_mix > 0.01 {
            self.params.set_mist_type(MistType::Brown);
        }
        self.params.set_emergence(0.0);
        self.params.set_shepard(0.0);
        self.clear_emergence_snapshot();
        self.current_preset = Some(PresetSelection::BuiltIn(idx));
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
        self.reset_unprogrammed_sequence_layers(seq.steps);
        if let Some(step) = seq.steps.first() {
            self.snap_to_step(step);
        }
    }

    fn reset_unprogrammed_sequence_layers(&mut self, steps: &[SequenceStep]) {
        if !steps
            .iter()
            .any(|step| step.noise_level.is_some() || step.mist_type.is_some())
        {
            self.params.set_noise_level(0.0);
            self.params.set_mist_type(MistType::Brown);
        }
        if !steps.iter().any(|step| step.emergence.is_some()) {
            self.params.set_emergence(0.0);
            self.clear_emergence_snapshot();
        }
        if !steps.iter().any(|step| step.shepard.is_some()) {
            self.params.set_shepard(0.0);
        }
    }

    /// Snap every parameter that the step automates to its starting value.
    /// Used at sequence entry so the listener never sees a one-frame flicker
    /// from prior state before `update_sequence` takes over.
    fn snap_to_step(&mut self, step: &SequenceStep) {
        self.params.set_base_freq(step.base_freq);
        self.params.set_beat_freq(step.beat_freq);
        if let Some(v) = step.volume {
            self.params.set_volume(v);
        }
        if let Some(v) = step.noise_level {
            self.params.set_noise_level(v);
        }
        if let Some(v) = step.harmonics {
            self.params.set_harmonics(v);
        }
        if let Some(v) = step.emergence {
            self.params.set_emergence(v);
        }
        if let Some(v) = step.shepard {
            self.params.set_shepard(v);
        }
        if let Some(t) = step.timbre {
            self.params.set_timbre(t);
        }
        if let Some(m) = step.mist_type {
            self.params.set_mist_type(m);
        }
        if let Some(d) = step.shepard_direction {
            self.params.set_shepard_direction(d);
        }
        if let Some(sm) = step.spawn_mode {
            self.params.set_spawn_mode(sm);
        }
        if let Some(vm) = step.viz_mode {
            self.viz_mode = vm;
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

                // Always-automated continuous params.
                let beat = step.beat_freq + (next.beat_freq - step.beat_freq) * progress;
                let base = step.base_freq + (next.base_freq - step.base_freq) * progress;
                self.params.set_beat_freq(beat);
                self.params.set_base_freq(base);

                // Optionally-automated continuous params: lerp toward the
                // next step's value, falling back to the current step when
                // the next step leaves the field unset.
                if let Some(v) = step.volume {
                    let target = next.volume.unwrap_or(v);
                    self.params.set_volume(v + (target - v) * progress);
                }
                if let Some(v) = step.noise_level {
                    let target = next.noise_level.unwrap_or(v);
                    self.params.set_noise_level(v + (target - v) * progress);
                }
                if let Some(v) = step.harmonics {
                    let target = next.harmonics.unwrap_or(v);
                    self.params.set_harmonics(v + (target - v) * progress);
                }
                if let Some(v) = step.emergence {
                    let target = next.emergence.unwrap_or(v);
                    self.params.set_emergence(v + (target - v) * progress);
                }
                if let Some(v) = step.shepard {
                    let target = next.shepard.unwrap_or(v);
                    self.params.set_shepard(v + (target - v) * progress);
                }

                // Discrete params: snap to the current step's value every
                // tick. Atomic stores are idempotent and the audio thread's
                // 50 ms exponential smoothing handles the boundary gracefully.
                if let Some(t) = step.timbre {
                    self.params.set_timbre(t);
                }
                if let Some(m) = step.mist_type {
                    self.params.set_mist_type(m);
                }
                if let Some(d) = step.shepard_direction {
                    self.params.set_shepard_direction(d);
                }
                if let Some(sm) = step.spawn_mode {
                    self.params.set_spawn_mode(sm);
                }
                if let Some(vm) = step.viz_mode {
                    self.viz_mode = vm;
                }
                break;
            }
            acc += step.duration_secs;
        }
    }

    /// Name of the currently-active sequence step, if it has one.
    pub fn current_step_name(&self) -> Option<&'static str> {
        let (seq_idx, start) = match (self.current_sequence, self.sequence_start) {
            (Some(i), Some(s)) => (i, s),
            _ => return None,
        };
        let seq = &SEQUENCES[seq_idx];
        let elapsed = start.elapsed().as_secs_f32();
        if elapsed >= seq.total_duration_secs {
            return None;
        }
        let mut acc = 0.0_f32;
        for step in seq.steps {
            if elapsed < acc + step.duration_secs {
                return step.name;
            }
            acc += step.duration_secs;
        }
        None
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
        self.current_preset = None;
    }

    pub fn toggle_emergence(&mut self) {
        let current = self.params.get_emergence();
        if current > 0.01 {
            self.params.set_emergence(0.0);
            self.clear_emergence_snapshot();
        } else {
            self.params.set_emergence(0.5);
        }
        self.current_preset = None;
    }

    pub fn toggle_noise(&mut self) {
        let current = self.params.get_noise_level();
        if current > 0.01 {
            self.params.set_noise_level(0.0);
        } else {
            self.params.set_noise_level(0.15);
        }
        self.current_preset = None;
    }

    pub fn next_viz_mode(&mut self) {
        self.viz_mode = self.viz_mode.next();
        self.current_preset = None;
    }

    pub fn prev_viz_mode(&mut self) {
        self.viz_mode = self.viz_mode.prev();
        self.current_preset = None;
    }

    pub fn cycle_mist_type(&mut self) {
        let next = self.params.get_mist_type().next();
        self.params.set_mist_type(next);
        if self.params.get_noise_level() <= 0.01 {
            self.params.set_noise_level(0.15);
        }
        self.current_preset = None;
    }

    pub fn cycle_timbre(&mut self) {
        let next = self.params.get_timbre().next();
        self.params.set_timbre(next);
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
            ActiveParam::Shepard => {
                let v = (self.params.get_shepard() + delta * 0.05).clamp(0.0, 1.0);
                self.params.set_shepard(v);
            }
            ActiveParam::ShepardBase => {
                let v = self.params.get_shepard_base_freq() + delta;
                self.params.set_shepard_base_freq(v);
            }
        }
        self.current_preset = None;
    }

    pub fn toggle_shepard(&mut self) {
        let current = self.params.get_shepard();
        if current > 0.01 {
            self.params.set_shepard(0.0);
        } else {
            self.params.set_shepard(0.35);
        }
        self.current_preset = None;
    }

    pub fn reverse_shepard(&mut self) {
        let next = self.params.get_shepard_direction().flipped();
        self.params.set_shepard_direction(next);
        self.current_preset = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn app_for_tests() -> App {
        app_for_tests_with_storage(None)
    }

    fn app_for_tests_with_storage(preset_storage_path: Option<PathBuf>) -> App {
        let params = Arc::new(AudioParams::new());
        let viz_buffer = Arc::new(Mutex::new(VizBuffer::new(64)));
        let emergence_snapshot = Arc::new(Mutex::new(EmergenceSnapshot::empty()));
        App::with_local_presets(
            params,
            viz_buffer,
            emergence_snapshot,
            Vec::new(),
            preset_storage_path,
        )
    }

    fn sample_local_preset(name: &str) -> LocalPreset {
        LocalPreset {
            name: name.to_string(),
            base_freq: 245.0,
            beat_freq: 18.0,
            volume: 0.65,
            noise_level: 0.25,
            mist_type: MistType::Velvet,
            harmonics: 0.7,
            emergence: 0.45,
            spawn_mode: SpawnMode::Penrose,
            shepard: 0.35,
            shepard_base_freq: DEFAULT_BASE_FREQ_HZ as f32 * 2.0,
            shepard_direction: Direction::Up,
            timbre: Timbre::Bell,
            viz_mode: VizMode::Emergence,
        }
    }

    fn temp_preset_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        env::temp_dir()
            .join(format!("microtube-app-{name}-{nanos}"))
            .join("presets.json")
    }

    #[test]
    fn defaults_use_half_gain_and_brown_mist() {
        let params = AudioParams::new();

        assert_eq!(params.get_volume(), 0.5);
        assert_eq!(params.get_mist_type(), MistType::Brown);
        assert_eq!(params.get_shepard_base_freq(), DEFAULT_BASE_FREQ_HZ as f32);
    }

    #[test]
    fn quick_preset_clears_sequence_only_layers() {
        let mut app = app_for_tests();
        app.params.set_noise_level(0.85);
        app.params.set_mist_type(MistType::Velvet);
        app.params.set_emergence(0.7);
        app.params.set_shepard(0.6);

        app.apply_preset(2);

        assert_eq!(app.params.get_noise_level(), 0.0);
        assert_eq!(app.params.get_emergence(), 0.0);
        assert_eq!(app.params.get_shepard(), 0.0);
    }

    #[test]
    fn legacy_sequence_start_clears_unprogrammed_layers() {
        let mut app = app_for_tests();
        app.params.set_noise_level(0.85);
        app.params.set_mist_type(MistType::Blue);
        app.params.set_emergence(0.7);
        app.params.set_shepard(0.6);

        app.start_sequence(0);

        assert_eq!(app.params.get_noise_level(), 0.0);
        assert_eq!(app.params.get_mist_type(), MistType::Brown);
        assert_eq!(app.params.get_emergence(), 0.0);
        assert_eq!(app.params.get_shepard(), 0.0);
    }

    #[test]
    fn toggling_emergence_does_not_change_visualization() {
        let mut app = app_for_tests();
        app.viz_mode = VizMode::Spectrum;

        app.toggle_emergence();

        assert_eq!(app.params.get_emergence(), 0.5);
        assert_eq!(app.viz_mode, VizMode::Spectrum);
    }

    #[test]
    fn toggling_spawn_mode_does_not_change_visualization() {
        let mut app = app_for_tests();
        app.viz_mode = VizMode::Spectrum;
        app.params.set_emergence(0.5);

        app.toggle_spawn_mode();

        assert_eq!(app.params.get_spawn_mode(), SpawnMode::Penrose);
        assert_eq!(app.viz_mode, VizMode::Spectrum);
    }

    #[test]
    fn shepard_base_frequency_is_adjustable_and_clamped() {
        let mut app = app_for_tests();
        app.active_param = ActiveParam::ShepardBase;

        app.adjust_param(10.0);

        assert_eq!(
            app.params.get_shepard_base_freq(),
            DEFAULT_BASE_FREQ_HZ as f32 + 10.0
        );

        app.params.set_shepard_base_freq(1.0);
        assert_eq!(app.params.get_shepard_base_freq(), MIN_BASE_FREQ_HZ as f32);

        app.params.set_shepard_base_freq(1_000.0);
        assert_eq!(app.params.get_shepard_base_freq(), MAX_BASE_FREQ_HZ as f32);
    }

    #[test]
    fn saving_local_preset_captures_live_fields() {
        let path = temp_preset_path("save");
        let mut app = app_for_tests_with_storage(Some(path.clone()));
        app.params.set_base_freq(245.0);
        app.params.set_beat_freq(18.0);
        app.params.set_volume(0.65);
        app.params.set_noise_level(0.25);
        app.params.set_mist_type(MistType::Velvet);
        app.params.set_harmonics(0.7);
        app.params.set_emergence(0.45);
        app.params.set_spawn_mode(SpawnMode::Penrose);
        app.params.set_shepard(0.35);
        app.params
            .set_shepard_base_freq(DEFAULT_BASE_FREQ_HZ as f32 * 2.0);
        app.params.set_shepard_direction(Direction::Up);
        app.params.set_timbre(Timbre::Bell);
        app.viz_mode = VizMode::Emergence;

        app.save_local_preset("  Glass Focus  ");

        assert_eq!(app.local_presets, vec![sample_local_preset("Glass Focus")]);
        assert_eq!(
            local_presets::try_load(&path).expect("saved preset file"),
            vec![sample_local_preset("Glass Focus")]
        );
        let _ = path.parent().map(fs::remove_dir_all);
    }

    #[test]
    fn local_preset_recall_restores_snapshot_and_clears_sequence() {
        let mut app = app_for_tests();
        app.local_presets.push(sample_local_preset("Glass Focus"));
        app.start_sequence(0);

        app.apply_local_preset(0);

        assert_eq!(app.current_sequence, None);
        assert_eq!(app.sequence_start, None);
        assert_eq!(app.current_preset, Some(PresetSelection::Local(0)));
        assert_eq!(app.params.get_base_freq(), 245.0);
        assert_eq!(app.params.get_beat_freq(), 18.0);
        assert_eq!(app.params.get_volume(), 0.65);
        assert_eq!(app.params.get_noise_level(), 0.25);
        assert_eq!(app.params.get_mist_type(), MistType::Velvet);
        assert_eq!(app.params.get_harmonics(), 0.7);
        assert_eq!(app.params.get_emergence(), 0.45);
        assert_eq!(app.params.get_spawn_mode(), SpawnMode::Penrose);
        assert_eq!(app.params.get_shepard(), 0.35);
        assert_eq!(
            app.params.get_shepard_base_freq(),
            DEFAULT_BASE_FREQ_HZ as f32 * 2.0
        );
        assert_eq!(app.params.get_shepard_direction(), Direction::Up);
        assert_eq!(app.params.get_timbre(), Timbre::Bell);
        assert_eq!(app.viz_mode, VizMode::Emergence);
    }

    #[test]
    fn deleting_presets_only_removes_local_entries() {
        let path = temp_preset_path("delete");
        let mut app = app_for_tests_with_storage(Some(path.clone()));
        app.local_presets.push(sample_local_preset("One"));
        app.local_presets.push(sample_local_preset("Two"));
        app.current_preset = Some(PresetSelection::Local(1));

        assert!(!app.delete_preset_menu_index(0));
        assert_eq!(app.local_presets.len(), 2);

        assert!(app.delete_preset_menu_index(PRESETS.len()));
        assert_eq!(app.local_presets.len(), 1);
        assert_eq!(app.local_presets[0].name, "Two");
        assert_eq!(app.current_preset, Some(PresetSelection::Local(0)));
        let _ = path.parent().map(fs::remove_dir_all);
    }
}
