use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::app::{MistType, Timbre, VizMode};
use crate::emergence::SpawnMode;
use crate::shepard::Direction;

const FILE_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq)]
pub struct LocalPreset {
    pub name: String,
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
    pub viz_mode: VizMode,
}

impl LocalPreset {
    pub fn short_description(&self) -> String {
        format!(
            "{:.1} Hz beat / {:.1} Hz base / {} / {}",
            self.beat_freq,
            self.base_freq,
            self.timbre.label(),
            self.viz_mode.label()
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PresetFile {
    version: u32,
    presets: Vec<StoredPreset>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredPreset {
    name: String,
    base_freq: f32,
    beat_freq: f32,
    volume: f32,
    noise_level: f32,
    mist_type: String,
    harmonics: f32,
    emergence: f32,
    #[serde(default = "default_gravity")]
    gravity: f32,
    spawn_mode: String,
    shepard: f32,
    shepard_base_freq: f32,
    shepard_direction: String,
    timbre: String,
    viz_mode: String,
}

impl From<&LocalPreset> for StoredPreset {
    fn from(preset: &LocalPreset) -> Self {
        Self {
            name: preset.name.clone(),
            base_freq: preset.base_freq,
            beat_freq: preset.beat_freq,
            volume: preset.volume,
            noise_level: preset.noise_level,
            mist_type: preset.mist_type.label().to_string(),
            harmonics: preset.harmonics,
            emergence: preset.emergence,
            gravity: preset.gravity,
            spawn_mode: preset.spawn_mode.label().to_string(),
            shepard: preset.shepard,
            shepard_base_freq: preset.shepard_base_freq,
            shepard_direction: preset.shepard_direction.label().to_string(),
            timbre: preset.timbre.label().to_string(),
            viz_mode: preset.viz_mode.label().to_string(),
        }
    }
}

impl StoredPreset {
    fn into_local(self) -> io::Result<LocalPreset> {
        let preset = LocalPreset {
            name: normalized_name(&self.name)?,
            base_freq: finite("base_freq", self.base_freq)?,
            beat_freq: finite("beat_freq", self.beat_freq)?,
            volume: finite("volume", self.volume)?,
            noise_level: finite("noise_level", self.noise_level)?,
            mist_type: parse_mist_type(&self.mist_type)?,
            harmonics: finite("harmonics", self.harmonics)?,
            emergence: finite("emergence", self.emergence)?,
            gravity: finite("gravity", self.gravity)?,
            spawn_mode: parse_spawn_mode(&self.spawn_mode)?,
            shepard: finite("shepard", self.shepard)?,
            shepard_base_freq: finite("shepard_base_freq", self.shepard_base_freq)?,
            shepard_direction: parse_direction(&self.shepard_direction)?,
            timbre: parse_timbre(&self.timbre)?,
            viz_mode: parse_viz_mode(&self.viz_mode)?,
        };
        Ok(preset)
    }
}

pub fn default_path() -> Option<PathBuf> {
    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME")
        && !config_home.as_os_str().is_empty()
    {
        return Some(
            PathBuf::from(config_home)
                .join("microtube")
                .join("presets.json"),
        );
    }

    env::var_os("HOME")
        .filter(|home| !home.as_os_str().is_empty())
        .map(|home| {
            PathBuf::from(home)
                .join(".config")
                .join("microtube")
                .join("presets.json")
        })
}

pub fn load(path: &Path) -> Vec<LocalPreset> {
    try_load(path).unwrap_or_default()
}

pub fn try_load(path: &Path) -> io::Result<Vec<LocalPreset>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)?;
    let file: PresetFile = serde_json::from_str(&contents).map_err(invalid_data)?;
    if file.version != FILE_VERSION {
        return Err(invalid_data(format!(
            "unsupported preset file version {}",
            file.version
        )));
    }

    file.presets
        .into_iter()
        .map(StoredPreset::into_local)
        .collect()
}

pub fn save(path: &Path, presets: &[LocalPreset]) -> io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let file = PresetFile {
        version: FILE_VERSION,
        presets: presets.iter().map(StoredPreset::from).collect(),
    };
    let json = serde_json::to_string_pretty(&file).map_err(invalid_data)?;
    let tmp_path = temp_path(path);

    fs::write(&tmp_path, json)?;
    if let Err(err) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        return Err(err);
    }
    Ok(())
}

fn temp_path(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!("{ext}.tmp"))
        .unwrap_or_else(|| "tmp".to_string());
    tmp.set_extension(extension);
    tmp
}

fn normalized_name(name: &str) -> io::Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(invalid_data("preset name is empty"));
    }
    Ok(trimmed.to_string())
}

fn finite(field: &'static str, value: f32) -> io::Result<f32> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(invalid_data(format!("{field} is not finite")))
    }
}

fn default_gravity() -> f32 {
    0.5
}

fn parse_mist_type(value: &str) -> io::Result<MistType> {
    match value {
        "Pink" | "pink" => Ok(MistType::Pink),
        "White" | "white" => Ok(MistType::White),
        "Brown" | "brown" => Ok(MistType::Brown),
        "Blue" | "blue" => Ok(MistType::Blue),
        "Velvet" | "velvet" => Ok(MistType::Velvet),
        _ => Err(invalid_data(format!("unknown mist type {value:?}"))),
    }
}

fn parse_timbre(value: &str) -> io::Result<Timbre> {
    match value {
        "Organ" | "organ" => Ok(Timbre::Organ),
        "Flute" | "flute" => Ok(Timbre::Flute),
        "Bell" | "bell" => Ok(Timbre::Bell),
        "Saw" | "saw" => Ok(Timbre::Saw),
        _ => Err(invalid_data(format!("unknown timbre {value:?}"))),
    }
}

fn parse_viz_mode(value: &str) -> io::Result<VizMode> {
    match value {
        "Wave" | "wave" | "Waveform" | "waveform" => Ok(VizMode::Waveform),
        "Spectrum" | "spectrum" => Ok(VizMode::Spectrum),
        "Harmonics" | "harmonics" => Ok(VizMode::Harmonics),
        "Envelope" | "envelope" => Ok(VizMode::Envelope),
        "Penrose" | "penrose" => Ok(VizMode::Penrose),
        "Emergence" | "emergence" => Ok(VizMode::Emergence),
        _ => Err(invalid_data(format!(
            "unknown visualization mode {value:?}"
        ))),
    }
}

fn parse_spawn_mode(value: &str) -> io::Result<SpawnMode> {
    match value {
        "canon" | "Canon" => Ok(SpawnMode::Canon),
        "penrose" | "Penrose" => Ok(SpawnMode::Penrose),
        "fuxian" | "Fuxian" => Ok(SpawnMode::Fuxian),
        _ => Err(invalid_data(format!("unknown spawn mode {value:?}"))),
    }
}

fn parse_direction(value: &str) -> io::Result<Direction> {
    match value {
        "rising" | "Rising" | "up" | "Up" => Ok(Direction::Up),
        "falling" | "Falling" | "down" | "Down" => Ok(Direction::Down),
        _ => Err(invalid_data(format!("unknown Shepard direction {value:?}"))),
    }
}

fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shepard::DEFAULT_BASE_FREQ_HZ;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_preset() -> LocalPreset {
        LocalPreset {
            name: "Glass Focus".to_string(),
            base_freq: 240.0,
            beat_freq: 18.0,
            volume: 0.6,
            noise_level: 0.25,
            mist_type: MistType::Velvet,
            harmonics: 0.7,
            emergence: 0.45,
            gravity: 0.6,
            spawn_mode: SpawnMode::Penrose,
            shepard: 0.3,
            shepard_base_freq: DEFAULT_BASE_FREQ_HZ as f32,
            shepard_direction: Direction::Up,
            timbre: Timbre::Bell,
            viz_mode: VizMode::Emergence,
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("microtube-{name}-{nanos}"))
    }

    #[test]
    fn json_round_trip_uses_stable_labels() {
        let preset = sample_preset();
        let stored = StoredPreset::from(&preset);
        let json = serde_json::to_string(&stored).expect("serialize preset");

        assert!(json.contains("\"mist_type\":\"Velvet\""));
        assert!(json.contains("\"gravity\":0.6"));
        assert!(json.contains("\"spawn_mode\":\"penrose\""));
        assert!(json.contains("\"shepard_direction\":\"rising\""));
        assert!(json.contains("\"timbre\":\"Bell\""));
        assert!(json.contains("\"viz_mode\":\"Emergence\""));
        assert_eq!(stored.into_local().expect("parse stored preset"), preset);
    }

    #[test]
    fn missing_and_malformed_files_load_empty() {
        let dir = temp_dir("malformed");
        let path = dir.join("presets.json");

        assert!(load(&path).is_empty());
        fs::create_dir_all(&dir).expect("create temp dir");
        fs::write(&path, "{not json").expect("write malformed file");

        assert!(load(&path).is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn saves_and_loads_presets() {
        let dir = temp_dir("save-load");
        let path = dir.join("nested").join("presets.json");
        let preset = sample_preset();

        save(&path, std::slice::from_ref(&preset)).expect("save presets");

        assert_eq!(try_load(&path).expect("load presets"), vec![preset]);
        let _ = fs::remove_dir_all(&dir);
    }
}
