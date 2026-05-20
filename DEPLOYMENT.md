# Deploying the MicroTube web app to Vercel

The web app is a fully static bundle (`apps/web/dist`) — no serverless
functions, no runtime backend. The only wrinkle is that the build compiles
Rust to WebAssembly, and Vercel's build image has no Rust toolchain.

## One-time Vercel project setup

1. **Import** the Git repository into Vercel.
2. **Root Directory — this is the one that matters.** It MUST be the
   repository root (leave the field blank / `.`).

   > Vercel's monorepo auto-detection often sees the Vite app and silently
   > sets the Root Directory to `apps/web`. That breaks the build: the
   > Cargo workspace, `crates/core`, and `scripts/` all live above
   > `apps/web` and become unreachable. If it was set to `apps/web`, open
   > **Project → Settings → Build & Deployment → Root Directory**, clear it
   > back to the repository root, and redeploy.
3. **Framework Preset:** `Other`. All build settings come from `vercel.json`,
   so the dashboard fields can be left on their defaults.
4. No environment variables are required.

With the Root Directory at the repo root, `vercel.json` does the rest.

## Troubleshooting

**`Command "bash scripts/vercel-build.sh" exited with 127` /
`bash: scripts/vercel-build.sh: No such file or directory`**

The build ran somewhere other than the repository root — almost always the
Root Directory is set to `apps/web`. Fix it per step 2 above (set Root
Directory to the repository root) and redeploy. `scripts/vercel-build.sh`
exists only at the repo root, by design — the build needs the whole
workspace.

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
npx vite preview --outDir apps/web/dist   # serve the built bundle
```

## Notes

- The app marshals parameters over a `MessagePort`, not a
  `SharedArrayBuffer`, so it needs **no** cross-origin-isolation headers
  (`COOP`/`COEP`). If a future change adopts `SharedArrayBuffer`, add them
  back in `vercel.json` and Vite's dev server.
- `processor.js` and `microtube_core_bg.wasm` have stable (unhashed) names;
  they are served with Vercel's default revalidating cache so a redeploy is
  picked up immediately.
