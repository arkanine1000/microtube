# Deploying the MicroTube web app to Vercel

The web app is a fully static bundle (`apps/web/dist`) — no serverless
functions, no runtime backend. The only wrinkle is that the build compiles
Rust to WebAssembly, and Vercel's build image has no Rust toolchain.

## One-time Vercel project setup

1. **Import** the Git repository into Vercel.
2. **Root Directory:** it must be the repository root (leave the field
   blank / `.`). Vercel's monorepo detection tends to auto-set this to
   `apps/web`; if so, clear it — the build must run from the workspace
   root to reach `crates/core` and `scripts/`.
3. **Framework Preset:** `Other`. Every build setting comes from
   `vercel.json`, so leave the dashboard's *Build & Development Settings*
   overrides **off** — a stale override there will fight `vercel.json`.
4. No environment variables are required.

That is it — `vercel.json` does the rest.

## What `vercel.json` declares

| Field             | Value                          | Why                                                        |
| ----------------- | ------------------------------ | ---------------------------------------------------------- |
| `buildCommand`    | `bash scripts/vercel-build.sh` | Installs the Rust/Wasm toolchain, then builds.             |
| `outputDirectory` | `apps/web/dist`                | The static bundle Vite emits.                              |
| `framework`       | `null`                         | Custom monorepo — no framework auto-detection.             |
| `headers`         | immutable cache for `/assets/*`| Vite content-hashes those files, so they never go stale.   |

The install step (`npm install`) is left as Vercel's default; the npm
workspace pulls in `apps/web`'s dependencies.

## What the build does (`scripts/vercel-build.sh`)

1. Installs `rustup` (minimal profile) if `cargo` is absent.
2. Adds the `wasm32-unknown-unknown` target.
3. Installs `wasm-pack` (prebuilt binary; falls back to `cargo install`).
4. Runs `npm run build`:
   - `wasm-pack build crates/core` → `apps/web/public/microtube-worklet/wasm/`
   - `scripts/build-worklet.mjs` assembles the self-contained AudioWorklet
     and prunes the wasm output to just `microtube_core_bg.wasm`
   - `tsc && vite build` → `apps/web/dist`

The first build installs the toolchain (~1–2 min); subsequent builds in a
warm cache are faster. The script is idempotent and safe to run locally.

## Local production build

```bash
npm run build          # if the Rust/Wasm toolchain is already installed
bash scripts/vercel-build.sh   # installs the toolchain first, like Vercel
npm run preview --workspace @microtube/web   # serve the built bundle
```

## Troubleshooting

Both common failures come from the Vercel **dashboard**, not the repo:

- **`bash: scripts/vercel-build.sh: No such file or directory` (exit 127)** —
  the build is not running from the repo root. The Root Directory is set to
  `apps/web`; clear it (step 2).
- **`npm error Tracker "idealTree" already exists`** — a stale Install
  Command override is set in the dashboard. Turn the Install/Build/Output
  overrides off under *Settings → Build & Deployment* (step 3).

## Notes

- The app marshals parameters over a `MessagePort`, not a
  `SharedArrayBuffer`, so it needs **no** cross-origin-isolation headers
  (`COOP`/`COEP`). If a future change adopts `SharedArrayBuffer`, add them
  back in `vercel.json` and Vite's dev server.
- `processor.js` and `microtube_core_bg.wasm` have stable (unhashed) names;
  they are served with Vercel's default revalidating cache so a redeploy is
  picked up immediately.
