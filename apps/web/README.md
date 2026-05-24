# MicroTube Web

`apps/web` is the browser front end for MicroTube. It is a React + TypeScript +
Vite app that runs the shared Rust DSP core as WebAssembly inside an
`AudioWorklet`.

This README is for web development. The project overview lives in the root
[README.md](../../README.md), and deployment notes live in
[DEPLOYMENT.md](../../DEPLOYMENT.md).

## Requirements

- Node.js 18 or newer.
- Rust 1.85 or newer.
- `wasm-pack` for building `crates/core` to WebAssembly.
- Optional: `cargo-watch` for automatic Rust/Wasm rebuilds during development.
- Headphones for manual audio testing.

Install JavaScript dependencies from the repository root:

```bash
npm install
```

## Develop

From the repository root:

```bash
npm run dev
```

The root dev command does three things:

- Runs `npm run wasm:build` once.
- Starts `npm run wasm:watch`, which watches `crates/core` and rebuilds the
  Wasm/worklet artifacts after Rust changes.
- Starts the Vite dev server for `@microtube/web` at
  <http://localhost:5173>.

If `cargo-watch` is not installed, use this manual loop instead:

```bash
npm run wasm:build
npm run dev --workspace @microtube/web
```

Then rerun `npm run wasm:build` whenever `crates/core` changes.

## Build

Build the full production web app from the repository root:

```bash
npm run build
```

This runs release Wasm packaging and then `tsc && vite build`. The static output
lands in `apps/web/dist`.

When the Wasm/worklet artifacts already exist and only TypeScript or React code
changed, this narrower command is useful:

```bash
npm run build --workspace @microtube/web
```

Preview an existing production build:

```bash
npm run preview --workspace @microtube/web
```

## Wasm And Worklet Pipeline

The browser app does not call the Rust engine from the main thread. Audio runs
on the browser audio render thread:

- `crates/core` builds as both `rlib` and `cdylib`.
- `npm run wasm:build` runs `wasm-pack build crates/core --target web
  --features wasm`.
- `wasm-pack` writes ignored generated files to
  `apps/web/public/microtube-worklet/wasm/`.
- `apps/web/scripts/build-worklet.mjs` combines the wasm-bindgen glue with
  `apps/web/worklet/processor.src.js`.
- The generated, import-free worklet module is written to
  `apps/web/public/microtube-worklet/processor.js`.

The worklet module is intentionally import-free because nested imports from
`AudioWorkletProcessor` modules are unreliable across browsers. The main thread
fetches `/microtube-worklet/wasm/microtube_core_bg.wasm`, transfers the raw
`ArrayBuffer` into the worklet, and the worklet initializes the Wasm engine with
`initSync`.

Rendering uses Wasm-owned buffers and cached `Float32Array` views over Wasm
memory. Parameter changes flow from React state to the worklet over the
`MessagePort`.

## Source Layout

- `src/App.tsx`: top-level studio UI and tab layout.
- `src/audio/useMicroTube.ts`: Web Audio session lifecycle, worklet messaging,
  timer, Media Session integration, and sequence execution.
- `src/audio/params.ts`: parameter metadata, EEG bands, enum values, and slider
  specs.
- `src/audio/sequences.ts`: web presets and executable timed sequences.
- `src/audio/localPresets.ts`: localStorage persistence and validation.
- `src/i18n/`: typed English and Croatian copy.
- `src/components/`: reusable UI panels and controls.
- `worklet/processor.src.js`: source for the generated audio worklet.
- `scripts/build-worklet.mjs`: worklet bundling step.
- `public/`: static PWA metadata, icons, service worker, and generated worklet
  output.

## Progressive Web App

The web app is installable as a PWA:

- `public/manifest.webmanifest` defines app metadata, standalone display mode,
  theme colors, categories, and standard/maskable icons.
- `public/sw.js` exists for installability and is intentionally network-only.
  It does not cache the app shell, worklet, or Wasm for offline playback.
- The service worker is registered only in production from `src/main.tsx`.
- `vercel.json` serves `/sw.js` and `/manifest.webmanifest` with no-cache
  headers so installed clients can pick up redeploys.
- Vite-hashed `/assets/*` files are served with immutable caching in
  production.

PWA icons are generated from `public/microtube-icon.png`. If the source artwork
changes, regenerate:

- `apple-touch-icon.png`
- `pwa-192.png`
- `pwa-512.png`
- `pwa-maskable-192.png`
- `pwa-maskable-512.png`

## Localization

The UI copy lives in `src/i18n/copy.ts` and is provided through
`LocaleProvider`.

- English is the default locale.
- Croatian is available from the start-screen language selector.
- Explicit user choices persist in `localStorage`.
- First visits fall back to browser language detection for `hr*` locales.
- User-facing labels stay out of audio/domain parameter metadata; localized
  labels, hints, preset text, EEG band text, sequence names, and step names come
  from the i18n dictionary.

## Notes

- Use headphones when testing audio behavior. Speakers mix the channels and
  destroy the binaural separation.
- The app does not require COOP/COEP headers because it does not use
  `SharedArrayBuffer`; parameters move over `MessagePort`.
- If a production service worker from a previous preview sticks around during
  local development, unregister it from the browser devtools Application panel.
- Visualizations are currently terminal-only. The web app focuses on audio,
  control ergonomics, local presets, sequences, and PWA installation.
