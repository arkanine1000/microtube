# Let the Sea Play MicroTube's Emergence

MicroTube can get very close to the Zadar Sea Organ with a fourth spawn mode, `SpawnMode::SeaOrgan = 3`. The core change is to stop spawning voices on a fixed timer. Instead, a simulated sea drives seven per-segment water-column resonators, and those resonators decide when pipe voices speak and how loud they are. The real instrument is well documented on three points. It has **35 stopped flue pipes in 7 segments of 5**. Odd segments are tuned to **G major (D–G–d–g–h)** and even segments to **C major with an added sixth (C–D–c–e–a)**, all within roughly **65–250 Hz**. Each segment's air chamber is a damped oscillator with a resonance between **0.215 and 0.528 Hz** (periods of 1.9–4.7 s). Within that frame the sea decides which pipes sound, when, and how loudly ([Krvavica et al. 2019](https://arxiv.org/pdf/1903.09314)). I fed Krvavica's published segment resonances (fr, ζ) into a plain linear resonator and drove it with a JONSWAP sea spectrum. It reproduced five of the six segment-ranking observations in the paper, including segment 1 falling silent in small seas and waking up in jugo storms. That makes a cheap control-rate sea model, rather than a full hydraulic simulation, a defensible core for the mode. Most of the existing emergence machinery can be reused: the 12-voice cap, HRTF panning, energy bound and additive partials. Three things are new: a sea driver, an envelope driven by that sea instead of the symmetric sin² bell, and a little breath noise for each voice. Much of the sonic detail is not verified for Zadar and is labelled as inference below. That covers the pressure-dependent pitch bend, the chiff, the overblowing and how air going back into the pipes sounds, as well as the wake rates. No one has published spectra or SPL measurements of the organ.

## Seven chambers, two chords: what is actually known about the instrument

The organ is a stepped stone seawall of about **70–75 m** on Zadar's riva. Architect Nikola Bašić designed it and it opened on **15 April 2005** ([Wikipedia](https://en.wikipedia.org/wiki/Sea_organ)). Ivica Stamać did the acoustic and tuning design, and the Heferer organ workshop calculated the flues. The workshop chose stainless steel over polyethylene for the sounding part because temperature changes would have detuned the labium gap ([Architectuul](https://architectuul.com/architecture/the-sea-organ)). Each pipe system has three parts in series:

- a submerged horizontal PE inlet, D1 ≈ 0.3 m;
- a sloped PE riser of **50–125 mm** diameter, inclined **20–55°**;
- a short horizontal stainless "acoustical" pipe under the walkway, "closed at the end but has a small orifice at the beginning" ([Krvavica et al.](https://arxiv.org/pdf/1903.09314)).

Sound leaves through holes in the top step and slits in the vertical step faces. The five pipes in each section are spaced **1.5 m** apart ([OddMusic, reproducing project text](http://www.oddmusic.com/gallery/om24550.html)).

The sound mechanism is an oscillating water column. The sea surface forces the water up the sloped pipe, and the water compresses the trapped air. That air is pushed through a labium into a stopped resonator. In Krvavica's words, "nature itself determines the duration and intensity of each note, but the arrangement of the pipes… governs the resulting melody". Waves, tides and passing boats all drive it ([Krvavica et al.](https://arxiv.org/pdf/1903.09314)). Stamać picked **stopped flue pipes** for their rounded timbre ([AES 2007 abstract](https://aes.org/e-lib/browse.cfm?elib=14132)). He placed them in the great and small octaves (**65–250 Hz**), where "flue pipes emanate their noblest sonority" ([Stamać 2005 via croatia.org](http://www.croatia.org/crown/articles/9359/1/nikola-baiae-author-of-the-zadar-sea-organ.html)). The chords are said to follow the "musical matrix of the klapa song" ([Architectuul](https://architectuul.com/architecture/the-sea-organ)). Several popular claims are wrong. There are not seven different chords. The chords do not change with wave height. The tuning is not random. What is random is which pipes speak.

The hydraulic paper gives the numbers that matter most for an emulator. Krvavica et al. estimated the geometry from design drawings and model each segment's middle pipe as a damped single-degree-of-freedom oscillator ([Krvavica et al., Tables 1–2](https://arxiv.org/pdf/1903.09314)):

| Segment | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|
| Chord (verified) | G | C6 | G | C6 | G | C6 | G |
| Crest height hc (m asl) | 0.30 | 0.55 | 0.70 | 0.85 | 1.00 | 1.16 | 1.30 |
| Slope φ (°) | 20 | 25 | 30 | 25 | 30 | 45 | 55 |
| Resonance fr (Hz) | 0.215 | 0.263 | 0.308 | 0.275 | 0.323 | 0.449 | 0.528 |
| Resonant period (s) | 4.65 | 3.80 | 3.25 | 3.64 | 3.10 | 2.23 | 1.89 |
| Damping ζ | 0.115 | 0.101 | 0.103 | 0.087 | 0.116 | 0.137 | 0.140 |

The pitch mapping below is inference, but it holds together well. The note names follow German/Helmholtz usage (h = B, capitals = great octave). Read that way in 12-TET at A4 = 440 Hz, the two chords contain **nine distinct pitches**: C2 65.41, D2 73.42, G2 98.00, C3 130.81, D3 146.83, E3 164.81, G3 196.00, A3 220.00 and B3 246.94 Hz. That matches both the "nine tones" in Stamać's AES abstract and the stated 65–250 Hz band exactly. **D2 occurs in every section** and acts as a shared drone. G major and C6 share G, C and D. Where sections overlap, the main colour comes from a B–C semitone across octaves. Several details are still unknown: the temperament and reference pitch, the order of the five pipes within a section, the real acoustic resonator lengths, and the labium geometry. Krvavica's L3 = 0.3 m is a hydraulic placeholder. A stopped pipe sounding C2 would need about 1.3 m ([Krvavica et al.](https://arxiv.org/pdf/1903.09314)). The notes also disagree on whether the modelled orifice area is 112 mm² or a garbled 12 mm². The 12.5 m flume calibration's best fit was A0 = 12 mm² at scale.

## A short-period Adriatic sea makes 2–5 s breaths in 15–40 s sets

No wave buoy covers the Zadar riva. The best local anchors are Krvavica's two design sea states, generated from the Adriatic Tabain spectrum (a JONSWAP variant):

- light NW wind (maestral-type): **Hs = 0.4 m, fp = 0.34 Hz (Tp ≈ 2.9 s)**;
- strong SE jugo: **Hs = 1.0 m, fp = 0.22 Hz (Tp ≈ 4.5 s)**;
- both with still water at +0.35 m asl ([Krvavica et al.](https://arxiv.org/abs/1903.09314)).

The Adriatic is fetch-limited. Jugo blows along the basin's long axis and builds much bigger waves than bura does at the same wind speed ([Frontiers in Climate 2024](https://www.frontiersin.org/articles/10.3389/fclim.2024.1409237/full)). Bura is offshore at the riva. So the inference is that jugo and maestral drive the organ hardest, and claims that it is "loud in bura" are doubtful.

Each segment responds most strongly near its own resonance. In the small sea, segments 3 and 5 had the highest air pressure and segments 1 and 6 the lowest. In the jugo case, segment 4 peaked and the last two were weakest. The authors add from experience on site that "the sound from the first segment is quieter than the others or even non-existing during small waves, but for higher wave heights… can be heard quite loudly" ([Krvavica et al.](https://arxiv.org/abs/1903.09314)). Visitors say the organ is "almost inaudible" on calm days and louder and deeper with bigger waves ([Sail Croatia](https://sail-croatia.com/set-sail/the-zadar-sea-organ); [Absolute Croatia](https://www.absolute-croatia.com/zadar/attractions/sea-organ)). "Deeper" fits the physics. Long jugo waves resonate segment 1, which is a G-major segment, so its low D2 and G2 come forward. The pitches stay fixed; only the set of sounding pipes changes. A listener anywhere on the steps hears about **5–7 tuned pipes** at once ([OddMusic](http://www.oddmusic.com/gallery/om24550.html)). That fits comfortably inside MicroTube's 12-voice cap.

I checked how far a linear model gets. The test drove each segment's Table-2 resonator, as gain |H(f)| = 1/√((1−(f/fr)²)² + (2ζf/fr)²), with a JONSWAP spectrum (γ = 3.3). This is my own calculation, not published. The table shows RMS water-column gain relative to the surface:

| Sea state | Seg 1 | Seg 2 | Seg 3 | Seg 4 | Seg 5 | Seg 6 | Seg 7 | Rank (loudest first) |
|---|---|---|---|---|---|---|---|---|
| fp 0.34 Hz (maestral) | 0.71 | 1.63 | 2.76 | 2.07 | **2.86** | 2.34 | 2.00 | 5, 3, 6, 4, 7, 2, **1** |
| fp 0.22 Hz (jugo) | 3.02 | 3.04 | 2.50 | **3.07** | 2.27 | 1.61 | 1.43 | **4**, 2, 1, 3, 5, **6, 7** |

The linear filter gets these right: 3 and 5 on top in maestral, 1 at the bottom in maestral, 4 on top in jugo, and 6–7 at the bottom in jugo. It also makes segment 1 jump from last to near the top. It misses one result: it puts segment 6 third in maestral, where the paper has it among the weakest. The likely causes are crest height, reflection and nonlinear losses. A fixed trim of about **0.6 on segment 6** fixes this without breaking the jugo ranking.

The absolute levels are also plausible. With Hs = 0.4 m (surface RMS ≈ 0.1 m), the loudest segment moves about 0.29 m RMS. Krvavica reports internal water-level amplitudes of about 15–30 cm ([Krvavica et al.](https://arxiv.org/abs/1903.09314)). For jugo the linear model gives about 0.77 m RMS against a reported 50–70 cm. That overshoot is what you would expect from the quadratic loss term β·Q|Q|, and it argues for damping that grows with amplitude.

The research notes disagree on the absolute air pressures read off Krvavica's Figure 9. One reading gives hundreds of Pa (axis to about 800 Pa) for maestral and about 2–2.5 kPa for jugo. Another gives 1.2–1.4 kPa and 3.5–4 kPa. A third found no absolute values. The plan below therefore works in normalised water-column displacement, not pascals.

Wave grouping, wakes and tides fill in the temporal texture. The grouping figures are general wind-sea theory, not Zadar measurements. A random-phase JONSWAP sum produces groups naturally ([Krvavica et al.](https://arxiv.org/abs/1903.09314)). With Tp ≈ 3 s that means sets of about 5–6 s every 15–25 s; at Tp ≈ 4.5 s, sets of 7–9 s every 25–40 s. Individual wave heights follow a Rayleigh distribution ([HR Wallingford SR39](https://eprints.hrwallingford.com/80/1/SR39.pdf)).

Passing boats are a confirmed forcing. Car ferries now sail from Gaženica, about 4 km away ([In Your Pocket](https://inyourpocket.com/zadar/arriving)). Passenger catamarans and excursion boats still use the old-town port. In general wake physics, packets arrive as tens of seconds of longer-period waves. Fast-ferry leading waves can reach 10–15 s periods ([Kurennoy & Soomere](https://coruna.coastdyn.ru/iccal/pub/kurennoy.pdf)). No count of boats passing the riva exists, so the suggested rate of about **4–10 wakes/hour on summer days and 1–3 in winter** is an estimate.

Tides at Zadar are microtidal and mixed, mainly diurnal. K1 = 11.40 cm and M2 = 5.76 cm, and the daily range is about **8–32 cm** ([HHI Mijene 2026](https://www.hhi.hr/Portals/0/adam/HHI/pMB-Mvl4nUWpnu4mb5ShNw/PdfView/Mijene%202026%20za%20web.pdf)). The step crests rise from 0.30 to 1.30 m asl. That points to a plausible but unmeasured modulation of segment-1 activity over a 12–25 hour cycle. Over the length of a listening session this effect is negligible.

The obliquity of arriving waves gives the "rolling chord", and that too is inference. Deep-water crests move at about 4.7 m/s at T = 3 s. A crest arriving at 30–60° to the wall sweeps the 70 m length in roughly **5–13 s**. That staggers the segment entries, and the sweep direction should reverse between NW maestral and SE jugo. The wall's exact orientation and which end faces NW were not found.

## Uncoupled pipe voices beat full flue physics on cost and control

There are two ways to synthesise the pipes. The first is a coupled physical model. STK's Flute is the classic: jet delay, cubic jet nonlinearity, lossy bore delay, with noiseGain 0.15 and jetRatio 0.32 ([STK Flute.cpp](https://raw.githubusercontent.com/thestk/stk/master/src/Flute.cpp)). The more rigorous "toy model" uses U_j = √(2P/ρ), a convective delay τ ≈ W/(0.5·U_j), a tanh nonlinearity and one or two resonator modes. In that model the oscillation threshold, the slight pitch rise with pressure, the register jumps and their hysteresis all emerge on their own ([arXiv 1207.7136](https://ar5iv.arxiv.org/html/1207.7136)).

The second is an "uncoupled" source–filter design. Commercial organ modelling uses this: a free-running generator excites a resonator, and coupling effects are added empirically ([Pykett](https://colinpykett.org.uk/complete-physical-modelling.htm)). Pykett also gives useful voicing facts. Pitch can drift "around a semitone (around 6%)" as pressure falls, attack transients briefly add partials, and a symmetric jet gives mostly odd harmonics.

For MicroTube the uncoupled route is the better fit. Pitch has to sit exactly on ratios to the binaural carrier, and the emergence voice is already an additive oscillator. The fit goes further. The existing `Timbre::Flute` weights over partials 2..6 are `[0.0, 0.5, 0.0, 0.125, 0.0]` ([synth.rs:48](../crates/core/src/synth.rs)). That is only the 3rd and 5th partials, which is exactly the odd-harmonic spectrum of a stopped pipe.

The Zadar pipes are short tubes with small orifices, closer to whistles than to voiced organ mouths. Nobody has published their spectra, so any timbre choice is a calibration target. The best listening reference is the Global Sound Movement's multichannel recordings and octave-by-octave samples of the organ ([Global Sound Movement](https://globalsoundmovement.com/?p=2816)). No real-time sea-organ synthesiser turned up in NIME, ICMC or DAFx papers, but those searches were limited.

The sea driver also has a validated model. Krvavica's Eq. 25 gives three states per pipe: water-column length l2, flow Q and air overpressure Δp. Orifice flow is C_d·A0·√(2|Δp|ρ_air), with C_d calibrated at 0.6–0.7 ([arXiv 1903.09314](https://ar5iv.labs.arxiv.org/html/1903.09314)). At about 1 kHz control rate, 7 × 3 states cost almost nothing. The square root is stiff near Δp = 0 and needs regularising, for example √(|Δp| + ε). The sign convention in the extracted equation should be checked against the PDF.

Phase 1 does not need that model. As the table above shows, a second-order resonator per segment, fed by the random-phase surface, already gets the segment behaviour right. The surface is synthesised as η(t) = Σ aᵢ cos(ωᵢt − kᵢx·sinθ + φᵢ) with aᵢ = √(2S(ωᵢ)Δω). Per-segment phase offsets kᵢx produce the oblique sweep exactly, with no delay lines.

## A phased build that reuses emergence.rs and touches 25 known places

The integration point is clear from the code. `try_spawn` dispatches on `match self.spawn_mode` to compute `final_ratio` ([emergence.rs:540](../crates/core/src/emergence.rs)). Lifetime is then set generically as `4.0 + consonance·8.0` seconds with ±20 % jitter. Spawning runs on a mode-independent timer: `spawn_interval = 3.0 / (0.5 + 1.5·intensity)`. Every voice gets a symmetric `sin(πt)²` envelope and a fixed random pan in ±0.7. The voice gain is ×0.15 and the bus is scaled down when Σ amplitude exceeds 2.0 ([emergence.rs:416-577](../crates/core/src/emergence.rs)).

A Sea Organ that only supplied a ratio would sound like Canon with different pitches. Three paths have to change for this mode: timing, envelope and pan. Everything else stays: the HRTF spatialiser, the energy bound, the 12-voice cap, the additive partials and the allocation-free per-sample loop.

### Parameter mappings and starting values

Pitch should stay relative to `base_freq`. Voice frequency is `base_freq × interval_from_root`, and the carrier relationship is the point of MicroTube. All the ratios below are measured against d (D3), so a base of about 146.8 Hz reproduces the inferred Zadar register exactly. At the default 220 Hz the instrument is transposed up a fifth, and every ratio sits inside `fold_voice_ratio`'s [0.25, 4] window.

The table uses just ratios, which keep the consonance score high. The real temperament is unknown. 12-TET would differ by at most about 14 cents, on the B: 5/3 against 1.682.

| Mapping | Value | Basis |
|---|---|---|
| G segments (1,3,5,7): D, G, d, g, h | ratios 1/2, 2/3, 1, 4/3, 5/3 | Chord verified; ratios and register inferred |
| C6 segments (2,4,6): C, D, c, e, a | ratios 4/9, 1/2, 8/9, 9/8, 3/2 | Chord verified; ratios inferred |
| Segment fr, ζ | Table 2 values above | Verified (estimated geometry) |
| Segment gain trim | 1.0 except seg 6 ≈ 0.6 | Calibrated against paper rankings |
| Emergence slider `intensity` → Hs | Hs = 0.05 + 1.15·intensity² m (0.5 → 0.34 m, 1.0 → 1.2 m) | Anchored to 0.4 m and 1.0 m cases |
| `gravity` → Tp (repurposed in this mode) | Tp = 2.5 + 2.0·gravity s (0.2 → 2.9 s maestral, 1.0 → 4.5 s jugo) | Anchored; interpolation inferred |
| Wave synthesis | 32 JONSWAP components in 0.5–3·fp, γ = 3.3, σ 0.07/0.09, random phases | Standard method used by Krvavica |
| Wave direction | θ = ±45°, sign flips at Tp > 3.7 s (jugo) | Inference; wall orientation unknown |
| Speak threshold (water-column displacement) | on at 0.20 m, off at 0.14 m (hysteresis) | Calibration: seg 1 near-silent at Hs 0.4, active at 1.0 |
| Loudness | A = smoothstep((l − 0.20)/0.60), outflow (rising) phase only | Inference from OWC physics |
| Overblow to 3rd partial | above 1.0 m, back below 0.85 m, 30 ms crossfade | Inference; storm only |
| Pitch vs pressure | +0.5 % to +2.5 % across speaking range; −2 % onset scoop gliding up over 80 ms | Pykett range scaled down; inference |
| Partial-3 weight | 0.15 → 0.5 as A rises; partial 5 at 0.125·A | Flute weights; brightening inferred |
| Breath noise | per-voice SVF bandpass at f, Q ≈ 8, level ∝ √(drive); starts at 0.7× threshold | Inference (STK noiseGain 0.15 as ceiling) |
| Chiff | 40 ms noise burst bandpassed at 3f on each onset | Pykett transient description; inference |
| Pipes per crossing | 1–3 of the segment's 5, weighted, fixed ±20 % per-pipe threshold jitter (seeded) | "Random which pipes sound" verified; method inferred |
| Wakes | Poisson; real-time rate 6/h, optional compressed 1 per 3 min; packet 8 waves, H 0.4 m, T 3 s (leading 6 s), 40 s Gaussian envelope | Forcing verified; numbers inferred |
| Spatial | segment centres x = (k−4)·10.5 m, pipes 1.5 m apart; pan = clamp(atan(x/4 m)/(π/2), ±0.85); gain 4/max(r,4) | 1.5 m spacing verified; listener model inferred |

### Phase 0: plumbing (about one day, mirrors the Fuxian commits)

Phase 0 adds the variant end-to-end with a stub dispatch arm, following the four-commit pattern Fuxian used (`57a00d4`, `5b5ddd6`, `a05d7cf`, `40c610e`). Append `SeaOrgan = 3`; do not reorder, so existing presets and wire values stay valid. The changes are:

- **Core.** In `crates/core/src/emergence.rs`: the enum, `label()` returning `"sea_organ"`, `toggled()` and `from_u32` (lines 249-287).
- **CLI presets.** In `crates/cli/src/local_presets.rs:249-256`, `parse_spawn_mode` must accept `"sea_organ" | "SeaOrgan"`. Without that, presets saved in the new mode fail to load.
- **CLI tests and UI.** Extend the cycle-order test at `crates/cli/src/app.rs:1295-1307`. Update the help text in `crates/cli/src/ui.rs` and the Penrose-only branch in `crates/cli/src/visualization.rs:644-680`.
- **Web.** In `apps/web`:
  - `params.ts`: the `SpawnMode = 0 | 1 | 2 | 3` type;
  - `localPresets.ts:32`: `SPAWN_MODES = [0,1,2,3]`, or saved presets are rejected;
  - `copy.ts`: change `spawnModes` from `Trio` to `Quad`, with EN "Sea Organ" and HR "Morske orgulje", plus mode-aware gravity hints at lines 243-245 and 450-452;
  - `sequences.ts`: the legend;
  - check the four-button `Segmented` layout on mobile.
- **Docs.** `emergence.md`, `controls.md:50`, both READMEs, and optionally a new wiki article registered in `knowledge/content.rs`.

The worklet's `set_spawn_mode` path is already generic and needs no change ([wasm.rs:137-139](../crates/core/src/wasm.rs)).

### Phase 1: the sea driver and driven voices (the core of the sound)

Phase 1 creates `crates/core/src/sea_organ.rs`, a sibling of `penrose.rs`. It holds `SeaOrganState`, all of it fixed-size:

- `[WaveComponent; 32]` (ω, k, amplitude, phase);
- `[SegmentResonator; 7]` (l, dl/dt, fr, ζ, trim, on/off latch);
- a wake state;
- its own xorshift RNG;
- a control-tick counter.

It is advanced every **32 samples**, which is 1.5 kHz at 48 kHz and far above the 0.2–3 Hz content. It uses semi-implicit Euler with linear interpolation between ticks.

`EmergenceEngine` gains a `sea: SeaOrganState` field, initialised in `new()` and cleared in `reset()`. In `process()`, while the mode is `SeaOrgan`, the periodic `spawn_timer` is bypassed. Each control tick instead checks for upward threshold crossings and calls a new `spawn_pipe(segment, pipe)`. This skips both the parent search and the consonance-weighted lifetime, because Zadar pitches are fixed by geometry, not by voice leading.

`Voice` gains these small fields:

- `pipe: Option<u8>` (0..34);
- `register: u8` (fundamental or overblown);
- per-voice noise state: a `u32` RNG plus two SVF integrators;
- `onset_age`.

For a driven voice, `amplitude = speak(l_seg) · trim · distance_gain`. The bell envelope is ignored. The voice dies when drive stays below the off-threshold for 250 ms, so `lifetime` becomes a 20 s safety cap. Three more changes complete the phase. Eviction at the cap should drop the lowest current amplitude, not the least remaining life. `try_spawn`'s "always maintain a root" and the root push in `reset()` must be skipped in this mode, because a calm sea should be nearly silent. Sea Organ voices should also use their own Flute-like partial weights rather than the carrier's `harm_weights`.

The tests for this phase follow existing styles:

- a seeded engine at Hs 0.4 / Tp 2.9 ranks segments 3 and 5 above segment 1 over 10 simulated minutes;
- at Hs 1.0 / Tp 4.5, segment 1 is active and 6–7 are lowest;
- at intensity 0.1, no pipe speaks without a wake;
- output stays bounded at intensity 1 for 60 s at 48 kHz;
- `voices.len() ≤ 12`;
- `from_u32(3)` round-trips;
- the CLI local-preset parse accepts the new label.

### Phase 2: physical fidelity

Phase 2 swaps the per-segment proxy for Krvavica's three-state ODE (Eq. 25), behind a feature flag or constant. That gives true pressure, a leaky orifice, compressibility, and asymmetric breathing in which air goes in on the falling phase. Damping should grow with amplitude, in line with the jugo overshoot noted earlier. The phase also adds overblow hysteresis and the Poisson wakes, whose packets arrive obliquely and sweep the wall. It can add a Rayleigh-based diagnostic for "notes per minute", exp(−2(H*/Hs)²), which gives a quick check that each sea state sounds right. Krvavica's simplified model makes the same move from wave time series to note events, for comparison ([Peroli & Krvavica 2019](https://zr.gradri.uniri.hr/zr/en/article/view/7)).

### Phase 3: place and atmosphere

Phase 3 adds the sense of place:

- **Walking the riva.** A listener-position control along x, so moving shifts the crossfade between the G-major and C6 sections.
- **Surf bed.** A low-level internal layer correlated with η(t): brown noise lowpassed at 300–1500 Hz. It is kept separate from the user's mist so user settings are not silently coupled.
- **Cavity colour.** A short comb or allpass "stone cavity" per segment, with 2–15 ms taps. This is unsourced and should be tuned by ear against recordings.
- **CLI ribbon.** A seven-bar swell display fed by a fixed-size `[f32; 7]` added to `EmergenceSnapshot`.
- **Sequence preset.** A "Riva" sequence that ramps gravity through a maestral afternoon into jugo.

An optional wall-clock tide from HHI constants is possible but would be a novelty only, given the 8–32 cm daily range.

### CPU and real-time constraints

All new state must be fixed arrays so that `process()` stays allocation-free. The voice Vec is pre-sized to 12, and `Voice` is already about 2 KB because of its inline 256-sample HRTF delay. The new per-voice fields add under 64 bytes.

The costs are small. The driver evaluates 32 cosines × 7 segments per control tick, about 336k cosines per second, or rotating phasors to avoid `cos` entirely. Each voice adds about 15–20 flops per sample for its SVF and noise on top of the existing sine-plus-partials and HRTF. Twelve voices stay well under 1 % of a desktop core in Rust. The same code compiles to WASM for the AudioWorklet unchanged. These are estimates; the repo has no benchmark harness.

One existing hazard is unchanged: the CLI's `snapshot()` allocates on the audio thread about 30 times a second ([audio.rs:57-91](../crates/cli/src/audio.rs)). The Sea snapshot should add fixed-size fields only.

## Conclusion

The main finding is that the Zadar organ's "personality" depends far more on its slow hydraulics than on its pipe timbre. The tuning is two fixed chords. Seven resonators between 1.9 and 4.7 s, filtering a fetch-limited Adriatic sea, set when each chord sounds, how dense it is, and which register comes forward. That is why a linear model fed only Krvavica's published numbers already reproduces the segment behaviour seen on site. For MicroTube this means fidelity effort should go first into the sea driver and the driven envelope. A physical flue model can wait. The cheap odd-partial voice the engine nearly has already is enough.

The remaining unknowns are calibration questions, not blockers:

- the temperament and the order of pipes within a section;
- whether pipes sound when air is drawn back in;
- real air-pressure amplitudes, where the notes' readings of the same figure disagree by about 2×;
- the wall's orientation;
- wake frequency at the riva;
- the absence of any published spectrum.

The highest-value next step is to obtain Stamać's 2005 and 2007 papers and the Global Sound Movement recordings. They are the only sources likely to settle timbre, onset behaviour and absolute pitch. Until then, every inferred value in the mapping table should stay a named constant in `sea_organ.rs`, so each can be retuned without touching the architecture.
