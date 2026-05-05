# MicroTube

A terminal-native binaural beats engine with generative audio, built in Rust.

MicroTube synthesizes binaural beats in real-time, streams stereo audio to PipeWire, and renders animated visualizations using Unicode braille characters — all within your terminal emulator.

## Features

**Audio Synthesis**
- Phase-accumulator binaural beat generation (drift-free, runs indefinitely)
- Harmonic overtones (2nd/3rd/4th partials) for warm, organ-like timbre
- Selectable mist algorithms: pink, white, brown, blue, and velvet textures
- Exponential parameter smoothing (~50ms) eliminates clicks on transitions
- Soft-clip limiter prevents digital distortion

**Emergence Engine**
- Generative audio system inspired by Conway's Game of Life and Bach canons
- Up to 12 simultaneous voices spawning at harmonic/golden-ratio intervals
- Two spawn modes: a fugue-style canon, or a Penrose Conway-worm walk
- Consonance-based lifetime: harmonically pure voices live longer
- Energy conservation: total amplitude is bounded, voices compete
- Random mutations introduce organic variation

**Visualization**
- Braille-character waveforms (2x4 sub-cell resolution per character)
- Cava-style spectrum bars with gravity falloff
- Harmonic phase portrait with just-intonation and golden-ratio lattice nodes
- Beat envelope showing the interference pattern
- Penrose-inspired rotating geometric patterns (golden spiral, concentric polygons)
- Emergence constellation (living nodes connected by harmonic relationships)

**Interface**
- Vim-inspired keybindings (h/j/k/l navigation)
- 5 brainwave presets with quick-select
- 5 timed sequences with smooth interpolation
- Aurora-style terminal stage with animated backdrop and spectral control deck
- Session timer and breathing pacer
- Context-sensitive help overlay
- Color palette tuned for dark terminal backgrounds

## Requirements

- Linux with PipeWire (or ALSA-compatible audio)
- A modern terminal emulator (alacritty, kitty, wezterm, foot)
- Rust toolchain (1.85+; edition 2024)
- Headphones (binaural beats require stereo separation)

## Installation

```bash
git clone https://github.com/YOU/microtube.git
cd microtube
cargo build --release
```

The binary lands at `target/release/microtube` (< 1MB stripped).

## Usage

```bash
cargo run --release
# or directly:
./target/release/microtube
```

### Keybindings

| Key | Action |
|-----|--------|
| `j` / `k` | Select parameter |
| `h` / `l` | Adjust value |
| `H` / `L` | Big adjustment (5x) |
| `Space` | Play / Pause |
| `1`-`5` | Quick preset |
| `p` | Preset menu |
| `s` | Sequence menu |
| `v` / `V` | Next / previous visualization |
| `e` | Toggle emergence |
| `g` | Switch spawn mode (canon ↔ penrose) |
| `n` | Toggle mist layer |
| `m` | Cycle mist type |
| `?` | Help |
| `q` / `Esc` | Quit |

### Presets

| # | Name | Frequency | Band | Use Case |
|---|------|-----------|------|----------|
| 1 | Deep Sleep | 2 Hz | Delta | Deep dreamless sleep |
| 2 | Meditation | 6 Hz | Theta | Creative visualization, deep meditation |
| 3 | Relaxation | 10 Hz | Alpha | Calm awareness, stress relief |
| 4 | Focus | 18 Hz | Beta | Concentration, alertness |
| 5 | Flow State | 40 Hz | Gamma | Peak performance, insight |

### Sequences

Timed programs that smoothly transition between brainwave bands:

- **Deep Focus** (25 min) — Beta to Alpha to Theta
- **Wake Up** (10 min) — Delta through Theta and Alpha to Beta
- **Power Nap** (20 min) — Alpha descent into Theta, then back up
- **Deep Meditation** (30 min) — Alpha into deep Theta territory and return
- **Orch-OR** (25 min) — Gamma to Schumann resonance (7.83 Hz) and back; inspired by Penrose-Hameroff theory

### Emergence Mode

Press `e` to bring the system to life. Voices emerge from silence, interact through harmonic consonance, and fade back into the carrier wave. The intensity parameter controls how active the generative system is:

- **Low (20-30%)** — Subtle shimmer beneath the primary tone
- **Medium (50%)** — Distinct harmonic voices audible as background texture
- **High (80-100%)** — Full generative counterpoint; a self-composing canon

The system follows a repeating canon pattern through harmonic ratios (perfect fifths, major thirds, golden ratio intervals), transposing every 8 spawns like a fugue shifting key. Voice lifetimes are proportional to their consonance with the harmonic series — the universe rewards simplicity but tolerates novelty.

Press `g` to switch the spawn engine to **Penrose** mode. Each spawn now advances a step along a Conway worm — a row of parallel rhombs through a Penrose P3 tiling — whose tile sequence is the Fibonacci word, the canonical 1D quasicrystal. The substitution `L → LS, S → L` produces an aperiodic but self-similar binary stream; pairs of consecutive tiles select the harmonic move:

| Pair | Ratio | Move | Asymptotic frequency |
|------|-------|------|----------------------|
| LL | 3:2 | perfect fifth (anchor) | 1/φ³ ≈ 23.6% |
| LS | 5:4 | major third (descent) | 1/φ² ≈ 38.2% |
| SL | 4:3 | perfect fourth (ascent) | 1/φ² ≈ 38.2% |

(SS never occurs in the Fibonacci word — every short rhomb is bracketed by long ones.) The resulting harmonic stream is structurally aperiodic at every scale and yet bound to a small consonant palette, so it never repeats but always sounds like itself.

## Architecture

```
src/
├── main.rs            Entry, terminal setup, event loop
├── app.rs             Application state, lock-free AudioParams
├── audio.rs           cpal stream, synthesis, emergence integration
├── emergence.rs       Generative voice engine (canon + cellular rules)
├── presets.rs         Brainwave presets and timed sequences
├── ui.rs              ratatui layout and widget composition
├── penrose.rs         Fibonacci-word walk (Penrose P3 Conway worm)
└── visualization.rs   Braille waveforms, spectrum, Penrose, emergence viz
```

**Thread model:** Three threads communicate lock-free.

```
┌──────────────────┐  Arc<AtomicU32>  ┌─────────────────────┐
│   Main Thread    │◄────────────────►│  Audio Thread (cpal) │
│  (UI @ 30fps)   │                  │  (synthesis @ 48kHz) │
│                  │◄─ Mutex<VizBuf> ─│                      │
│                  │◄─ Mutex<Snap>  ──│  EmergenceEngine     │
└──────────────────┘                  └─────────────────────┘
```

Parameters flow to the audio thread via atomic `f32`-as-`u32` bit reinterpretation. Visualization samples flow back via a ring buffer behind a `try_lock` (never blocks the audio thread). The emergence snapshot updates at 30Hz for the constellation display.

## How Binaural Beats Work

When two tones of slightly different frequency are presented separately to each ear, the brain perceives a phantom "beat" at the difference frequency:

- Left ear: `sin(2π × 220 × t)`
- Right ear: `sin(2π × 230 × t)`
- Perceived beat: 10 Hz (Alpha band)

This 10 Hz pulse corresponds to the Alpha brainwave band, associated with relaxed wakefulness. The theory of brainwave entrainment suggests the brain's dominant frequency tends to synchronize with this external stimulus.

**Headphones are required** — speakers mix the channels and destroy the binaural effect.

## Variations on a Theme

Ideas for future exploration — each a self-contained direction this instrument could grow:

**Isochronal Tones**
Pulsed mono tones as an alternative entrainment method. Unlike binaural beats, these work through speakers. The pulse rate creates the entrainment frequency directly. Could coexist as a toggle alongside the binaural engine.

**Spectral Morphing**
Smoothly interpolate the harmonic spectrum between different timbres (flute → organ → bell) while maintaining the binaural beat. The warmth parameter is a first step; full spectral control would allow the tone itself to evolve over a session.

**Polyrhythmic Emergence**
Extend the emergence engine with rhythmic awareness — voices spawn not just at harmonic intervals but at polyrhythmic time offsets. A 3-against-4 pattern in the voice spawning would create temporal interference patterns alongside the frequency relationships.

**Stochastic Resonance**
Add noise not as a masking agent but as a functional component — at certain levels, noise actually enhances the brain's ability to detect weak periodic signals (stochastic resonance). An adaptive noise floor that maximizes this effect.

**Biofeedback Loop**
Read heart rate variability or breathing rate (via external sensor) and use it as a control signal. The system responds to your state rather than imposing one — speeding up when you're drowsy during focus sessions, deepening when your breathing slows during meditation.

**Shepard-Risset Integration**
Continuously ascending/descending Shepard tones layered with the binaural beat. The auditory illusion of infinite ascent combined with entrainment creates a powerful sense of momentum or deepening.

**Network Emergence**
Multiple MicroTube instances communicating over the network, each contributing voices to a shared emergence pool. Distributed generative music — a flock of synthesizers finding consonance together.

**Microtonal Exploration**
Escape 12-TET entirely. The emergence engine already uses golden-ratio intervals; extend this to Bohlen-Pierce scale, 19-TET, or pure just intonation lattices. Each tuning system produces different emergent consonance patterns.

**Session Memory**
Log which frequencies and sequences correlate with self-reported states over time. Build a personal model: "Tuesday morning focus sessions work best starting at Beta 16Hz rather than 18Hz." The instrument learns its player.

## License

Unlicense

## Acknowledgments

- [ratatui](https://ratatui.rs) — Terminal UI framework
- [cpal](https://github.com/RustAudio/cpal) — Cross-platform audio
- Roger Penrose — For the geometry, the impossible triangles, and the audacious claim that consciousness arises from quantum gravity
- Douglas Hofstadter — For *Gödel, Escher, Bach* and the concept of strange loops
- John Conway — For showing that simple rules generate unbounded complexity
- J.S. Bach — For proving that counterpoint is the mathematics of the soul
