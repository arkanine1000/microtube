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
