# MicroTube

A binaural beats engine with generative audio, built in Rust. It ships as two front-ends over one shared DSP core: a terminal-native app and a browser app.

MicroTube synthesizes binaural beats in real-time. The terminal app (`crates/cli`) streams stereo audio via cpal and renders animated visualizations using Unicode braille characters; the web app (`apps/web`) runs the same engine compiled to WebAssembly inside an `AudioWorklet`. Both ride the pure DSP core in `crates/core`.

## Features

**Audio Synthesis**
- Phase-accumulator binaural beat generation (drift-free, runs indefinitely)
- Harmonic overtones (2nd/3rd/4th partials) for warm, organ-like timbre
- Selectable mist algorithms: pink, white, brown, blue, and velvet textures
- Shepard-Risset glissando: a 7-octave stack under a raised-cosine bell, endlessly rising or falling
- Exponential parameter smoothing (~50ms) eliminates clicks on transitions
- Soft-clip limiter prevents digital distortion

**Emergence Engine**
- Generative audio system inspired by Conway's Game of Life and Bach canons
- Up to 12 simultaneous voices spawning at harmonic/golden-ratio intervals
- Built-in HRTF-style spatialization gives each voice binaural position cues
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
- 6 timed sequences with smooth interpolation, including the 25½-minute *Journey Through the Cosmos*
- Sequence steps automate every audible & visible parameter (frequencies, timbre, mist, Shepard direction, spawn mode, visualisation) — not just the binaural carrier
- Audio-reactive deep-space backdrop: aurora curtains, sparse twinkling stars, and an RMS-driven haze whose drift speed tracks the beat frequency
- Borderless visualization stage with a floating viz-mode sigil and a soft inner-glow that brightens with the audio
- Band-keyed accent that tints the whole UI to match the active brainwave band; meters glow with a 600 ms afterglow on every adjustment
- Single information column stacking parameters, session readouts, and the H1–H6 partial bars with hairline dividers
- Sequence film-strip below the stage with a luminous "head" tracking the active step
- Contextual key-hint footer that adapts to the current mode and collapses gracefully on narrow terminals
- Two-column help overlay with grouped commands and lore
- Adjustable timer with auto-stop, breathing pacer, and live `epoch` indicator during journey-class sequences

## Requirements

- Linux with PipeWire (or ALSA-compatible audio)
- A modern terminal emulator (alacritty, kitty, wezterm, foot)
- Rust toolchain (1.85+; edition 2024)
- Headphones (binaural beats require stereo separation)
- For the web app: Node 18+ (see [apps/web/README.md](apps/web/README.md))

## Installation

```bash
git clone https://github.com/arkanine1000/microtube.git
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
| `a` | Toggle timer |
| `1`-`5` | Quick preset |
| `p` | Preset menu |
| `s` | Sequence menu |
| `v` / `V` | Next / previous visualization |
| `e` | Toggle emergence |
| `g` | Switch spawn mode (canon ↔ penrose) |
| `r` | Toggle Shepard-Risset drift |
| `R` | Reverse drift direction (rising ↔ falling) |
| `n` | Toggle mist layer |
| `m` | Cycle mist type |
| `t` | Cycle timbre |
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

Timed programs that smoothly transition between brainwave bands. The first five automate `beat_freq` and `base_freq` and leave every other parameter under manual control. The sixth — **Journey Through the Cosmos** — automates every parameter in the program: timbre, mist texture, Shepard intensity and direction, emergence intensity, spawn mode, even the active visualisation. See the companion essay [JOURNEY_THROUGH_THE_COSMOS.md](JOURNEY_THROUGH_THE_COSMOS.md) for the design notes, the science (Penrose–Hameroff, Babcock superradiance, Schumann resonance, Fibonacci quasicrystals, Shepard–Risset), and a full citation list.

- **Deep Focus** (25 min) — Beta to Alpha to Theta
- **Wake Up** (10 min) — Delta through Theta and Alpha to Beta
- **Power Nap** (20 min) — Alpha descent into Theta, then back up
- **Deep Meditation** (30 min) — Alpha into deep Theta territory and return
- **Orch-OR** (25 min) — Gamma to Schumann resonance (7.83 Hz) and back; inspired by Penrose-Hameroff theory
- **Journey Through the Cosmos** (25½ min) — A 13-step strange loop. Microtubule → Synapse → Brain → Body → Earth (Schumann) → Lunar → Solar → Stellar → Galactic → Cosmic Web → Background Radiation → Singularity → back to Microtubule. Step durations follow the Fibonacci sequence (21, 34, 55, 89, 144, 233, 377 s rising and descending symmetrically); base frequency descends monotonically from 432 Hz to 55 Hz before snapping back; beat frequency traces a U from gamma through delta to gamma; Shepard direction flips Down once, at the singularity. Headphones, dimmed lights, no interruptions.

### Emergence Mode

Press `e` to bring the system to life. Voices emerge from silence, interact through harmonic consonance, and fade back into the carrier wave. The intensity parameter controls how active the generative system is:

- **Low (20-30%)** — Subtle shimmer beneath the primary tone
- **Medium (50%)** — Distinct harmonic voices audible as background texture
- **High (80-100%)** — Full generative counterpoint; a self-composing canon

Each emergence voice is spatialized with a lightweight built-in HRTF approximation: short interaural timing differences, level differences, and far-ear softening. This gives headphones a stronger sense of position without adding measured HRIR assets or convolution latency.

The system follows a repeating canon pattern through harmonic ratios (perfect fifths, major thirds, golden ratio intervals), transposing every 8 spawns like a fugue shifting key. Voice lifetimes are proportional to their consonance with the harmonic series — the universe rewards simplicity but tolerates novelty.

Press `g` to switch the spawn engine to **Penrose** mode. Each spawn now advances a step along a Conway worm — a row of parallel rhombs through a Penrose P3 tiling — whose tile sequence is the Fibonacci word, the canonical 1D quasicrystal. The substitution `L → LS, S → L` produces an aperiodic but self-similar binary stream; pairs of consecutive tiles select the harmonic move:

| Pair | Ratio | Move | Asymptotic frequency |
|------|-------|------|----------------------|
| LL | 3:2 | perfect fifth (anchor) | 1/φ³ ≈ 23.6% |
| LS | 5:4 | major third (descent) | 1/φ² ≈ 38.2% |
| SL | 4:3 | perfect fourth (ascent) | 1/φ² ≈ 38.2% |

(SS never occurs in the Fibonacci word — every short rhomb is bracketed by long ones.) The resulting harmonic stream is structurally aperiodic at every scale and yet bound to a small consonant palette, so it never repeats but always sounds like itself.

### Shepard-Risset Drift

Press `r` to layer a continuous Shepard-Risset glissando over the binaural carrier; press `R` to flip its direction. Seven sine oscillators spaced one octave apart sweep in parallel through a raised-cosine amplitude window centered around 370 Hz by default; the bell fully silences each oscillator at the spectrum's edge, so the wrap-around is inaudible and the pitch appears to rise (or fall) forever.

The default rate is 36 seconds per octave — slow enough to feel ambient, fast enough that the motion remains legible. The Drift Base parameter moves the bottom of the seven-octave stack from C0 to C3, with C1 as the default. Descending drift pairs naturally with the meditation/sleep presets ("a feeling of falling", per Mainsbridge & Marques 2016); rising drift adds momentum to focus and flow sessions. The layer is summed mono and mixed equally to both ears so it does not interfere with the binaural difference frequency.

## Web app

`apps/web` is a React + TypeScript front-end that runs the same synthesis engine in the browser: `crates/core` is compiled to WebAssembly with `wasm-pack` and executed inside an `AudioWorklet`. It is a mobile-first studio — a sticky transport bar carrying the play/pause control, a persistent strip of brainwave presets keyed to the five EEG bands, and two tabs: **Play** (transport, modes, and the *Journey Through the Cosmos* sequence) and **Shape** (tone parameters in collapsible, icon-led groups). Panels collapse by default to keep first contact calm, the accent colour retints the whole UI to the active band, and an auto-stop timer counts the session down in the transport bar. Visualizations are terminal-only for now.

```bash
npm install        # from the repo root, once
npm run dev        # builds the Wasm core, then serves at localhost:5173
```

See [apps/web/README.md](apps/web/README.md) for the dev loop and [DEPLOYMENT.md](DEPLOYMENT.md) for deploying to Vercel.

## Architecture

A Cargo + npm workspace. The pure DSP engine lives in `crates/core` and is shared, unchanged, by the native CLI (linked as an `rlib`) and the web app (compiled to WebAssembly).

```
crates/
├── core/   microtube-core — pure DSP, builds as rlib + cdylib
│   └── src/
│       ├── engine.rs        Sample-accurate synthesis engine + parameter smoothing
│       ├── synth.rs         Timbre/mist enums, noise-colour generators, soft limiter
│       ├── emergence.rs     Generative voice engine (canon + cellular rules)
│       ├── shepard.rs       Shepard-Risset glissando engine
│       ├── penrose.rs       Fibonacci-word walk (Penrose P3 Conway worm)
│       └── wasm.rs          wasm-bindgen bridge (WasmEngine), behind the `wasm` feature
└── cli/    microtube — the terminal app
    └── src/
        ├── main.rs            Entry, terminal setup, event loop
        ├── app.rs             App state, lock-free AudioParams, frame-coherent Signals
        ├── audio.rs           cpal stream driving core::Engine
        ├── presets.rs         Brainwave presets and timed sequences
        ├── local_presets.rs   User preset persistence
        ├── theme.rs           Palette, accent ramps, semantic colors, color math
        ├── ui.rs              ratatui layout, panels, modals, living backdrop
        ├── visualization.rs   Braille waveforms, spectrum, Penrose, emergence viz
        └── knowledge/         In-app wiki, glossary, and MicroTube guide

apps/
└── web/    React + WebAssembly browser app (Vite + TypeScript)
```

**Thread model (terminal app):** Three threads communicate lock-free.

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
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) & [wasm-pack](https://rustwasm.github.io/wasm-pack/) — The Rust → WebAssembly bridge
- [React](https://react.dev) & [Vite](https://vite.dev) — The web front-end
- Roger Penrose — For the geometry, the impossible triangles, and the audacious claim that consciousness arises from quantum gravity
- Stuart Hameroff — For taking that claim into the operating theatre, and for the 2014 *Physics of Life Reviews* paper that re-frames EEG bands as beat frequencies of microtubule vibrations
- N. S. Babcock, P. Kurian, and collaborators — For the 2024 *J. Phys. Chem. B* result on collective superradiance in tryptophan mega-networks, which puts the warm-wet-brain decoherence objection into a different, kinder light
- Douglas Hofstadter — For *Gödel, Escher, Bach* and the concept of strange loops
- W. O. Schumann and Herbert König — For showing the Earth has a fundamental frequency, and that it sits inside the alpha-theta band
- Roger Shepard and Jean-Claude Risset — For the auditory illusion that lets a tone climb forever
- John Conway — For showing that simple rules generate unbounded complexity, and for the worms
- M. C. Escher — For hands drawing hands
- J. S. Bach — For *Canon per Tonos*, the contrapuntal Shepard tone, and for proving that counterpoint is the mathematics of the soul
