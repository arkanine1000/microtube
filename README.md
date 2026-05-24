# MicroTube

MicroTube is a small instrument for deliberate listening. At its center is a
Rust DSP engine that holds a stable binaural carrier over long sessions: one
tone for the left ear, a slightly higher tone for the right, and a controllable
beat in the space between them.

Around that carrier, the instrument can add harmonic color, colored-noise mist,
endless Shepard-Risset drift, and generative voices that follow canon patterns
or a Penrose/Fibonacci walk, or use Fuxian counterpoint rules for voice leading.
The same engine drives a keyboard-first terminal studio and a touch-friendly web
studio running in an `AudioWorklet`.

Headphones are required. Speakers mix the channels and remove the binaural
separation.

## Features

**Sound engine**

- Stable stereo carrier: left ear receives `base_freq`, right ear receives
  `base_freq + beat_freq`, and the beat lives in the difference.
- f64 phase accumulators wrap per sample, so long sessions stay stable instead
  of slowly drifting out of shape.
- Parameter smoothing absorbs preset changes, sequence steps, and fast control
  movements without clicks.
- Harmonic timbres preserve the binaural offset while adding Organ, Flute, Bell,
  or Saw color through upper partials.
- Mist textures add Pink, White, Brown, Blue, or Velvet noise with per-texture
  gain compensation.
- Shepard-Risset drift layers seven octave-spaced oscillators under a
  raised-cosine bell, creating pitch motion that can rise or fall indefinitely.
- Emergence adds up to 12 generative voices with consonance-weighted lifetimes,
  lightweight spatialization, bounded total energy, and Canon, Penrose, or
  Fuxian spawn modes.
- Soft limiting keeps dense combinations of carrier, mist, harmonics, drift, and
  emergence under control.

**Composed motion**

- Five quick presets cover the common Delta, Theta, Alpha, Beta, and Gamma band
  labels without locking the sound into a fixed session.
- Six built-in sequences range from simple carrier transitions to the
  25.5-minute Journey Through the Cosmos.
- Journey is a 13-step arc with Fibonacci-timed epochs and automation for
  timbre, mist, emergence, voice-leading gravity, Shepard-Risset drift, spawn
  mode, and visualization state.
- Penrose mode follows a Fibonacci-word walk through consonant interval choices,
  producing an ordered but non-repeating voice stream.
- Fuxian mode chooses from the consonance pool with leap recovery, no repeated
  fifth/octave parallels, and a gravity control that pulls lines back to root.

**Terminal studio**

- Native stereo output through `cpal`.
- Keyboard-first `ratatui` interface with Studio and Knowledge tabs.
- Waveform, spectrum, harmonic, envelope, Penrose, and emergence
  visualizations.
- Built-in presets, timed sequences, local preset persistence, and an auto-stop
  timer.
- In-app guide, wiki articles, glossary, and silent playground concepts.

**Web studio**

- React + TypeScript + Vite front end.
- The same Rust engine compiled with `wasm-pack` and hosted in an
  `AudioWorklet`.
- Touch-first controls with start-screen language selection, sticky transport,
  local presets, collapsible sound-shaping panels, and `Main` / `Sequences`
  tabs.
- Installable PWA metadata with a deliberately network-only service worker.
- English and Croatian localization.

## Requirements

- Rust toolchain 1.85 or newer.
- Headphones.
- For the terminal app: Linux with PipeWire or ALSA-compatible audio, plus a
  modern terminal emulator such as alacritty, kitty, wezterm, or foot.
- For the web app: Node.js 18 or newer, `wasm-pack`, and a browser with Web
  Audio `AudioWorklet` support.
- Optional for web development: `cargo-watch` for automatic Rust/Wasm rebuilds.

## Quick Start

Build and run the terminal app:

```bash
cargo run --release
```

Build the release binary:

```bash
cargo build --release
./target/release/microtube
```

Run the web app from the repository root:

```bash
npm install
npm run dev
```

The Vite server runs at <http://localhost:5173>. See
[apps/web/README.md](apps/web/README.md) for the web development workflow.

## Usage Notes

- Start at low volume. Binaural material is usually more effective when it is
  comfortable and unobtrusive.
- Press `?` in the terminal app for the current control reference.
- Press `Tab` in the terminal app to switch between Studio and Knowledge.
- Use presets for quick static sessions and sequences for timed motion.
- Local presets are available in both front ends and store the current
  sound-shaping state.

MicroTube is an audio instrument, not a medical device. Do not use it as a
substitute for medical care, and stop listening if you feel discomfort.

## Presets And Sequences

Presets are stable starting points. MicroTube includes five quick presets keyed
to common EEG band labels:

- Deep Sleep: Delta.
- Meditation: Theta.
- Relaxation: Alpha.
- Focus: Beta.
- Flow State: Gamma.

Sequences are composed sessions that move the instrument over time:

- Deep Focus.
- Wake Up.
- Power Nap.
- Deep Meditation.
- Orch-OR.
- Journey Through the Cosmos.

Most legacy sequences automate the binaural carrier. Journey Through the Cosmos
is the largest built-in sequence: a 13-step, 25.5-minute arc that also automates
timbre, mist, emergence, voice-leading gravity, Shepard-Risset drift, spawn
mode, and visualization state. See
[JOURNEY_THROUGH_THE_COSMOS.md](JOURNEY_THROUGH_THE_COSMOS.md) for its design
essay and citations.

## Development

The repository is a Cargo workspace plus an npm workspace. Common commands:

```bash
cargo test
npm run wasm:build
npm run dev
npm run build
npm run build --workspace @microtube/web
```

- `cargo test` runs the Rust workspace tests.
- `npm run wasm:build` builds the Rust core for the web app and regenerates the
  ignored worklet bundle.
- `npm run dev` builds Wasm once, starts a Rust/Wasm watcher, and starts Vite.
- `npm run build` builds release Wasm plus the web app into `apps/web/dist`.
- `npm run build --workspace @microtube/web` runs TypeScript/Vite only and
  expects the Wasm/worklet artifacts to already exist.

Deployment notes live in [DEPLOYMENT.md](DEPLOYMENT.md).

## Architecture

```text
crates/
├── core/   microtube-core: pure DSP, native rlib plus wasm cdylib
└── cli/    microtube: terminal app using cpal, ratatui, and crossterm

apps/
└── web/    React + TypeScript + Vite browser app
```

The core engine owns synthesis behavior and exposes a `Params` state shared by
both front ends. The CLI mirrors UI state into lock-free atomics consumed by the
audio thread. The web app mirrors React state to an `AudioWorklet` over a
`MessagePort`; the worklet renders through the Wasm engine on the browser audio
render thread.

Generated web audio artifacts are intentionally ignored:

- `apps/web/public/microtube-worklet/wasm/`
- `apps/web/public/microtube-worklet/processor.js`

Regenerate them with `npm run wasm:build`.

## Documentation

- [apps/web/README.md](apps/web/README.md): web development, build, PWA, and
  worklet notes.
- [DEPLOYMENT.md](DEPLOYMENT.md): Vercel build and deployment setup.
- [JOURNEY_THROUGH_THE_COSMOS.md](JOURNEY_THROUGH_THE_COSMOS.md): companion
  essay for the Journey sequence.
- `crates/cli/src/knowledge_assets/`: embedded guide, wiki, and glossary
  content compiled into the terminal app.

## License

Unlicense. See [LICENSE](LICENSE).

## Acknowledgments

- [ratatui](https://ratatui.rs) for the terminal UI framework.
- [cpal](https://github.com/RustAudio/cpal) for cross-platform audio.
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) and
  [wasm-pack](https://rustwasm.github.io/wasm-pack/) for the Rust-to-WebAssembly
  bridge.
- [React](https://react.dev) and [Vite](https://vite.dev) for the web front end.
- Roger Penrose for the geometry, the impossible triangles, and the audacious
  claim that consciousness arises from quantum gravity.
- Stuart Hameroff for taking that claim into the operating theatre, and for the
  2014 Physics of Life Reviews paper that reframes EEG bands as beat
  frequencies of microtubule vibrations.
- N. S. Babcock, P. Kurian, and collaborators for the 2024 J. Phys. Chem. B
  result on collective superradiance in tryptophan mega-networks.
- Douglas Hofstadter for Godel, Escher, Bach and the concept of strange loops.
- W. O. Schumann and Herbert Konig for showing the Earth has a fundamental
  frequency, and that it sits inside the alpha-theta band.
- Roger Shepard and Jean-Claude Risset for the auditory illusion that lets a
  tone climb forever.
- John Conway for showing that simple rules generate unbounded complexity, and
  for the worms.
- M. C. Escher for hands drawing hands.
- J. S. Bach for Canon per Tonos, the contrapuntal Shepard tone, and for proving
  that counterpoint is the mathematics of the soul.
