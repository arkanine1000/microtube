# Real-time DSP Synthesis Techniques for a Wave-Driven Sea Organ (Zadar-style)

Scope: algorithms that are cheap enough for a per-sample Rust engine and a Web Audio AudioWorklet, covering pipe tone, breath noise, pressure-dependent onset and decay, stochastic wave driving, and spatial ambience. Unless a bullet carries a source link, treat it as an inference (these are kept in the "Inferences" subsections).

## 1. Flue pipe / edge-tone / whistle physical models

### Takeaway
The standard real-time flue model has three parts in a loop: a jet delay (convection time across the mouth), a memoryless sigmoid nonlinearity (jet-labium interaction), and a linear resonator (a waveguide delay line or a sum of 1-2 modes). Blowing pressure enters only through the jet velocity U_j = sqrt(2P/rho), which sets the jet delay. That one relationship accounts for the oscillation threshold, the slight rise in pitch with pressure, the jumps to the octave or overblown register (with hysteresis), and the "aeolian" low-pressure regime. STK's Flute is a ready-made, very cheap template: two fractional delay lines, a one-pole filter, a cubic polynomial and a DC blocker per sample.

### Cited Findings
- **STK Flute structure:** STK's Flute is described as a simple waveguide flute model "based on work by Karjalainen, Smith, and Waryznyk", with a jet "polynomial, following Cook". It exposes jet delay (CC 2), noise gain (CC 4), vibrato frequency (CC 11), vibrato gain (CC 1) and breath pressure (CC 128), plus `setJetReflection`/`setEndReflection` in the range [-1, 1] and `startBlowing(amplitude, rate)`/`stopBlowing(rate)`. — [STK Flute class docs](https://ccrma.stanford.edu/software/stk/classstk_1_1Flute.html); [STK Flute.h](https://raw.githubusercontent.com/thestk/stk/master/include/Flute.h)
- **STK Flute per-sample tick:**
  1. `breath = maxPressure * adsr`, plus `noiseGain * noise`, plus `vibratoGain * sin`.
  2. `temp = -filter(boreDelay.lastOut())`, a one-pole lowpass with an inverted reflection.
  3. `pressureDiff = breath - jetReflection * temp`, then `pressureDiff = jetDelay.tick(pressureDiff)`.
  4. `pressureDiff = dcBlock(jetTable(pressureDiff)) + endReflection * temp`, where jetTable is the polynomial.
  5. `out = 0.3 * boreDelay.tick(pressureDiff)`.

  — [STK Flute.h](https://raw.githubusercontent.com/thestk/stk/master/include/Flute.h) (as summarised by the fetch tool; I did not get the exact source text). STK's JetTable computes `x*(x*x - 1)`, clipped to ±1, which is a cubic. This is from my own prior knowledge of STK and was not re-verified in this session.
- **STK Flute default values:** endReflection = 0.5, jetReflection = 0.5, jetRatio = 0.32 (jet delay = 0.32 × bore delay), noiseGain = 0.15, vibratoGain = 0.05, vibrato at 5.925 Hz, filter pole = `0.7 - 0.1*22050/fs`, ADSR times (0.005, 0.01, 0.8, 0.010). In `setFrequency`, `lastFrequency = f*0.66666` and bore delay = `fs/lastFrequency - filter.phaseDelay - 1`, so the bore is deliberately tuned 1.5× long and the target pitch comes from the jet/bore interaction. `noteOn` calls `startBlowing(1.1 + 0.2*amp, 0.02*amp)`. — [STK Flute.cpp](https://raw.githubusercontent.com/thestk/stk/master/src/Flute.cpp)
- **"Toy model" of flute-like instruments (Terrien/Auvray/Fabre et al.):**
  - Convection velocity c_v ≈ 0.5·U_j, so the jet delay is τ = W / c_v ≈ W / (0.5·U_j). W is the mouth height (flue-to-labium distance). Blowing harder raises U_j and shortens τ.
  - The nonlinearity is p(t) = α·tanh[v(t − τ)].
  - The resonator is a modal admittance, Y(ω) = Σ a_k·jω / (ω_k² − ω² + jω·ω_k/Q_k), with at most two modes.
  - Example parameters: one mode with α = 10, a1 = 70, ω1 = 2260 rad/s, Q1 = 50. Two modes with α = 340, a1 = 10, a2 = 30, Q = 100, ω2/ω1 = 1.99–2.05.

  — [Flute-like musical instruments: a toy model investigated through numerical continuation (arXiv 1207.7136)](https://ar5iv.arxiv.org/html/1207.7136)
- **Threshold, regimes and hysteresis in the toy model:**
  - Oscillation exists only over certain ranges of dimensionless delay. For one mode, the equilibrium is unstable for 1.8 < ωτ < 4.1 and 9.1 < ωτ < 9.5, and Hopf bifurcations at those points give the oscillation thresholds.
  - The "standard" regime exists as τ → 0 (high pressure). "Aeolian" regimes appear at low jet velocity, at θ = U_j/(W·f) < 4, with a bell-shaped amplitude.
  - Blowing harder makes the pitch jump to the octave. With ω2/ω1 = 1.99, both registers are stable for 0.1 < ωτ < 0.7, so the model has hysteresis.
  - With exactly harmonic modes (ω2 = 2ω1), a pressure ramp can produce a quasiperiodic, "unstable" regime (0.71 < τ̃ < 0.9).

  — [arXiv 1207.7136](https://ar5iv.arxiv.org/html/1207.7136)
- **Jet-drive model lineage:** Coltman proposed the jet-drive model, in which the labium acts as a flow partition. Verge et al., Fabre & Hirschberg, Meissner and Dequand et al. developed it into a dipole pressure jump across the window. Verge et al. found that the dimensionless amplitude of the fundamental depends, for a given geometry, only on the Strouhal number. — [Time-domain simulation of flute-like instruments: comparison of jet-drive and discrete-vortex models (HAL)](https://hal.archives-ouvertes.fr/hal-01426971); [Verge et al., Sound production in recorderlike instruments (HAL)](https://hal.archives-ouvertes.fr/hal-01105554)
- **Mouth pressure dynamics and thresholds:** A paper on regime-change thresholds in flute-like instruments finds that the dynamics of mouth pressure (for example, the ramp rate) affect the thresholds. Ramp speed therefore shifts where the octave jump and the extinction happen. — [Regime change thresholds in flute-like instruments: influence of the mouth pressure dynamics (arXiv 1403.7487)](https://arxiv.org/pdf/1403.7487). The full text could not be retrieved, so this is based on the title and abstract only.
- **Recorder synthesis lineage:** A recorder synthesiser (Shiraiwa, Kishi, Nakamura) uses a jet oscillation model following Adachi's organ flue pipe simulation, which is in turn based on Fletcher's semi-empirical jet-drive model, coupled to a digital waveguide resonator. — [recorder_asa03.pdf (CCRMA)](https://ccrma.stanford.edu/~hiroko/paper/recorder_asa03.pdf)
- **Pipe-organ voicing facts (Pykett):**
  - A jet of about 35 m/s comes from about 75 mm water gauge (~735 Pa).
  - The disturbance on the jet travels at about half the jet speed.
  - At low pressure for a given cut-up, a pipe can jump to roughly the octave above with a "thin, quavery" sound.
  - Pitch variation with falling pressure before speech stops "can typically approach a semitone (around 6%)".
  - Attack transients include brief 2nd and 4th harmonics on principals. Some flutes produce "chiff" or whistles at the 3rd, 5th or 7th harmonics.
  - Transient partials start anharmonic and phase-lock to exact harmonics in the sustain.
  - A symmetric jet pulse gives mostly odd harmonics. Lip offset adds even harmonics.

  — [Colin Pykett, The physical modelling of organ flue pipes](https://colinpykett.org.uk/complete-physical-modelling.htm)
- **Uncoupled commercial model (Viscount):** Viscount's patented real-time organ model is "uncoupled": a free-running generator excites the resonator, so pitch can be set precisely and instantly, while coupling effects such as transient phase-locking have to be approximated empirically. — [Pykett](https://colinpykett.org.uk/complete-physical-modelling.htm); see also the [organ flue pipe synthesis patent (excitation/resonator split)](https://patents.justia.com/patent/20060201312)

### Inferences
- **Recommended "full" per-pipe model (about 15-25 flops per sample plus two fractional-delay reads):**
  1. `U_j = sqrt(2*max(P,0)/rho)`.
  2. `tau = W/(0.5*U_j)`. This is a fractional delay, so clamp it between about 1 sample and the bore length.
  3. `j = tanh(g*(x_delayed + y0))`, where the y0 offset gives even harmonics (Pykett's lip offset).
  4. Feed j, scaled by P, into a lossy bore delay line with a one-pole lowpass at the reflection, as in STK.
  5. Add `noise * k * sqrt(P)` at the jet input for turbulence.

  The threshold, octave jumps and pitch drift then come from the model itself. For 35 pipes this is cheap, well under 1% of one core in Rust. Replacing tanh with a rational approximation or the STK cubic keeps it branch-free.
- Zadar's acoustic pipes are short horizontal tubes ending in small orifices (about 6-18 mm², see Section 3), not voiced organ mouths. They behave more like whistles (hole tone or edge tone over a cavity) than like organ flues. A modal "jet + 1-2 resonator modes" toy model is probably closer than a long bore waveguide. Tuning targets: odd segments D-G-d-g-b and even segments C-D-c-e-a, roughly 60-250 Hz (see Section 3).
- Pitch dependence on pressure: in the toy model, pitch rises slightly as τ shrinks. Mapping about ±6% (one semitone) of drift over the useful pressure range matches Pykett's figure and gives the slightly unstable sea-organ character.
- Use hysteresis deliberately. Keep a per-pipe "register" state that switches up above P_hi and back down below P_lo, with P_lo < P_hi. This emulates overblowing cheaply even if the resonator is a simple model.

### Gaps
- I could not get the exact STK source text or the exact JetTable code in this session. Verify against the repository before porting.
- I found no published measured thresholds (in Pa) for Zadar's whistles, and no harmonic spectra of Zadar recordings.
- The Välimäki/Karjalainen flute nonlinearity details (sigmoid versus polynomial, measured jet functions) were not retrieved.
- The Fletcher & Rossing chapter on flue pipes was not accessed directly.

## 2. Cheaper alternatives: additive, modal and noise-filter synthesis

### Takeaway
An "uncoupled" source-filter design is the cheapest robust option and is what commercial organ modelling does (Viscount). It drives a few sine partials, plus band-passed noise, plus a short chiff burst from a per-pipe pressure envelope, with the threshold, pitch drift and register switching applied as explicit mappings instead of emerging from a nonlinear loop. A resonant bandpass (biquad or state-variable filter) excited by noise is the cheapest way to get a breathy, unstable whistle.

### Cited Findings
- An uncoupled generator → resonator model lets frequency be set precisely and nearly instantly. Coupling effects must then be added by empirical tuning. — [Pykett](https://colinpykett.org.uk/complete-physical-modelling.htm)
- The pipe resonator can be represented in the frequency domain by its natural frequencies, end corrections and Q factors, which together act as an asymmetric comb filter. — [Pykett](https://colinpykett.org.uk/complete-physical-modelling.htm)
- The STK flute injects breath noise at the jet input. The default noiseGain is 0.15 relative to breath, and a 5.925 Hz vibrato at gain 0.05 is added to the breath pressure. — [STK Flute.cpp](https://raw.githubusercontent.com/thestk/stk/master/src/Flute.cpp)
- Real-time aeolian-tone (vortex-shedding) compact source models exist (Selfridge, Moffat, Avital, Reiss). They are physically informed, run in real time, and have been demonstrated on a sword, an aeolian harp and a propeller. This is relevant as a low-pressure whistling or edge-tone layer. — [Selfridge et al., Real-time physical model of an aeolian harp (IOA)](https://www.ioa.org.uk/system/files/publications/R%20SELFRIDGE%20DJ%20MOFFAT%20JD%20REISS%20EJ%20AVITAL%20REAL-TIME%20PHYSICAL%20MODEL%20OF%20AN%20AEOLIAN%20HARP.pdf); [Selfridge thesis, Creating Real-Time Aeroacoustic Sound Effects](https://pearl.plymouth.ac.uk/handle/10026.1/16546)
- Dobashi et al. precompute CFD-based "sound textures" for aerodynamic sound and play them back in real time, scaled by speed, because running CFD at audio rate is too expensive. — [Sound synthesis survey (arXiv 2011.05538)](https://arxiv.org/pdf/2011.05538); [Stanford aerosound reimplementation](https://web.stanford.edu/~jw969/assets/projects/aerosound/aerosound.pdf)

### Inferences
These are my design proposals; the parameter values are suggestions, not sourced.

- **Cheap per-pipe voice (about 4 oscillators + 2 SVFs, around 40 flops per sample):**
  1. Compute excess pressure `e = max(0, P - P_th)` and drive amplitude `A = smoothstep(e/P_sat)`.
  2. Set the fundamental to `f = f0 * (1 + kpitch * (P/P_ref - 1))`, with about 1-6% drift over the range.
  3. Use 3-5 harmonics with weights that brighten with pressure, for example `h_n ∝ A^(1+0.5n)`. Weight odd harmonics more, and add a small even component for lip offset.
  4. For breath noise, pass white noise through an SVF bandpass centred on f (Q about 5-20) plus a broadband highpassed hiss. Scale the hiss by `sqrt(P)` and keep it present even below threshold, so air sound leaks before the pipe "speaks".
  5. For jitter, add slow random-walk frequency modulation (about 0.1-0.5% at 2-8 Hz) and amplitude modulation from lowpassed noise. This gives the "slightly unstable" whistle.
  6. For chiff on onset, when A crosses up from 0, fire a 20-80 ms decaying burst of noise bandpassed at 2f, 3f or 5f. You can also briefly detune the partials and glide them into lock over about 50 ms, mimicking Pykett's phase-locking description.
  7. For register switching, use hysteresis thresholds (P_hi, P_lo) to crossfade the fundamental to 2f (or to the next partial) over about 20-50 ms.
- **Alternative: "noise-excited resonator bank".** Feed noise (and the chiff burst) into 2-4 high-Q biquads at the partial frequencies, with Q rising with pressure. This is cheap and inherently breathy. At very high Q (above about 200) it begins to sound like a pure whistle with natural amplitude fluctuation. Each biquad costs about 5 multiply-adds.
- Hybrid option: use the STK-style waveguide for a few "hero" pipes near the listener and the cheap voice for the rest.
- For an AudioWorklet, 35 cheap voices at 48 kHz is about 35 × 40 × 48k ≈ 67 Mflop/s. This should be comfortable in JS or WASM. Avoid `Math.tanh` in hot loops on older engines and use a rational approximation instead.

### Gaps
- I found no specific DAFx/ICMC paper comparing CPU cost and perceptual quality of modal versus waveguide flue synthesis. The CPU figures above are estimates.

## 3. Driving signal: wave pressure, water-column / air-chamber dynamics, propagation

### Takeaway
There is a validated, directly implementable physical model of the Zadar organ's internal hydraulics (Krvavica et al.). It has three ODE states per pipe: water column length, flow rate, and air overpressure. It is driven by a random-phase wave spectrum, and the pipe air pressure feeds the orifice/whistle. Water-column resonances are at about 0.2-0.55 Hz with damping ratio ζ of about 0.09-0.14. The air chamber acts as a nonlinear low-pass and resonator on wave pressure, so a cheap substitute is a per-segment second-order resonant lowpass (f_r, ζ) applied to a JONSWAP or random-phase wave signal, followed by a sqrt/abs nonlinearity for orifice flow.

### Cited Findings
- **Zadar geometry:**
  - About 75 m long, with 7 segments of 5 pipes each (35 pipes).
  - Each pipe has a submerged horizontal entry pipe (D1 = 0.3 m), a sloped pipe (D2 = 0.075 m, length L2 = 2.2-5.26 m, slope 20-55°), and a horizontal acoustic pipe under the walkway (L3 = 0.3 m, D3 = 0.075 m) that ends in a small orifice.
  - Odd segments are tuned to G major (D-G-d-g-b) and even segments to C major with an added sixth (C-D-c-e-a). The tones lie at about 60-250 Hz.

  — [Krvavica et al., Time-dependent numerical model for simulating internal oscillations in a sea organ (arXiv 1903.09314)](https://ar5iv.labs.arxiv.org/html/1903.09314)
- **Governing equations (Eq. 25):**
  - `dl2/dt = Q/A2`.
  - `dQ/dt = [p_wave/ρ − Δp/ρ − g·l2·sinφ − β·Q|Q|/2] / [L1/A1 + l2/A2]`.
  - `dΔp/dt = [sign(Δp)·c²·C_d·A0·sqrt(2|Δp|ρ_air) − γ·p·Q] / [A3·L3 + A2·(L2 − l2)]`.
  - Air is treated as isentropic with γ = 1.4, and β combines entry, elbow and friction losses.
  - Orifice flow is `ṁ = sign(Δp)·C_d·A0·sqrt(2|Δp|ρ_air)`, with calibrated C_d of 0.6, 0.64 and 0.7 for A0 = 18, 12 and 6 mm².
  - The paper integrates with an implicit trapezoidal rule and Broyden iterations.
  - The sign convention in the Δp equation is as extracted. Verify against the PDF.

  — [arXiv 1903.09314](https://ar5iv.labs.arxiv.org/html/1903.09314)
- **Resonances by segment (Table 2):** f_r = 0.215, 0.263, 0.308, 0.275, 0.323, 0.449 and 0.528 Hz, with ζ = 0.115, 0.101, 0.103, 0.087, 0.116, 0.137 and 0.140. The strongest response is near resonance. — [arXiv 1903.09314](https://ar5iv.labs.arxiv.org/html/1903.09314)
- **Wave cases and validation:**
  - Case 1: H_s = 0.4 m, f_p = 0.34 Hz (light NW wind).
  - Case 2: H_s = 1.0 m, f_p = 0.22 Hz (strong SE wind).
  - The Adriatic "T-spectrum" was used. Still water level is +0.35 m asl, and segment crest heights range from 0.30 to 1.30 m asl.
  - Wave pressure is hydrostatic plus dynamic from N linear components, multiplied by (1 + K_r), with `K_r = (6/11)·h_c/H_s + 0.7` to account for reflection.
  - Lab validation gave a water-level RMSE of 5.1 mm and an air-pressure RMSE of 20.5-53.2 Pa.

  — [arXiv 1903.09314](https://ar5iv.labs.arxiv.org/html/1903.09314)
- **Earlier integrated model:** An earlier integrated model (Građevinar 2018) chains a wave generator, water-mass oscillation and a musical sound generator. It finds that segments differ mainly because of pipe geometry and that the sound depends on wave length and direction. — [DOAJ: Građevinar 2018](https://doaj.org/article/4e06d34ff32a4159b128200cf201c7dd); [CROSBI 903100](https://www.bib.irb.hr/903100?rad=903100)
- **Simplified model:** A 2D surface-wave generator feeds an algorithm that turns sea-surface time series into MIDI notes (frequency, duration, intensity). — [Peroli & Krvavica, A Simplified Computational Model of the Sea Organ in Zadar, Zbornik Radova 19(1) 2019](https://zr.gradri.uniri.hr/zr/en/article/view/7)
- **Random-phase sea synthesis recipe:**
  - `η(t) = Σ a_i cos(ω_i t + φ_i)`, with `a_i = sqrt(2 S(ω_i) Δω)` and φ_i drawn uniformly from [0, 2π).
  - JONSWAP: `S(ω) = α g² ω⁻⁵ exp(−5/4 (ω_p/ω)⁴) γ^r`, where `r = exp(−(ω−ω_p)²/(2σ²ω_p²))`, typically γ = 3.3 and σ = 0.07/0.09.
  - Check the result against `H_s = 4 sqrt(m0)`.
  - α parameterisations vary between sources.

  — [JonSwap.m (Univ. Trieste course)](https://moodle2.units.it/pluginfile.php/816322/mod_resource/content/2/JonSwap.m); [arXiv 1610.09558](https://arxiv.org/pdf/1610.09558); [netsse example](https://netsse.readthedocs.io/en/wip/examples/example1.html). The σ values come from the standard JONSWAP definition I know; the search snippets only mentioned the γ^r form.

### Inferences
These are implementation proposals, not sourced.

- **Control-rate driver.** The wave signal is about 0.05-1 Hz, so run it at block or control rate (for example once per 32-128 samples, or about 1 kHz) and interpolate. 20-50 random-phase cosines from JONSWAP cost almost nothing. A recursive oscillator bank or a precomputed 15-30 minute looping table avoids audible repetition. To avoid periodicity, slowly re-randomise phases, or use a filtered-noise alternative: white noise through a 2nd- or 4th-order bandpass at f_p with Q of about 2-4 gives similar narrow-band, group-structured waves.
- **Wave groups.** The sum of random-phase components with a peaked spectrum (γ = 3.3) naturally produces groups, with an envelope period of roughly 1/Δf of the spectral width. A higher γ gives longer groups, so this is a single "sea state" knob.
- **Wakes.** Add Poisson-timed events (rate λ of about 1 per 5-20 minutes) of short wave trains, for example 5-15 waves at 0.4-0.7 Hz with a Gaussian envelope, on top of the wind sea. The Krvavica papers list boats as a driver, but I found no quantitative wake-model source.
- **Propagation along 70-75 m.** Apply a per-pipe delay `Δt = x·cos(θ)/c_g` for wave direction θ. For deep water `c_g = g/(2ω)` (about 2.3 m/s at 0.34 Hz). In shallow water use `c = sqrt(g·h)`. A wave crossing the full organ obliquely therefore takes tens of seconds, which produces the characteristic "rolling" chord. Implement it per segment using phase offsets `k·x` in the cosine sum, which is exact and free, rather than delay lines.
- **Cheap air-chamber substitute.**
  1. Pass the wave pressure through a per-segment 2nd-order resonator at the Table 2 values of f_r and ζ. That gives water-column displacement, which is proportional to chamber overpressure.
  2. Clamp at 0 for when water drops below the pipe mouth, because then no driving occurs.
  3. Model orifice flow velocity as `v ∝ sqrt(|Δp|)`.

  Only outflow (positive Δp) should sound loudly. Inflow (suction) may produce a weaker, different-pitched or noisier sound. This is a design choice, because I have no source on the Zadar inflow sound.
- **Full model option.** Krvavica's three-state ODE per pipe is small enough to run at about 1 kHz with semi-implicit Euler or RK2 (the paper used implicit trapezoidal with Broyden). That is 35 × 3 states, which is trivial. The stiff term is the orifice sqrt near Δp = 0, so regularise it with `sqrt(|Δp| + ε)`.
- **Whistle pressure mapping.** Chamber overpressure in Pa maps to jet velocity `U_j = sqrt(2Δp/ρ_air)`. For example, 100 Pa gives about 13 m/s, and the lab RMSE values suggest pressures of tens to hundreds of Pa. This feeds Section 1's τ directly.

### Gaps
- I did not obtain absolute air-pressure amplitudes for the real Zadar organ. The paper reports only relative trends, and Table 1's A0 value was garbled ("112 mm²", possibly 12 mm²).
- I could not find the Adriatic "T-spectrum" parameters.
- The full texts of the 2018 integrated model and of Peroli 2019 were not accessed (DOAJ returned a 403).

## 4. Existing projects emulating sea organs or wave organs; ocean sonification

### Takeaway
Zadar-specific digital work is mainly engineering simulation, plus one sampling/installation project. I found no published NIME/ICMC/DAFx physical-model sea organ synthesiser, so a real-time physically informed synthesiser appears to be a gap in the literature.

### Cited Findings
- Krvavica, Ružić, Ožanić et al.: an integrated model of wave generation, pipe hydraulics and a sound generator (2018), and a time-dependent 1D hydro-thermodynamic model validated on a scale model (Ocean Engineering, 2020; arXiv 1903.09314). — [CROSBI 994019](https://www.bib.irb.hr/994019); [arXiv 1903.09314](https://ar5iv.labs.arxiv.org/html/1903.09314); [AIRI Rijeka project page](https://airi.uniri.hr/research/time-dependent-numerical-model-for-simulating-internal-oscillations-in-a-sea-organ/)
- Peroli & Krvavica (2019) map wave time series to MIDI notes. — [Zbornik Radova](https://zr.gradri.uniri.hr/zr/en/article/view/7)
- Global Sound Movement recorded the Zadar organ multichannel, sampled each octave (including mechanical noise), and built a MIDI sampler instrument. — [Global Sound Movement, Zadar 2017](https://globalsoundmovement.com/?p=2816)
- *Morske Musak* (2018) is an installation where visitors control pitch, rhythm and diffusion parameters over recordings of the organ. — [REF 2021 output record](https://results2021.ref.ac.uk/outputs/fad6b853-c522-40e9-9241-0e1af631ee7e?page=1)
- In SuperCollider, I found only generic ocean-sound patches built from filtered noise and LFNoise modulation (sccode.org "Wave-like"), not a wave-organ model. — [sccode.org](https://sccode.org/1-5go-HTfPyehpvV/fork)
- Farnell's *Designing Sound* (MIT Press, 2010) is the standard procedural-audio reference with Pure Data implementations. I could not confirm whether it contains an ocean or wind model specifically. — [Designing Sound listing](https://tenlong.com.tw/products/9780262014410)

### Inferences
- The Global Sound Movement samples could serve as listening references for calibrating timbre, such as noise-to-tone ratio and register-jump character.

### Gaps
- I found no NIME, ICMC, DAFx or SMC paper on sea or wave organ synthesis. Searches were limited, so this is a search gap rather than proof of absence.
- I did not investigate the Exploratorium Wave Organ (San Francisco), the Blackpool High Tide Organ or the Sea Organ in Sanya.
- I found no measured spectral analysis of Zadar recordings.

## 5. Spatial and ambience modelling

### Takeaway
I did not research this area with dedicated sources in this session, so everything below is standard DSP practice from prior knowledge and is unsourced. The practical approach is to treat each of the 7 segments, or each of the 35 pipes, as a point source along a line. Give each a distance-dependent delay, 1/r gain, air-absorption lowpass and stereo/HRTF pan. Then add a short, dense "stone cavity" early reflection or comb resonance per segment, one shared outdoor reverb, and a separately synthesised surf bed.

### Cited Findings
- The sound exits through openings in the top and sides of the stone steps, and the acoustic pipes sit in channels under the walkway. — [arXiv 1903.09314](https://ar5iv.labs.arxiv.org/html/1903.09314)

### Inferences
These are unsourced, from general DSP knowledge.

- **Source layout.** Place 35 pipes over about 70-75 m, about 2 m apart with 5 per segment. For a listener at position L, each pipe gets:
  - `r_i = |p_i − L|`;
  - delay `r_i/343` s, using a fractional-delay line or a single shared multi-tap line;
  - gain `1/max(r_i, 1)`;
  - a one-pole lowpass whose cutoff falls with distance, for air absorption plus ground and stone shadowing;
  - constant-power pan or ITD/ILD.

  Cost is about 10 flops per source per sample. Moving the listener causes Doppler shifts, so smooth the delay changes.
- **Stone cavity.** Each segment's channel under the steps behaves like a small cavity. A 2-4 tap comb or allpass network (about 2-15 ms delays, high damping), or a tiny 4×4 feedback delay network per segment, gives a "hollow" coloration. Then share one outdoor reverb across all sources: sparse early reflections from the steps and the sea surface plus a short, low-density tail. Open seafront space has little reverberation.
- **Surf bed.** Use pink or brown noise through a lowpass (about 300-1500 Hz) with amplitude following the same wave signal η(t), delayed by the breaking time. Add highpassed "fizz" bursts on wave arrivals and a slow stereo decorrelation. Tying it to the same driver keeps the surf synchronised with the pipes.
- **CPU priority.** Put spatialisation on the cheap per-source path and use one shared reverb. Avoid per-pipe convolution.

### Gaps
- I found no sources on measured acoustics of the Zadar steps or cavities, outdoor coastal reverb parameters, or procedural surf synthesis papers. A follow-up search is recommended, for example on Farnell's ocean and wind chapters or DAFx procedural water papers.
