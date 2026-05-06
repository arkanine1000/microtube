# Journey Through the Cosmos
## Notes on a 25½-minute strange loop, in a Rust terminal

> *The cosmos is the microtubule. The microtubule is the cosmos.*
> A 25-minute audiovisual sequence, and what eight thinkers had to do with it.

---

## 0. The night I tried to write a piece of music as a piece of physics

You can `cargo run --release` it tonight. Press `s`, choose **Journey Through the Cosmos**, set down the laptop, put on headphones. For 25 minutes and 29 seconds, a terminal window will pulse and shimmer and slide between brain-states and cosmic scales while a phantom interference tone — the binaural beat — slows from 40 Hz gamma to 2 Hz delta, then climbs back through every band of human consciousness to gamma again. A second tone, the Shepard–Risset glissando, rises continuously the whole time, except for thirty-four seconds near the end when it falls. Voices spawn out of nowhere, pulled from a Fibonacci-word quasicrystal, and decay back into the carrier. The base frequency descends monotonically from 432 Hz to 55 Hz, like a camera pulling back through scales, until — at the singularity — it reverses and snaps back to 432 in a slow lerp that doubles as the cosmic origin and the closing of the loop.

By the time the last 21-second hold ends, the parameters are byte-identical to the parameters at the very first second. You are back at the microtubule. You have travelled through the cosmos. *You are not the same.* That is the whole idea.

This essay is the attempt to say, plainly, what the sequence is doing and why — *artistically*, *mathematically*, and (where the evidence permits) *physically*. It is also a partial inventory of debts: to Penrose, Hameroff, Hofstadter, Bach, Escher, Gödel, Shepard, Risset, Schumann, Conway, and the small army of physicists currently re-running quantum-biology experiments because of them.

---

## 1. MicroTube, briefly

MicroTube is a terminal-native binaural beats engine I have been building in Rust for a few weeks. It has no GUI, no shaders, no GPU — every visual is drawn with Unicode braille and block elements (the 2×4 sub-cell trick gives surprising resolution); every sound is synthesised from phase-accumulator sines into a `cpal` stereo stream at 48 kHz. The audio thread runs lock-free, parameters arrive as `f32`-bit-cast `AtomicU32`s, and a 50 ms exponential smoother absorbs any rough edges so a sequence step can rewrite ten parameters at once and still sound continuous.

The point of the program is to be a small, focused instrument for *deliberate listening* — for sessions, not playlists. It ships with five preset brainwave bands, six visualisations, four timbres (Organ · Flute · Bell · Saw), five "mist" textures (pink · white · brown · blue · velvet noise), a Shepard–Risset glissando, and a generative emergence engine that spawns up to twelve voices at harmonic ratios with consonance-weighted lifetimes — Bach canons or Penrose quasicrystal walks, depending on mode. Until last week the sequence layer only automated two of those parameters. Now it automates all of them. The first sequence to use the full surface is the one this essay is about.

What follows isn't a manual. It is a set of notes on the *materials* — physical, mathematical, contemplative — out of which the journey is built.

---

## 2. The first material: the microtubule

The journey opens at 432 Hz, gamma beat, Bell timbre, velvet noise, Penrose spawn mode, Emergence visualization. The label in the session panel reads `epoch · Microtubule`. This is a deliberate citation.

In 1996, Stuart Hameroff (an anaesthesiologist at Arizona) and Roger Penrose (the geometer who would share the 2020 physics Nobel for proving black-hole singularities are generic) proposed that consciousness is not produced by neurons firing per se but by **orchestrated objective reduction** — *Orch-OR* — of quantum superpositions hosted inside microtubules, the cytoskeletal protein lattices that thread every neuron. Each conscious moment, on this theory, is the collapse of a wavefunction that has been protected from environmental decoherence long enough for gravitational self-energy to trigger a Diósi–Penrose reduction at timescale `τ ≈ ℏ / E_G` [^hameroff2014].

It is a theory that the physics community has spent thirty years arguing about. The *neuroscience* community has mostly raised an eyebrow and walked off. Decoherence at body temperature would seem to forbid any of this; Max Tegmark calculated a survival time for microtubule superpositions of femtoseconds, far below the ~25 ms of a conscious moment.

And yet the theory keeps getting revived by experimental hits. The 2014 Hameroff–Penrose review introduced a new and pointed claim: that the EEG rhythms we measure with scalp electrodes — alpha, theta, gamma — might be **beat frequencies** of much faster microtubule vibrations, the way a musical beat at 10 Hz can be the difference between two carrier tones at 220 and 230 Hz. They wrote, of these microtubule oscillations, that they "could be the source of EEG, like a piano with two strings vibrating at 440 and 442 Hz heard as a 2 Hz beat" [^hameroff2014]. MicroTube *is* that interference, made literal. The whole instrument is a kind of toy Orch-OR organ.

Then in April 2024, Babcock, Kurian, and collaborators published a paper that, depending on whom you ask, is either a sober quantum-biology result or a bombshell. They built a Lindblad-type model of the **tryptophan networks** in microtubules and centrioles — meshes of >10⁵ aromatic chromophores, each modelled as a two-level UV emitter — and showed that under single-photon excitation these networks form **collective superradiant states**: the kind of cooperative, coherent emission Robert Dicke first described for atomic ensembles in 1954. The brightest superradiant states emit in hundreds of femtoseconds; the darkest "subradiant" states sit on tens of seconds. The fluorescence quantum yield grows with system size. Crucially, **the effect survives thermal disorder** — they show it persists at room temperature [^babcock2024]. A follow-up by Patwa, Babcock, and Kurian extended the same machinery to actin filaments and amyloid fibrils [^patwa2024].

What this does *not* do is prove Orch-OR. What it does do is take the most often-quoted objection to Orch-OR — *quantum effects can't survive a warm wet brain* — and demote it from impossibility to "depends on the geometry." When the journey opens with the microtubule epoch, the bell-like inharmonic timbre and the sparse velvet noise are aimed at exactly that picture: a coherent quantum state, occasionally fluctuating, in which voices spawn from a Penrose quasicrystal walk because the underlying lattice itself has Penrose-style aperiodic order.

I do not know if Orch-OR is true. I find myself caring less than I expected. The journey is not a thesis statement; it is a meditation on what *would have to be true about the universe* for the theory to make sense. That is a different and more useful kind of art.

---

## 3. The second material: the strange loop

Twenty-five minutes and twenty-nine seconds is a strange amount of time for a binaural beats sequence. The Fibonacci sums look like this:

```
21 + 34 + 55 + 89 + 144 + 233 + 377 + 233 + 144 + 89 + 55 + 34 + 21 = 1529 s
```

The durations rise to **377 s** — 6 minutes 17 seconds at the *Solar Wind* epoch — and then collapse symmetrically. The whole sequence is therefore palindromic in time but asymmetric in content: outward expansion through cosmic scales, then a ferocious eight-minute compression through the cosmic web, the CMB, and the singularity, and finally a 34-second lerp back to the parameters of step 1. This is a strange loop in the precise sense Douglas Hofstadter gave the term in *Gödel, Escher, Bach* (1979) and, more pointedly, in *I Am a Strange Loop* (2007):

> "What I mean by 'strange loop' is — here goes a first stab, anyway — not a physical circuit but an *abstract* loop in which, in the series of stages that constitute the cycling-around, there is a shift from one level of abstraction (or structure) to another, which feels like an upwards movement in an hierarchy, and yet somehow the successive 'upward' shifts turn out to give rise to a closed cycle. That is, despite one's sense of departing ever further from one's origin, one winds up, to one's shock, exactly where one had started out. In short, a strange loop is a paradoxical level-crossing feedback loop." [^hofstadter2007]

Hofstadter's three exemplars in *GEB* are Gödel's incompleteness statements, M. C. Escher's lithographs (*Drawing Hands*, *Ascending and Descending*, *Waterfall*) and a single canon of J. S. Bach's: the *Canon per Tonos* from the *Musical Offering* (BWV 1079) [^geb1979]. Bach's canon modulates upward by a whole step at the end of each repetition; after six repetitions it has climbed a major sixth and returned, in pitch class, to where it began — an octave higher than it started. He notated it to be played indefinitely. As one survey puts it, the canon is "literally an infinite ascending spiral in pitch... the contrapuntal equivalent" of a Shepard tone [^datafield2026].

That is the engine of the journey. The first epoch (Microtubule) and the last epoch (Strange Loop) hold *byte-identical parameter values* — beat 40 Hz, base 432 Hz, Bell timbre, Velvet mist, Penrose spawn, Up direction, Emergence visualisation. The lerp between Singularity and Strange Loop is the closing of the loop, and the listener crosses it without any obvious local change: each tick of the audio thread interpolates one part in a thousand toward the destination. The arrival is invisible. Only afterwards, looking at the parameter panel, does one notice that the new parameters are identical to the old. *You wound up, to your shock, exactly where you had started out.*

There is a working hypothesis behind this construction — Hofstadter's hypothesis, stretched into time. *I Am a Strange Loop* proposes that the self is not a substance but a pattern of self-reference looped through a substrate; a loop in which symbols stand for other symbols, including, eventually, themselves. If consciousness is such a loop, then any meditation that *itself* loops at multiple scales — quantum, neural, planetary, cosmic — and arrives back at its starting state is doing, structurally, the thing it is meditating on. The journey is a strange loop *about* strange loops.

This is also why the *parameter snap* between epochs is, in this sequence, often abrupt. Bach modulates on the bar line, not the half-bar. The audio thread's 50 ms smoothing means the listener never hears a click; what they hear is a *deliberate articulation*, the way a key change in classical music marks a passage from one section to the next. The journey has thirteen movements. Each movement has its own key.

---

## 4. The third material: the Earth's cavity

By minute four we are at the *Earth · Schumann* epoch. The beat frequency has dropped to **7.83 Hz**. This is not a coincidence.

In 1952 the German physicist Winfried Otto Schumann predicted a set of standing-wave electromagnetic resonances in the spherical cavity formed between the Earth's surface and the lower edge of the ionosphere [^schumann1952]. The cavity has a circumference of roughly 40,000 km, a fundamental wavelength close to the Earth's circumference, and a fundamental resonant frequency around 7.83 Hz with harmonics near 14.1, 20.3, 26.4, and 32.5 Hz. Lightning strikes — thousands per second worldwide — pump energy into the cavity and the modes are continuously excited.

So far, so meteorological. The interesting part begins with Herbert König's observation, originally in the 1950s, that the fundamental Schumann frequency falls inside the alpha-theta boundary of human EEG. König and several decades of follow-up work compared neural rhythms with the planetary cavity and found systematic correlations [^persinger2014][^nelson2025]. A 2025 *Electromagnetic Biology and Medicine* review surveyed the (still controversial) literature on Schumann–biology coupling, including effects on calcium-channel gating, melatonin secretion, blood pressure, and EEG coherence [^nelson2025]. The same review notes that **"human brainwave activity seems to align with Earth's natural electromagnetic rhythms"** and that future crewed deep-space missions may need to artificially reproduce the Schumann field to maintain crew physiology.

I want to be careful here. The Schumann–biology literature has a noisy long tail — anything billed as the "Earth's heartbeat" attracts a class of writing that is more enthusiastic than empirical. But the *core finding* — that the planetary EM cavity has a fundamental in the same band as the dominant resting human EEG — is durable. Whatever the causal story, the coincidence is real.

The journey treats it as a hinge. Steps 4 and 5 (Body, Earth) are the only steps in the descent that *hold* the same beat frequency for two consecutive epochs. The base frequency is dropping (256 → 196 Hz), the timbre rotates (Flute → Organ), the mist deepens to brown, and the visualisation switches from Envelope (heart-pulse) to Envelope (Schumann pulse). The auditory effect, with headphones, is that the listener has *settled into* the planet — that something underneath the personal nervous system has clicked into the same rhythm as the rock and air around them. This is metaphor at the level of parameters, not a clinical claim. But the coincidence, again, is real.

---

## 5. The fourth material: the quasicrystal

The Penrose-Hameroff microtubule meets a different Penrose at scale 7 (Solar Wind, 377 s) and stays with him until the end. At Solar Wind the spawn mode flips to **Penrose**. Once in Penrose mode, the emergence engine no longer reads from a fixed eight-element canon pattern; it reads from a *Fibonacci word*.

The Fibonacci word is the unique infinite binary string fixed by the substitution

```
σ:  L → LS,  S → L
```

starting from `L`. Iterating gives `L, LS, LSL, LSLLS, LSLLSLSL, ...` — words whose lengths are Fibonacci numbers and where each word is a prefix of the next. The infinite limit `LSLLSLSLLSL...` is **the canonical Sturmian sequence**, the canonical 1D quasicrystal, and a cut-and-project image of the integer lattice ℤ² onto a line of irrational slope `1/φ` where `φ = (1 + √5)/2` is the golden ratio [^ams2014][^bielefeld].

It has properties that read like a list of small miracles:

- It is **aperiodic** (no repeating period at any scale), and yet **self-similar**: any pattern of length *n* recurs at gaps controlled by the substitution.
- The **frequencies** of letter pairs are computable: `LL` appears with asymptotic frequency `1/φ³ ≈ 23.6%`; `LS` and `SL` each appear with frequency `1/φ² ≈ 38.2%`; **`SS` never appears** at all (every short tile in a Fibonacci word is bracketed by long ones). [^bielefeld]
- It is the **trace of a Conway worm** — a row of consecutive parallel rhombs — through Roger Penrose's P3 rhombic tiling of the plane, the same tiling whose 5-fold quasi-symmetry inspired the discovery of physical quasicrystals (Shechtman 1982; Nobel 2011) [^penrose-rhomb][^aperiodic].
- The lengths in Penrose tilings, the distances between Ammann bars, and the ratio of fat to thin rhombs in the infinite tile are all in Fibonacci/golden ratios. The Penrose tiling, the Fibonacci word, and `φ` are *the same phenomenon* viewed three ways.

MicroTube exploits this by mapping pair → ratio:

| Pair | Ratio | Move | Asymptotic frequency |
|------|-------|------|----------------------|
| `LL` | 3:2 | perfect fifth (anchor) | 1/φ³ ≈ 23.6% |
| `LS` | 5:4 | major third (descent) | 1/φ² ≈ 38.2% |
| `SL` | 4:3 | perfect fourth (ascent) | 1/φ² ≈ 38.2% |
| `SS` | — | (never occurs) | 0% |

The result is harmonic improvisation that is structurally aperiodic at every scale and yet bound to a small consonant palette, so the music **never repeats but always sounds like itself**. This is what a Penrose tiling sounds like. This is also, I would venture, what consciousness sounds like — coherent at every scale, but never twice the same configuration.

The cosmic half of the journey lives in this material. Steps 7 through 13 spawn voices from the Fibonacci word; the listener hears a kind of stellar Bach, a fugue with no period.

---

## 6. The fifth material: the endless rise

Layered over almost the entire journey is a second auditory illusion. From step 2 onward the Shepard parameter ramps from 0.10 to 0.85. From step 12 (Singularity) to step 13 the direction flips from Up to Down for thirty-four seconds, then back to Up. The listener never hears a Shepard "wrap-around" — that is the whole trick.

Roger Shepard discovered the discrete version in 1964 [^shepard1964]. A "Shepard tone" is a stack of sine waves spaced one octave apart whose amplitudes are governed by a fixed, bell-shaped spectral envelope. As you raise all the frequencies in lockstep, the lowest sine fades into silence at the bottom edge of the bell while a new sine fades in at the top. After one octave the spectrum is identical to the original, but every individual partial has shifted up by an octave. Played in semitone steps, the tone seems to climb forever; cycled at any interval other than an exact tritone, the brain reliably reads it as "going up." Jean-Claude Risset's 1969 modification swapped the discrete steps for a continuous glissando, producing the *Shepard–Risset glissando* — a literal endless rise [^risset1969][^aes2024].

Christopher Nolan used it in *Dunkirk*, and earlier in *The Dark Knight Rises*, to ratchet tension without ever resolving. It is a powerful stimulus. Mainsbridge & Marques's 2016 work showed that *descending* Shepard glissandi reliably elicit feelings of falling, and that *ascending* ones add momentum without resolution.

MicroTube implements seven octave-spaced sines under a `sin⁴` raised-cosine bell, normalised once at construction so that the total energy is constant as the offsets drift. The default rate is `1/36` octaves per second — a thirty-six-second octave, consciously chosen to be slow enough to feel ambient but fast enough that the motion remains perceptible. By coupling the layer's *direction* to the journey's narrative (`Up` for cosmic expansion, briefly `Down` for the singularity collapse), the program turns the illusion into commentary. The cosmos is rising. It pauses, falls into the singularity, and starts rising again. This is the Friedmann–Lemaître equation rendered as a Risset glissando. It is also the *Canon per Tonos* extended to 25 minutes.

---

## 7. The architecture of the journey

The thirteen epochs and their parameters are listed in full in the source at `src/presets.rs`. Briefly, in narrative order:

| # | Epoch | Duration | Beat | Base | Notes |
|--:|-------|---------:|-----:|-----:|-------|
| 1 | **Microtubule** | 21 s | 40 Hz γ | 432 Hz | Bell timbre, velvet noise, Penrose spawn — quantum coherence |
| 2 | Synapse | 34 s | 22 Hz βγ | 384 Hz | Waveform viz: action potential as raw signal |
| 3 | Neural Awareness | 55 s | 14 Hz β | 320 Hz | Flute, pink noise, Harmonics viz: lattice of brain rhythms |
| 4 | Body | 89 s | 10 Hz α | 256 Hz | Heartbeat scale, Envelope visualisation |
| 5 | **Earth · Schumann** | 144 s | 7.83 Hz | 196 Hz | Brown noise (telluric), Organ — settling into the planet |
| 6 | Lunar Tide | 233 s | 5 Hz θ | 165 Hz | Penrose visualisation — orbital geometry |
| 7 | **Solar Wind** | 377 s | 3 Hz δ | 130.81 Hz (C₃) | The still point. Spawn flips to Penrose |
| 8 | Stellar Bells | 233 s | 2 Hz δ | 110 Hz (A₂) | Bell timbre returns; sparse, inharmonic; blue noise |
| 9 | Galactic | 144 s | 4 Hz δθ | 87.31 Hz (E₂) | Voices everywhere — peak Penrose emergence |
| 10 | Cosmic Web | 89 s | 8 Hz θα | 73.42 Hz (D₂) | Saw timbre, white noise — large-scale structure |
| 11 | Background Radiation | 55 s | 18 Hz β | 65.41 Hz (C₂) | Spectrum viz; CMB-coloured |
| 12 | **Singularity** | 34 s | 60 Hz γ | 55 Hz (~A₁) | Shepard direction flips to Down. Cosmic compression |
| 13 | **Strange Loop** | 21 s | 40 Hz γ | 432 Hz | Identical to step 1 |

The Fibonacci durations (21, 34, 55, 89, 144, 233, 377) rise to the *Solar Wind* climax and descend symmetrically — a quasicrystal in time. The beat frequency traces a U: gamma at the microtubule, plunging through alpha and theta to delta at the stellar dream-depth, then climbing back through beta to gamma at the strange-loop closure. The base frequency descends monotonically because *small things vibrate fast and vast things slow*; the lerp from 55 Hz at the singularity to 432 Hz at the strange-loop closure is the cosmic origin gradually re-becoming the microtubule. Spawn mode is `Penrose` on quantum and cosmic scales (where the universe is genuinely quasicrystalline), `Canon` on human and Earth scales (where Bach is a better metaphor than Roger Penrose).

The visualisation is automated per epoch. Synapse uses *Waveform* (the action potential as raw signal). Neural Awareness uses *Harmonics* (the just-intonation lattice). Body and Earth use *Envelope* (the literal beat-frequency pulse). Lunar and Solar use *Penrose* (orbital geometry). The quantum, stellar, galactic, cosmic-web, and strange-loop epochs use *Emergence*: voices as nodes connected by harmonic relationships — galaxies as a graph of consonance. Background Radiation and Singularity use *Spectrum* — a CMB-style distribution of energy.

Every parameter is automated by the same machinery. The schema is the obvious extension of the existing one: each step gets `Option`s for volume, noise level, harmonics, emergence, shepard, timbre, mist type, shepard direction, spawn mode, visualisation mode. Continuous fields (volume, noise, harmonics, emergence, shepard) are linearly interpolated toward the next step's value over the step's duration. Discrete fields (timbre, mist, direction, spawn, viz) snap on entry — and the audio thread's exponential 50 ms parameter smoother absorbs the boundary so there is no click. The first step's snap is what `start_sequence` does. The last step's `next` is itself, so the final 21 seconds are a held silence-on-the-microtubule, and the listener has time to absorb that the loop has closed.

---

## 8. A caveat on entrainment evidence

Anything called a "binaural beats engine" runs into the question of *whether binaural beats actually entrain*. The honest answer, reading the literature, is **the evidence is mixed**.

The strongest result is Garcia-Argibay, Santed, and Reales's 2019 meta-analysis in *Psychological Research*: 22 studies, 35 effect sizes, an overall effect size *g* = 0.45 — medium and significant — for binaural-beat exposure on cognition, anxiety, and pain perception [^garcia2019]. A 2023 follow-up by Basu and Banerjee, also in *Psychological Research*, looked specifically at memory and attention and found *g* = 0.40 across 31 effect sizes [^basu2023].

These are not trivial effects.

But Ingendoh, Posny, and Heine's 2023 systematic review in *PLOS One* is more sceptical. They examined fourteen studies of binaural-beat effects on EEG parameters specifically — that is, on the brainwave-entrainment hypothesis itself, not on downstream behaviour. Five studies confirmed the entrainment hypothesis, eight contradicted it, one was mixed. The methodological heterogeneity was so extreme that they declined to do a meta-analysis [^ingendoh2023]. Aparecido-Kanzler's earlier review of 17 controlled trials found that 82.35% reported binaural or monaural beat stimulation more effective than a control, but cautioned that the evaluation tools across studies were not comparable [^kanzler2021].

What I take from this: binaural beats *probably* do something — the meta-analytic effect on behaviour is real and fairly robust — but the simple story that the brain's dominant frequency literally synchronises with the difference frequency is less well-supported than popular sources suggest. The mechanism may have more to do with cross-frequency coupling, attention-narrowing, or the meditative effect of long sustained tones than with literal entrainment.

This is why I have tried to write this essay, and the journey itself, in a particular register: as art that *takes the science seriously without overclaiming it*. The microtubule epoch is not a clinical Orch-OR therapy. The Schumann epoch is not a treatment. The journey is a contemplative instrument tuned by the best available picture of physics, biology, and mathematics — calibrated so that if the underlying picture turned out to be true, the sequence would happen to be *right*. If it turns out to be false, the sequence is still a beautiful piece of music. Either way, listening is its own reward.

---

## 9. Methodology, in brief

The decisions encoded in the journey were made by combining four constraints:

1. **Fibonacci durations.** The whole sequence is a quasicrystal in time. The longest epoch (Solar Wind, 377 s = F₁₄) sits at the still-point of the descent; the shortest epochs (21 s = F₈) bracket the loop closure.

2. **U-shaped beat frequency.** The descent through brain bands (gamma → delta) mirrors the cosmic zoom-out (microtubule → stellar). The ascent (delta → gamma) mirrors the compression back toward the cosmic origin. Schumann's 7.83 Hz lives at the natural hinge where the human nervous system has, for whatever reason, evolved to rest.

3. **Monotonically descending base frequency.** Small things vibrate fast, vast things slow. 432 Hz at the microtubule is high quantum coherence; 55 Hz at the singularity is cosmic bass. The strange-loop step's lerp from 55 to 432 over 34 seconds is the universe giving birth to the microtubule.

4. **Capability saturation.** All 4 timbres, all 5 mist textures, both spawn modes, both Shepard directions, all 6 visualisations are used. The brief was *make use of all program capabilities currently implemented*, and I read that as art-as-inventory — every dial of the instrument turned at least once.

The architecture supports any other narrative arc the program is asked to encode. The schema is `(beat_freq, base_freq, duration_secs, Option<everything else>)`; the existing five sequences set every `Option` to `None` and behave exactly as before. The new step name field surfaces as an `epoch` row in the session panel during a journey, replacing the `preset` row that would otherwise read `Custom`.

---

## 10. The loop closes

I want to end where the sequence ends.

The Singularity epoch holds 34 seconds — F₉. Its parameters are intentionally extreme: low base, high noise, low volume, Saw timbre, Down-direction Shepard. Within those 34 seconds the linear interpolator slowly drags every continuous parameter toward the values of the next epoch — Strange Loop — which are, again, *byte-identical* to Microtubule. The listener does not perceive the transition as a transition. They perceive a gradual emergence: the noise thins, the volume rises, the bass climbs, the Bell harmonics return, the visualisation snaps from Spectrum back to Emergence.

And then they are at the Microtubule again. The beat is 40 Hz gamma. The base is 432 Hz. The Shepard is silent. The voices spawn from a Penrose quasicrystal walk. The bell-like timbre and velvet noise sketch a coherent quantum state. The session panel reads `epoch · Strange Loop`.

> *And yet when I say "strange loop", I have something else in mind — a less concrete, more elusive notion... despite one's sense of departing ever further from one's origin, one winds up, to one's shock, exactly where one had started out.* — Hofstadter [^hofstadter2007]

For 21 seconds the parameters are held there, and the journey ends. The audio does not stop; the sequence simply releases the parameters back to manual control. If the listener wants, they can sit inside the microtubule for as long as they like, knowing that they have just contained the cosmos.

If Hameroff and Penrose are right, this is also what the listener was, all along.

---

## References

[^hameroff2014]: Hameroff, S., & Penrose, R. (2014). **Consciousness in the universe: A review of the 'Orch OR' theory.** *Physics of Life Reviews*, 11(1), 39–78. https://doi.org/10.1016/j.plrev.2013.08.002 *(open access)*. Introduces the "beat frequencies of microtubule vibrations as EEG sources" hypothesis.

[^babcock2024]: Babcock, N. S., Montes-Cabrera, G., Oberhofer, K. E., et al. (2024). **Ultraviolet Superradiance from Mega-Networks of Tryptophan in Biological Architectures.** *The Journal of Physical Chemistry B*, 128(17), 4035–4046. https://doi.org/10.1021/acs.jpcb.3c07936 *(PMC: PMC11075083)*. Predicts and experimentally observes collective superradiant states in tryptophan networks of microtubules and centrioles, robust to thermal disorder.

[^patwa2024]: Patwa, H., Babcock, N. S., & Kurian, P. (2024). **Quantum-enhanced photoprotection in neuroprotein architectures emerges from collective light-matter interactions.** *arXiv:2406.15403*. https://arxiv.org/abs/2406.15403. Extends the superradiance machinery to actin filaments and amyloid fibrils.

[^geb1979]: Hofstadter, D. R. (1979). **Gödel, Escher, Bach: An Eternal Golden Braid.** Basic Books. *(Pulitzer Prize, 1980.)* Coins "strange loop"; treats Bach's *Canon per Tonos* and Escher's *Drawing Hands* and *Waterfall* as exemplars.

[^hofstadter2007]: Hofstadter, D. R. (2007). **I Am a Strange Loop.** Basic Books. The strange-loop definition quoted above is on pp. 101–102.

[^datafield2026]: "Bach's Musical Offering — A Mastercrafted Network of Canons." *Physics of Music* (case study, DataField.Dev, 2026). On the *Canon per Tonos* as "literally an infinite ascending spiral in pitch... the contrapuntal equivalent" of a Shepard tone. https://datafield.dev/physics-of-music/chapter-16-symmetry-music-physics/case-study-01.html

[^schumann1952]: Schumann, W. O. (1952). Über die strahlungslosen Eigenschwingungen einer leitenden Kugel, die von einer Luftschicht und einer Ionosphärenhülle umgeben ist. *Zeitschrift für Naturforschung A*, 7(2), 149–154.

[^persinger2014]: Saroka, K. S., & Persinger, M. A. (2014). **Quantitative Evidence for Direct Effects between Earth-Ionosphere Schumann Resonances and Human Cerebral Cortical Activity.** *International Letters of Chemistry, Physics and Astronomy*, 39, 166–194. Notes that the first Schumann harmonic (~7.83 Hz) sits inside the alpha/theta boundary, and reports cross-frequency coupling between Schumann modes and EEG coherence.

[^nelson2025]: Nelson, I., et al. (2025). **Exploring the influence of Schumann resonance and electromagnetic fields on bioelectricity and human health.** *Electromagnetic Biology and Medicine*, 44(3). https://doi.org/10.1080/15368378.2025.2508466. Reviews mechanisms by which 7.83 Hz fields may modulate calcium-channel gating and resting membrane potential.

[^shepard1964]: Shepard, R. N. (1964). **Circularity in Judgments of Relative Pitch.** *Journal of the Acoustical Society of America*, 36(12), 2346–2353. Original Shepard-tone construction.

[^risset1969]: Risset, J.-C. (1969). *An Introductory Catalogue of Computer Synthesized Sounds.* Bell Telephone Laboratories. Introduces the continuous-glissando "Shepard–Risset" variant.

[^aes2024]: Author's name redacted in source PDF; (2024). **Variations on Shepard-Tone and Shepard–Risset Glissando Illusions.** *Audio Engineering Society Convention*, paper 22391. Demonstrates that continuous glides between Shepard tones disambiguate the tritone-paradox direction.

[^ams2014]: Berthé, V., & Rigo, M. (2014). **Combinatorics, Words and Symbolic Dynamics.** *AMS Notices*, 61(7). https://www.ams.org/notices/201407/rnoti-p768.pdf. Treats the Fibonacci word as the canonical 1D cut-and-project quasicrystal.

[^bielefeld]: Tilings Encyclopedia, *Fibonacci substitution tiling*. Bielefeld University. https://tilings.math.uni-bielefeld.de/substitution/fibonacci/. The substitution rule `L → LS, S → L` and Conway-worm interpretation.

[^penrose-rhomb]: Tilings Encyclopedia, *Penrose Rhomb (P3)*. Bielefeld University. https://tilings.math.uni-bielefeld.de/substitution/penrose-rhomb/. Inflation factor `φ`; Conway worms in P3 are Fibonacci-ordered.

[^aperiodic]: "Aperiodic tiling." *Wikipedia.* Quasicrystals were discovered by Dan Shechtman in 1982 (Nobel Prize in Chemistry, 2011); aperiodic tilings model their structure.

[^garcia2019]: Garcia-Argibay, M., Santed, M. A., & Reales, J. M. (2019). **Efficacy of binaural auditory beats in cognition, anxiety, and pain perception: a meta-analysis.** *Psychological Research*, 83, 357–372. https://doi.org/10.1007/s00426-018-1066-8. *g* = 0.45 across 22 studies.

[^basu2023]: Basu, S., & Banerjee, B. (2023). **Potential of binaural beats intervention for improving memory and attention: insights from meta-analysis and systematic review.** *Psychological Research*, 87, 951–963. https://doi.org/10.1007/s00426-022-01706-7. *g* = 0.40 across 15 studies / 31 effect sizes.

[^ingendoh2023]: Ingendoh, R. M., Posny, E. S., & Heine, A. (2023). **Binaural beats to entrain the brain? A systematic review of the effects of binaural beat stimulation on brain oscillatory activity, and the implications for psychological research and intervention.** *PLOS One*. https://doi.org/10.1371/journal.pone.0286023. Of 14 studies, 5 confirm the entrainment hypothesis at the EEG level, 8 contradict it, 1 mixed.

[^kanzler2021]: Aparecido-Kanzler, S. (2021). **Effects of binaural beats and isochronic tones on brain wave modulation: Literature review.** Reviews 17 RCTs scoring ≥3 on the Jadad scale.

---

*Source code: `github.com/arkanine1000/microtube`. The new sequence is at `src/presets.rs:JOURNEY_THROUGH_COSMOS_STEPS`; the parameter-automation logic is `App::update_sequence` in `src/app.rs`. The instrument is licensed under the Unlicense and is ~1 MB stripped. Headphones strongly recommended.*

*Acknowledgments: Roger Penrose, for the geometry, the impossible triangles, and the audacious claim that consciousness arises from quantum gravity. Stuart Hameroff, for taking that claim into the operating theatre. Douglas Hofstadter, for naming what we were all already feeling. John Conway, for the worms. J. S. Bach, for proving that counterpoint is the mathematics of the soul. Albert Einstein, for spacetime curvature, which the Solar Wind step is doing its best to evoke. M. C. Escher, for hands drawing hands.*

*The cosmos is the microtubule. The microtubule is the cosmos. **Ars gratia artis.***
