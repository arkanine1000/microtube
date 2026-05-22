# MicroTube Web

A React + WebAssembly front-end for the MicroTube binaural-beat engine. The
DSP runs in `microtube-core` (Rust), compiled to Wasm and executed inside an
`AudioWorklet` on the browser's audio render thread.

## Develop

From the **repository root**:

```bash
npm install                       # once — installs workspace deps
npm run dev                        # builds the Wasm core, then runs Vite
```

`npm run dev` first runs `wasm-pack` once, then concurrently starts:

1. a `cargo watch` that rebuilds the Wasm core on Rust changes
   (needs `cargo install cargo-watch` — optional; without it, re-run
   `npm run wasm:build` manually after editing `crates/core`), and
2. the Vite dev server at <http://localhost:5173>.

Use headphones — the binaural effect depends on per-ear separation.

## Localization

The UI copy lives in a typed internal dictionary at `src/i18n/copy.ts`, wired
through `LocaleProvider`. English is the default language, Croatian is
available from the start-screen selector, and explicit choices persist in
`localStorage` before falling back to browser language detection for `hr*`
locales.

## Progressive Web App

The web UI is installable as a PWA. Manifest metadata, the service worker, and
install icons live in `public/` so Vite copies them directly into
`apps/web/dist`.

- `public/manifest.webmanifest` defines the app name, standalone display mode,
  start URL, theme colors, categories, and standard/maskable icon entries.
- `public/sw.js` is intentionally install-focused and network-only: it
  registers a service worker for installability, but does not cache the app
  shell, worklet, or Wasm for offline playback.
- PWA icons are generated from `public/microtube-icon.png`. Use ImageMagick's
  `convert` command to regenerate `apple-touch-icon.png`, `pwa-192.png`,
  `pwa-512.png`, and the maskable variants after changing the source artwork.
- Vercel serves `/sw.js` and `/manifest.webmanifest` with no-cache headers from
  the root `vercel.json`, so installed clients can pick up redeploys promptly.

## Build

```bash
npm run build      # release Wasm + tsc + vite build  ->  apps/web/dist
```

## How the pipeline fits together

- `crates/core` — pure Rust DSP, `crate-type = ["rlib", "cdylib"]`.
- `public/microtube-worklet/wasm/` — `wasm-pack` output (git-ignored).
- `worklet/processor.src.js` + `scripts/build-worklet.mjs` — the latter
  prepends a `TextDecoder` shim and the wasm-bindgen glue to the former and
  emits `public/microtube-worklet/processor.js`: a single, **import-free**
  `AudioWorkletProcessor` module (nested imports inside a worklet are
  unreliable). Regenerated on every `wasm:build`; git-ignored.
- The worklet scope has no dependable `fetch`, so the main thread fetches
  the raw `.wasm` bytes and transfers the `ArrayBuffer` into the worklet;
  `initSync` compiles + instantiates them there.
- Parameter changes flow main-thread → worklet over the `MessagePort`.
- User-facing web labels are kept out of the audio/domain parameter metadata;
  localized labels, hints, presets, EEG band text, sequence names, and step
  names come from `src/i18n/copy.ts`.
- The service worker is registered only in production from `src/main.tsx`;
  local `npm run dev` sessions stay unregistered unless a browser still has an
  older worker from a previous production preview.
