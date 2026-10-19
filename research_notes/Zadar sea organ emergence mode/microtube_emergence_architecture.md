# MicroTube Emergence Architecture (for a new "Sea Organ" spawn mode)

Sources are local files in `/home/user/microtube` (repo HEAD `f65f429`). Citations use `path:line`. All line numbers verified by reading the files on 2026-10-19. `cargo test -p microtube-core` was run and passed (25 tests).

## 1. How are emergence voices represented, spawned, enveloped, aged/killed, spatialized and energy-bounded? What are "consonance-weighted lifetimes"?

### Takeaway
Emergence is a single `EmergenceEngine` in `crates/core/src/emergence.rs` holding up to 12 `Voice`s (pure sine + carrier-matched partials 2..6), each with a sin² "bell" envelope over a fixed lifetime of 4–12 s, where the lifetime is set at birth by the consonance score of the child/parent interval ("consonance-weighted lifetime"). Voices spawn on a timer (3 s / (0.5 + 1.5·intensity)) from the loudest living parent, are panned at random in ±0.7 through a per-voice HRTF-style spatializer, and the bus is scaled down when summed amplitude exceeds 2.0.

### Cited Findings
**Constants**
- `PHI = 1.618033988749895`, `MAX_VOICES = 12` — [emergence.rs:19-22](crates/core/src/emergence.rs)
- HRTF constants: delay buffer 256 samples, max ITD 0.65 ms, shadow attenuation 0.28, near cutoff 11 kHz, far cutoff 2.4 kHz; comment says "HRTF-style approximation, not measured HRIR convolution" — [emergence.rs:24-32](crates/core/src/emergence.rs)
- `CONSONANCE_RATIOS` table `(ratio, peak_score)`: 1:1 →1.0, 2:1 →0.95, 3:2 →0.9, 4:3 →0.85, 5:4 →0.8, 6:5 →0.75, 9:8 →0.68, 16:15 →0.62, φ →0.7, φ² →0.65 — [emergence.rs:37-48](crates/core/src/emergence.rs)
- `pub fn consonance_score(ratio)`: inverts ratios <1, triangular kernel of half-width 0.1 in log space, octave-folded distance for non-unison targets (`consonance_distance`), max contribution wins; doc comment says it is the single source of truth for both the audio engine and the Knowledge-tab playground — [emergence.rs:50-85](crates/core/src/emergence.rs)
- `SPAWN_RATIOS` (11 entries: 1/φ, 1/2, 2/3, 3/4, 1, 5/4, 4/3, 3/2, φ, 2, φ²) — [emergence.rs:87-100](crates/core/src/emergence.rs)

**Voice struct** (`#[derive(Clone)] pub struct Voice`) — [emergence.rs:102-115](crates/core/src/emergence.rs)
- `interval_from_root: f64` (ratio to base frequency), `trajectory: f64` (child/parent interval; >1 rises), `amplitude: f64` (0..1, recomputed every sample), `phase: f64` (normalized 0..1 phase accumulator), `age: f64` (s), `lifetime: f64` (s), `generation: u8`, `pan: f64` (−1..1), private `spatial: HrtfState`, private `alive: bool`.
- `Voice::new` starts at amplitude 0, phase 0, age 0 — [emergence.rs:117-137](crates/core/src/emergence.rs)
- `Voice::root()` = ratio 1.0, trajectory 1.0, lifetime 8.0 s, generation 0, pan 0 — [emergence.rs:139-141](crates/core/src/emergence.rs)

**Envelope** — `t = age/lifetime`; returns 0 at t≥1, else `sin(πt)²` (bell / raised-cosine, symmetrical attack and release, no sustain plateau despite the "attack/sustain/release" comment) — [emergence.rs:143-152](crates/core/src/emergence.rs)
- `is_alive()` = `alive && age < lifetime` — [emergence.rs:154-156](crates/core/src/emergence.rs)

**Per-sample processing** `EmergenceEngine::process(base_freq, intensity, gravity, harm_weights: &[f64;5], harm_intensity) -> (f64,f64)` — [emergence.rs:416-498](crates/core/src/emergence.rs)
- `dt = 1/sample_rate`; spawn timer check: `adjusted_interval = spawn_interval / (0.5 + intensity*1.5)` with `spawn_interval = 3.0` (so 6 s at intensity 0 to 1.5 s at intensity 1) — [emergence.rs:425-434](crates/core/src/emergence.rs), [emergence.rs:377](crates/core/src/emergence.rs)
- For each live voice: `age += dt`; `amplitude = envelope * intensity`; voices with amplitude < 0.001 are skipped (phase not advanced) — [emergence.rs:441-452](crates/core/src/emergence.rs)
- Frequency = `base_freq * interval_from_root`; phase accumulate and wrap — [emergence.rs:454-457](crates/core/src/emergence.rs)
- Timbre: fundamental sine plus partials 2..6 weighted by `harm_weights[i] * harm_intensity`, normalized by `1 + harm_intensity * Σweights` ("Dynamic timbre matching the carrier wave") — [emergence.rs:459-474](crates/core/src/emergence.rs)
- Voice gain: `sample * amplitude * 0.15` ("Background voice scale"), then `spatial.process(sample, pan, sample_rate)` — [emergence.rs:476-480](crates/core/src/emergence.rs)
- Energy bound: `total_energy = Σ amplitude`; if > 2.0 the summed L/R are scaled by `2.0/total_energy` — [emergence.rs:487-492](crates/core/src/emergence.rs)
- Dead voices removed with `self.voices.retain(|v| v.is_alive())` every sample — [emergence.rs:494-495](crates/core/src/emergence.rs)

**Spawning** `try_spawn(intensity, gravity)` — [emergence.rs:500-577](crates/core/src/emergence.rs)
- If `voices.len() >= MAX_VOICES`, the voice with least remaining life (`lifetime - age`) is marked dead and removed ("Kill the oldest voice to make room (conservation)") — [emergence.rs:501-511](crates/core/src/emergence.rs)
- If no voices remain, a root is pushed and the function returns ("Always maintain a root") — [emergence.rs:513-517](crates/core/src/emergence.rs)
- `epoch += 1`; parent = living voice with max amplitude, captured as `ParentInfo {interval_from_root, trajectory, generation}`, defaulting to root values — [emergence.rs:519-535](crates/core/src/emergence.rs), struct at [emergence.rs:342-347](crates/core/src/emergence.rs)
- Mode dispatch `match self.spawn_mode { Canon | Penrose | Fuxian }` produces `final_ratio` (child's interval_from_root) — [emergence.rs:537-556](crates/core/src/emergence.rs)
- **Consonance-weighted lifetime**: `realized_interval = final_ratio / parent.interval_from_root`; `consonance = consonance_score(realized_interval)`; `base_lifetime = 4.0 + consonance*8.0` (4–12 s); `lifetime = base_lifetime * (0.8 + rand*0.4)` (±20 % jitter) — [emergence.rs:558-563](crates/core/src/emergence.rs)
- `generation = parent.generation + 1`, capped at 7 — [emergence.rs:565](crates/core/src/emergence.rs)
- `pan = (rand*2-1)*0.7` (uniform −0.7..0.7) — [emergence.rs:567-568](crates/core/src/emergence.rs)
- New `Voice::new(final_ratio, realized_interval, lifetime, generation, pan)` is pushed — [emergence.rs:570-576](crates/core/src/emergence.rs)
- `fold_voice_ratio` octave-folds any ratio into [0.25, 4.0] (two octaves either side of base), returning 1.0 for invalid input — [emergence.rs:859-872](crates/core/src/emergence.rs)

**Spatialization** — `HrtfState { delay: [f64;256], write_idx, low_l, low_r }` per voice — [emergence.rs:159-185](crates/core/src/emergence.rs); `process()` maps pan to azimuth `pan·π/2`, writes to delay line, applies fractional ITD (`0.00065 s * sample_rate * sin(az)`), constant-power gains `cos/sin((pan+1)π/4)`, far-ear level shadow (×(1−0.28·shadow)), far-ear one-pole low-pass blending 11 kHz→2.4 kHz, and a 4 % "presence" dip — [emergence.rs:187-217](crates/core/src/emergence.rs); helpers `read_delay` (linear interp), `wrapped_delay_index`, `shape_far_ear`, `one_pole_alpha` — [emergence.rs:219-247](crates/core/src/emergence.rs). `HrtfState::reset()` exists but is not called by `Voice` creation (new voices get `HrtfState::default()`) — [emergence.rs:179-185](crates/core/src/emergence.rs), [emergence.rs:134](crates/core/src/emergence.rs)
- Pan is fixed for the voice's whole life (set only at spawn) — [emergence.rs:567-575](crates/core/src/emergence.rs)

**Engine state** `EmergenceEngine { voices: Vec<Voice>, rng: u64, time, sample_rate, spawn_timer, spawn_interval, epoch: u32, canon_pattern: Vec<usize>, canon_position: usize, canon_offset: f64, spawn_mode: SpawnMode, penrose_walk: PenroseWalk }` — [emergence.rs:325-340](crates/core/src/emergence.rs)
- `new()` preallocates `Vec::with_capacity(MAX_VOICES)`, seeds RNG `0xBAC0_CAFE_1685_1750`, canon pattern `[7,5,6,8,4,9,3,10]`, pushes a root voice — [emergence.rs:370-388](crates/core/src/emergence.rs)
- `reset()` clears voices, time, timer, epoch, canon position/offset, recreates PenroseWalk, re-pushes root (does NOT reset `rng` or `spawn_mode`) — [emergence.rs:390-399](crates/core/src/emergence.rs)
- `xorshift()` returns [0,1] — [emergence.rs:409-414](crates/core/src/emergence.rs)

**Visualization snapshot** — `EmergenceSnapshot { voices: Vec<VoiceInfo>, total_energy: f32, generation_count: u8, epoch: u32, spawn_mode, recent_tiles: Vec<Tile>, walk_position }` and `VoiceInfo { freq_ratio, amplitude, age_normalized, generation, pan }` — [emergence.rs:289-323](crates/core/src/emergence.rs); `snapshot()` collects Vecs and fills `recent_tiles` only in Penrose mode — [emergence.rs:668-701](crates/core/src/emergence.rs)

**Engine integration** — `Engine::process_frame` (crates/core/src/engine.rs)
- Syncs spawn mode each frame if changed: [engine.rs:161-164](crates/core/src/engine.rs)
- Emergence runs only when smoothed `current_emergence > 0.01`; passes `current_base`, `current_emergence`, `current_gravity`, `current_harm_weights`, `current_harmonics`; output multiplied by `current_vol` — [engine.rs:193-204](crates/core/src/engine.rs)
- When target emergence ≤ 0.01 and it was active, `emergence.reset()` is called — [engine.rs:205-208](crates/core/src/engine.rs)
- `emergence_snapshot()` returns empty snapshot when emergence off — [engine.rs:135-142](crates/core/src/engine.rs)

### Inferences
- A Sea Organ mode can reuse the whole voice pipeline (envelope, HRTF, energy bound, voice cap) by only producing a `final_ratio` in the `match` at emergence.rs:540-556; but authentic sea-organ behaviour (wave-driven onsets, breathy/noisy pipe tone, amplitude swelling with wave energy, fixed pipe positions in space) would need more than a ratio: likely per-voice extra fields (e.g. noise/breath amount, envelope shape, fixed pan per "pipe") or a mode-specific timing path, since spawn timing is currently a fixed periodic timer independent of mode.
- The sin² envelope is symmetric; a wave-driven "breath" (fast swell, slow decay, or repeated swells) would need either a new envelope variant on `Voice` or a per-voice amplitude modulator.
- Because consonance-weighted lifetime is computed generically after the dispatch, a Sea Organ mode inherits it automatically unless it overrides lifetime.

### Gaps
- No documentation/test explicitly specifying the energy-normalisation threshold rationale (2.0) beyond code comment "Energy conservation".

## 2. How are Canon, Penrose, Fuxian implemented, and where must a new variant be added?

### Takeaway
`SpawnMode` is a `#[repr(u32)]` enum (Canon=0, Penrose=1, Fuxian=2) with `label()`, `toggled()` (cycle) and `from_u32()` (fallback Canon). The integer discriminant is the wire format to WASM/worklet and the web UI; the lowercase `label()` string is the CLI local-preset JSON format. Adding "SeaOrgan = 3" touches ~20 locations across core, CLI, web and docs (enumerated below). Gravity is a global 0..1 param that only Fuxian currently reads.

### Cited Findings
**Enum & dispatch (core)**
- `pub enum SpawnMode { Canon = 0, Penrose = 1, Fuxian = 2 }`, doc comment describing each — [emergence.rs:249-261](crates/core/src/emergence.rs)
- `label()` → "canon"/"penrose"/"fuxian"; `toggled()` cycles Canon→Penrose→Fuxian→Canon; `from_u32` maps 1,2 else Canon — [emergence.rs:263-287](crates/core/src/emergence.rs)
- Dispatch `match self.spawn_mode` in `try_spawn` — [emergence.rs:540-556](crates/core/src/emergence.rs)

**Canon**: index from `canon_pattern[canon_position % len]` into `SPAWN_RATIOS`, then `canon_child_ratio(parent, base_ratio, intensity)` — [emergence.rs:541-546](crates/core/src/emergence.rs). `canon_child_ratio`: every 8 spawns (`canon_position % 8 == 0`) sets `canon_offset = SPAWN_RATIOS[(epoch/8) % 11]`; mutation `1 + (rand−0.5)*0.02*intensity`; result `fold_voice_ratio(parent * base * offset * mutation)` — [emergence.rs:579-593](crates/core/src/emergence.rs)

**Penrose**: `penrose_walk.step()` yields (prev, curr) tile; `penrose::pair_ratio(prev,curr)` gives base ratio; reuses `canon_child_ratio` — [emergence.rs:547-554](crates/core/src/emergence.rs). Fibonacci word via cut-and-project `tile_at(k)`; LL→3:2, LS→5:4, SL→4:3 (frequencies 23.6/38.2/38.2 %) — [penrose.rs:1-37](crates/core/src/penrose.rs); `PenroseWalk { position }` with `step()` — [penrose.rs:39-60](crates/core/src/penrose.rs). Snapshot exposes `recent_tiles` (24) only in Penrose mode — [emergence.rs:686-690](crates/core/src/emergence.rs).

**Fuxian**: `fuxian_child_ratio(parent, gravity)` builds a stack-allocated pool `[FuxianCandidate; 32]` from `CONSONANCE_RATIOS` and their inversions relative to the parent (dedup, excludes unison/parent), requires a step in the opposite direction if parent's trajectory leapt more than a minor third, falls back to a 2-entry step-recovery pool, else 1.0 — [emergence.rs:595-619](crates/core/src/emergence.rs), pool builders [emergence.rs:704-778](crates/core/src/emergence.rs). Constraints: no parallel fifths/octaves (`fuxian_parallel`), opposite-step rule — [emergence.rs:780-823](crates/core/src/emergence.rs). Weighted roulette selection with weight `(0.05 + score) * exp(pull * gravity * 6.0)` where `pull = |ln parent| − |ln candidate|` (closer to root ⇒ larger weight) — [emergence.rs:621-660](crates/core/src/emergence.rs), [emergence.rs:825-835](crates/core/src/emergence.rs), constants `FUXIAN_POOL_CAPACITY=32`, `FUXIAN_GRAVITY_STRENGTH=6.0`, `RATIO_EPSILON=1e-9` — [emergence.rs:349-367](crates/core/src/emergence.rs)

**Gravity parameter path**: `Params.gravity: f32` default 0.5 — [engine.rs:27](crates/core/src/engine.rs), [engine.rs:46](crates/core/src/engine.rs); smoothed `current_gravity` [engine.rs:176](crates/core/src/engine.rs); passed into `emergence.process` [engine.rs:199](crates/core/src/engine.rs) → `try_spawn` → only Fuxian uses it [emergence.rs:555](crates/core/src/emergence.rs). Web hint text "Fuxian pull toward the root" — [copy.ts:243-245](apps/web/src/i18n/copy.ts).

**Checklist — every place a new variant must be touched**

Core (`crates/core`)
1. Enum variant + `label()` + `toggled()` + `from_u32()` — [emergence.rs:249-287](crates/core/src/emergence.rs)
2. Dispatch arm in `try_spawn` (Rust `match` is exhaustive, so the compiler will force this) — [emergence.rs:540-556](crates/core/src/emergence.rs)
3. Any per-mode state on `EmergenceEngine` + init in `new()` + clearing in `reset()` — [emergence.rs:325-399](crates/core/src/emergence.rs)
4. Optional snapshot fields for viz (like `recent_tiles`) — [emergence.rs:289-314](crates/core/src/emergence.rs), [emergence.rs:668-701](crates/core/src/emergence.rs)
5. If the mode needs a new continuous parameter: `Params` field + default [engine.rs:17-54](crates/core/src/engine.rs), `current_*` smoothing [engine.rs:63-73, 96-113, 169-178](crates/core/src/engine.rs), `process` signature [emergence.rs:417-424](crates/core/src/emergence.rs), WASM setter [wasm.rs](crates/core/src/wasm.rs)
6. WASM: `set_spawn_mode(u32)` already generic via `from_u32` — [wasm.rs:137-139](crates/core/src/wasm.rs) (wasm.rs listed as 144 lines; setters at ~84-143). No change needed unless new params.
7. Unit tests in `mod tests` — [emergence.rs:874-1138](crates/core/src/emergence.rs)

CLI (`crates/cli`) — re-exports core modules via `pub use microtube_core::{emergence, penrose, shepard};` [main.rs:12-15](crates/cli/src/main.rs)
8. `AudioParams.spawn_mode: AtomicU32` stores `mode as u32`; `get_spawn_mode` via `from_u32` — [app.rs:53-54, 73, 150-156](crates/cli/src/app.rs) (generic; no change)
9. `toggle_spawn_mode()` uses `toggled()` — [app.rs:1012-1016](crates/cli/src/app.rs); keybinding `g` — [main.rs:210](crates/cli/src/main.rs); help overlay `("g", "spawn mode")` — [ui.rs:1578](crates/cli/src/ui.rs); help prose about Fuxian/Penrose — [ui.rs:~1632-1680](crates/cli/src/ui.rs)
10. Test `toggling_spawn_mode_does_not_change_visualization` asserts the cycle order Canon→Penrose→Fuxian — [app.rs:1295-1307](crates/cli/src/app.rs) (extend for the 4th mode)
11. Status line `"{n} voices · gen {g} · {mode_label} · g{gravity}"` uses `label()` — [ui.rs:789-805](crates/cli/src/ui.rs); gravity param row — [ui.rs:728-731](crates/cli/src/ui.rs)
12. Emergence visualization status `snapshot.spawn_mode.label()` and Penrose-only worm ribbon `if snapshot.spawn_mode == SpawnMode::Penrose` — [visualization.rs:644-680](crates/cli/src/visualization.rs); `render_emergence` places voices by `freq_ratio`/`pan` — [visualization.rs:536-642](crates/cli/src/visualization.rs). A Sea Organ ribbon (e.g. wave/swell indicator) would go here.
13. Local preset JSON: `spawn_mode: preset.spawn_mode.label().to_string()` on save — [local_presets.rs:83](crates/cli/src/local_presets.rs); `parse_spawn_mode` match on "canon"/"Canon"/"penrose"/... with error on unknown — [local_presets.rs:249-256](crates/cli/src/local_presets.rs) (MUST add e.g. "sea_organ"/"SeaOrgan" or presets saved in the new mode fail to load); serialization test — [local_presets.rs:~280-313](crates/cli/src/local_presets.rs)
14. Built-in sequences: `SequenceStep.spawn_mode: Option<SpawnMode>` and `epoch(...)` constructor; Journey steps use `SpawnMode::Canon/Penrose` — [presets.rs:19-110](crates/cli/src/presets.rs), [presets.rs:209-221](crates/cli/src/presets.rs). Sequence engine snaps discrete spawn mode — [app.rs:854-856, 933-935](crates/cli/src/app.rs). A dedicated "Sea Organ" preset/sequence would be added here.
15. Knowledge assets (compiled in via `include_str!`, adding an article requires an entry in [content.rs](crates/cli/src/knowledge/content.rs) ~lines 19-134): `knowledge_assets/microtube/emergence.md` (mode descriptions, [emergence.md:31-72](crates/cli/src/knowledge_assets/microtube/emergence.md)), `controls.md:50` ("`g` cycles the emergence spawn mode: Canon, Penrose, Fuxian."), `presets-and-sequences.md`, `wiki/consonance.md:75-77`, `glossary.toml` (entries for canon/fibonacci at ~106-124, 229, 269).

Web (`apps/web`)
16. `export type SpawnMode = 0 | 1 | 2;` and default `spawnMode: 0` — [params.ts:20-23, 56](apps/web/src/audio/params.ts) (header comment: enum orderings must mirror Rust exactly, [params.ts:1-5](apps/web/src/audio/params.ts))
17. i18n: `type Trio = readonly [string,string,string]` and `spawnModes: Trio` — [copy.ts:16, 120](apps/web/src/i18n/copy.ts); EN `['Canon','Penrose','Fuxian']` [copy.ts:224](apps/web/src/i18n/copy.ts); HR `['Kanon','Penrose','Fuxian']` [copy.ts:431](apps/web/src/i18n/copy.ts). Must become `Quad` (already defined at [copy.ts:17](apps/web/src/i18n/copy.ts)). Gravity hint text EN/HR [copy.ts:243-245, 450-452](apps/web/src/i18n/copy.ts).
18. `EmergenceSection` renders `<Segmented options={copy.modes.spawnModes} value={state.spawnMode}>` + emergence & gravity sliders — [EmergenceSection.tsx:22-35](apps/web/src/components/sections/EmergenceSection.tsx). Segmented uses option index as value — [Segmented.tsx:1-8](apps/web/src/components/Segmented.tsx). Four buttons may need mobile layout checks (recent commit "wrap mist options on mobile" handled the 5-option mist row).
19. `App.tsx` `setFeatureOption('spawnMode', v, 'emergence')` (auto-enables emergence) — [App.tsx:115-130, 184](apps/web/src/App.tsx)
20. Local presets validator `const SPAWN_MODES = [0, 1, 2] as const;` used by `enumField` — [localPresets.ts:32, 195](apps/web/src/audio/localPresets.ts) (must add 3 or saved presets are rejected)
21. Sequences: `SequenceStep.spawnMode?: SpawnMode`, comment legend "spawn: Canon 0 / Penrose 1 / Fuxian 2", Journey steps — [sequences.ts:44-49, 105-121](apps/web/src/audio/sequences.ts); `setDiscrete(..., 'spawnMode')` — [sequences.ts:187-200, 235, 261](apps/web/src/audio/sequences.ts); reset defaults in [useMicroTube.ts:150-166](apps/web/src/audio/useMicroTube.ts)
22. `LocalPresetsPanel` summary `copy.modes.spawnModes[snapshot.spawnMode]` — [LocalPresetsPanel.tsx:245](apps/web/src/components/LocalPresetsPanel.tsx)
23. Worklet message protocol: `{type:'param', name, value}` posted from [useMicroTube.ts:273-278](apps/web/src/audio/useMicroTube.ts); worklet `case 'spawnMode': e.set_spawn_mode(value >>> 0)` — [processor.src.js:109-128](apps/web/worklet/processor.src.js). Generic; only new params need new cases.
24. Web visualization: `WaveformVisualizer` only uses `s.emergence` scalar, no snapshot/mode — [WaveformVisualizer.tsx:64, 82-85](apps/web/src/components/WaveformVisualizer.tsx). WASM exposes no snapshot getter ([wasm.rs](crates/core/src/wasm.rs)); apps/web README: "Full visual modes remain terminal-only" ([apps/web/README.md:~166-169](apps/web/README.md)).

Docs
25. README feature bullets (lines 33-37, 46-51, 144) — [README.md](README.md); apps/web README "Emergence spawn mode values mirror Rust exactly: Canon 0, Penrose 1, and Fuxian 2" — [apps/web/README.md:159-161](apps/web/README.md); JOURNEY essay mentions modes (lines 23, 183) — [JOURNEY_THROUGH_THE_COSMOS.md](JOURNEY_THROUGH_THE_COSMOS.md).

**Historical precedent**: Fuxian was added in four commits: `57a00d4 feat(engine): add fuxian spawn mode` (emergence.rs, engine.rs, wasm.rs), `5b5ddd6 feat(web): expose fuxian spawn controls` (App.tsx, localPresets.ts, params.ts, sequences.ts, useMicroTube.ts, copy.ts, processor.src.js), `a05d7cf docs: document fuxian emergence mode` (JOURNEY, README, web README, controls.md, emergence.md, presets-and-sequences.md, consonance.md), `40c610e fix(engine): make fuxian moves audible` — [git log](.git)

### Inferences
- Appending `SeaOrgan = 3` (not reordering) keeps all existing presets, sequences, and wire values valid.
- The CLI `g` cycle gets a 4th stop automatically through `toggled()`; no new keybinding required unless a separate Sea-Organ-specific control (e.g. "sea state"/swell) is added — the active-param list (`ActiveParam` enum incl. `Gravity`, [app.rs:261-330](crates/cli/src/app.rs)) would then need a new row.
- Gravity could be repurposed per-mode (e.g. as "sea state" / wave energy in Sea Organ mode) to avoid adding a new parameter everywhere; that would require mode-aware labels in CLI `ui.rs` and web `copy.ts` hints.

### Gaps
- Did not read every line of `ui.rs` (2147 lines) / `app.rs` param-row code; exact line of `ActiveParam` row list for adding a new slider beyond `ui.rs:728` not fully mapped.

## 3. What oscillator/timbre/noise facilities exist that a Sea Organ voice could reuse?

### Takeaway
Reusable pieces: the per-voice additive oscillator (partials 2..6 with `Timbre` weight vectors — Organ is a 1/2^n series, Flute is odd-ish 3rd/5th partials), five stateful noise colours in `NoiseGen` (Brown is literally labelled "surf"), `mist_gain`, `soft_clip` (tanh above |0.9|), 50 ms one-pole parameter smoothing, the one-pole low-pass helper inside the HRTF code, and the Fuxian fixed-capacity candidate pool pattern. There is no band-pass/resonator filter, no per-voice noise, and no LFO facility.

### Cited Findings
- `Timbre { Organ=0, Flute=1, Bell=2, Saw=3 }` with `weights()` over partials 2..=6: Organ `[0.5,0.25,0.125,0.0625,0.03125]`, Flute `[0.0,0.5,0.0,0.125,0.0]`, Bell `[1.0,0.0,0.5,0.0,0.25]`, Saw `[0.5,0.333,0.25,0.2,0.166]` — [synth.rs:6-53](crates/core/src/synth.rs)
- Emergence voices use the carrier's smoothed timbre weights and `harmonics` amount (not a voice-specific timbre) — [engine.rs:180-183, 196-202](crates/core/src/engine.rs), [emergence.rs:459-474](crates/core/src/emergence.rs)
- `MistType { Pink, White, Brown, Blue, Velvet }` with textures "warm", "air", "surf", "glass", "sparks" — [synth.rs:55-106](crates/core/src/synth.rs); `mist_gain` per colour 0.30/0.24/0.22/0.20/0.26 — [synth.rs:108-118](crates/core/src/synth.rs)
- `NoiseGen` (xorshift64 RNG; 7-row Voss pink; brown leaky integrator `state*0.996 + white*0.035`, ×1.4; blue = first/second difference; velvet = sparse ±1 impulses with 0.88 decay) — [synth.rs:130-201](crates/core/src/synth.rs). Methods other than `sample()` are private; `NoiseGen` is a single global instance owned by `Engine` — [engine.rs:81, 110](crates/core/src/engine.rs)
- Mist is mono, mixed equally to L/R after emergence — [engine.rs:222-228](crates/core/src/engine.rs)
- `soft_clip(x)`: passthrough below |0.9|, else `tanh` — applied to final L/R — [synth.rs:120-128](crates/core/src/synth.rs), [engine.rs:230-231](crates/core/src/engine.rs)
- Parameter smoothing: `smooth_alpha = 1 − exp(−1/(sr·0.05))` (~50 ms), applied to every continuous param each sample — [engine.rs:90-91, 166-183](crates/core/src/engine.rs)
- `one_pole_alpha(cutoff, sr)` and `shape_far_ear` low-pass logic (private) — [emergence.rs:232-247](crates/core/src/emergence.rs)
- Shepard engine: 7 octave partials with a raised-cosine/gaussian-like `envelope(log_offset)` window, golden-ratio phase step `PHASE_STEP=0.618...`, OUTPUT_GAIN 0.30 — [shepard.rs:27-40, 115-160](crates/core/src/shepard.rs)
- Fuxian voice-leading pool: fixed-size stack array + weighted roulette selection with gravity bias — [emergence.rs:595-660, 704-835](crates/core/src/emergence.rs)
- Knowledge wiki already has articles on `raised-cosine`, `mist-textures`, `timbre-design`, `phase-accumulator` — [content.rs:19-68](crates/cli/src/knowledge/content.rs)

### Inferences
- A pipe-like "breathy" tone could combine a voice's sine/Flute-like partials with per-voice filtered noise; this needs a per-voice RNG/noise state (cheap: copy the xorshift + brown/one-pole pattern into `Voice` or a fixed array), since the global `NoiseGen` is single-instance and mono.
- Brown noise ("surf") at engine level is an obvious ambient sea bed; Sea Organ mode could optionally drive or correlate with mist, but mist is an independent user param — coupling them would be a design decision.
- Wave-driven swells could reuse a slow phase accumulator (as in Shepard/carrier) as a low-frequency "sea" oscillator with randomised period, evaluated per sample without allocation.

### Gaps
- No existing band-pass/resonant filter or formant facility found in core; would have to be written.

## 4. Constraints: max voices, sample rate, real-time safety, binaural carrier relation, tests

### Takeaway
Hard cap of 12 voices with oldest-by-remaining-life eviction; sample rate is injected at construction (CLI prefers 48 kHz, web uses the AudioWorklet `sampleRate`); the per-sample path is allocation-free in steady state (Vec pre-sized to 12, Fuxian pools on stack) but `snapshot()` allocates and is called from the CLI audio callback ~30×/s. Emergence voice pitch = `current_base` (the LEFT binaural carrier frequency) × ratio; the voices themselves are not binaurally detuned — their stereo image comes from the HRTF panner only. Tests: `cargo test` (25 core tests pass), web has no test script — only `tsc` via `npm run build`.

### Cited Findings
- `MAX_VOICES = 12` — [emergence.rs:22](crates/core/src/emergence.rs); eviction — [emergence.rs:501-511](crates/core/src/emergence.rs); README: "Emergence adds up to 12 generative voices with consonance-weighted lifetimes" — [README.md:33](README.md)
- Sample rate: `Engine::new(sample_rate)` → `EmergenceEngine::new(sample_rate)` — [engine.rs:89-112](crates/core/src/engine.rs); CLI picks 48 kHz if supported, else device max — [audio.rs:46-55](crates/cli/src/audio.rs); worklet constructs `new WasmEngine(sampleRate, 2048)` — [processor.src.js:44](apps/web/worklet/processor.src.js). HRTF ITD and one-pole coefficients are computed from `sample_rate` each call — [emergence.rs:196, 239](crates/core/src/emergence.rs). Tests use 100 Hz and 48 kHz sample rates — [emergence.rs:908, 1090](crates/core/src/emergence.rs)
- Allocation behaviour: `voices: Vec::with_capacity(MAX_VOICES)` [emergence.rs:372](crates/core/src/emergence.rs); `try_spawn` evicts before push so length stays ≤12 (no realloc); `retain` does not allocate; Fuxian pools are fixed arrays `[EMPTY_FUXIAN_CANDIDATE; 32]` [emergence.rs:596, 608](crates/core/src/emergence.rs); `canon_pattern` is a `Vec` allocated once in `new()` [emergence.rs:379](crates/core/src/emergence.rs). `Voice` is ~2 KB due to the inline `[f64;256]` HRTF delay (copied on push).
- `snapshot()` builds `Vec<VoiceInfo>` and `recent_tiles` Vec — [emergence.rs:669-690](crates/core/src/emergence.rs); CLI audio callback calls `engine.emergence_snapshot()` every `sample_rate/30` frames under `try_lock` — [audio.rs:57-91](crates/cli/src/audio.rs) (an existing allocation on the audio thread).
- WASM render path is allocation-free in normal playback: Wasm-owned buffers + cached Float32Array views; growth only on larger blocks — [wasm.rs:1-7, 332-353](crates/core/src/wasm.rs), [processor.src.js:9-12, 75-101](apps/web/worklet/processor.src.js)
- Carrier: `freq_l = current_base`, `freq_r = current_base + current_beat` — [engine.rs:185-186](crates/core/src/engine.rs); emergence receives `self.current_base` — [engine.rs:196-198](crates/core/src/engine.rs); voice freq = `base_freq * interval_from_root` — [emergence.rs:455](crates/core/src/emergence.rs). Shepard is mono "so it doesn't disturb the binaural phase difference" — [engine.rs:210-220](crates/core/src/engine.rs). Voice ratios folded to [0.25,4] ⇒ 55–880 Hz at the 220 Hz default.
- Mix order: carrier×vol + emergence×vol + shepard×vol + mist, then soft clip — [engine.rs:188-231](crates/core/src/engine.rs). Emergence bus peak bounded roughly by energy cap 2.0 × voice scale 0.15 ⇒ ≈0.3 per channel before HRTF gains.
- Emergence is reset (voices cleared to root) when switched off — [engine.rs:205-208](crates/core/src/engine.rs). Changing spawn mode does not reset voices or state — [engine.rs:161-164](crates/core/src/engine.rs), [emergence.rs:401-403](crates/core/src/emergence.rs).
- Commands: `cargo test` runs Rust workspace tests; `npm run wasm:build` builds Wasm + worklet; `npm run dev`; `npm run build` (release wasm + `tsc && vite build`) — [README.md:151-166](README.md), [package.json scripts](package.json), [apps/web/package.json](apps/web/package.json) (scripts: dev, build=`tsc && vite build`, preview; no test runner). Wasm build: `wasm-pack build crates/core --target web ... -- --features wasm && node apps/web/scripts/build-worklet.mjs` — [package.json](package.json). Core crate is `rlib`+`cdylib`, `wasm` feature gates wasm-bindgen — [crates/core/Cargo.toml](crates/core/Cargo.toml). Vercel build installs rustup/wasm-pack then `npm run build` — [scripts/vercel-build.sh](scripts/vercel-build.sh), [DEPLOYMENT.md:33-42](DEPLOYMENT.md).
- Observed: `cargo test -p microtube-core` → "25 passed; 0 failed" (run 2026-10-19 in this repo).
- Existing emergence tests: consonance inversion/octave equivalence, strongest-parent spawn, reset, Fuxian pool/parallels/leap/gravity, fold, HRTF balance/delay/boundedness/reset — [emergence.rs:892-1137](crates/core/src/emergence.rs). Shepard has a "output bounded at full intensity" pattern test — [shepard.rs:201](crates/core/src/shepard.rs). CLI tests for spawn toggle and local preset JSON — [app.rs:1295-1307](crates/cli/src/app.rs), [local_presets.rs:~280-313](crates/cli/src/local_presets.rs).
- No `.github` CI directory exists (checked `ls .github`: absent).

### Inferences
- New Sea Organ state must be fixed-size (arrays / scalars) to keep `process()` allocation-free; per-voice state added to `Voice` grows the struct already dominated by the 2 KB delay buffer.
- Good test candidates mirroring existing style: deterministic spawn ratios given a seeded engine; output bounded at intensity 1 for N seconds at 48 kHz; voices ≤ MAX_VOICES; mode cycle order; `from_u32(3)` round trip; local-preset parse of the new label.

### Gaps
- Did not measure CPU cost; no benchmark harness in the repo.

## 5. Docs describing emergence semantics

### Takeaway
Emergence semantics are documented in the CLI knowledge assets (`emergence.md`, `controls.md`, `consonance.md`, glossary), the root README, the web README, and the JOURNEY essay; DEPLOYMENT.md only covers the Wasm build.

### Cited Findings
- `emergence.md`: root voice, voices branch from strongest live voice, per-voice fields, HRTF-style spatial approximation (no HRIR files), Canon/Penrose/Fuxian descriptions, Emergence view description (nodes, lines, status line, Penrose worm ribbon), usage "Turn emergence on with `e`. Start around 30 to 50 percent" — [emergence.md:1-78](crates/cli/src/knowledge_assets/microtube/emergence.md)
- `controls.md:31` gravity description; `controls.md:50` "`g` cycles the emergence spawn mode: Canon, Penrose, Fuxian." — [controls.md](crates/cli/src/knowledge_assets/microtube/controls.md)
- `consonance.md:75-77` Darwinian filter / Fuxian — [consonance.md](crates/cli/src/knowledge_assets/wiki/consonance.md)
- Glossary entries for consonance score ("Defined in src/emergence.rs"), fibonacci word, conway worm, canon — [glossary.toml:106-124, 229, 269](crates/cli/src/knowledge_assets/glossary.toml)
- README feature summary: 12 voices, consonance-weighted lifetimes, Canon/Penrose/Fuxian; Fuxian "leap recovery, no repeated fifth/octave parallels, and a gravity control" — [README.md:10, 33-37, 46-51](README.md)
- JOURNEY essay: "a generative emergence engine that spawns up to twelve voices at harmonic ratios with consonance-weighted lifetimes"; sequence schema discrete fields snap, continuous lerp, 50 ms smoother absorbs boundary; Fuxian noted as later addition — [JOURNEY_THROUGH_THE_COSMOS.md:23, 153, 183](JOURNEY_THROUGH_THE_COSMOS.md)
- Web README: spawn mode integers mirror Rust; gravity default 0.5 preserved in local presets; visual modes terminal-only — [apps/web/README.md:105-106, 159-169](apps/web/README.md)
- No reference to "sea organ" or "Zadar" exists anywhere in the codebase (grep over crates).

### Inferences
- A new mode would follow the Fuxian doc commit pattern: add a "## Sea Organ mode" section to emergence.md, update controls.md line 50, README bullets, web README enum mapping, and possibly a new wiki article (requires `include_str!` entry in content.rs).

### Gaps
- None significant for docs.
