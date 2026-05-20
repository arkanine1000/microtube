# Binaural Beats

When two pure tones of slightly different frequency are presented one to
each ear over headphones, the auditory brainstem perceives a phantom third
tone — a **beat** — at the difference frequency. A 220 Hz left and a 230 Hz
right produce a 10 Hz beat, which falls inside the Alpha band of human
EEG.

This is *not* the same beat you hear when two tones combine in air. That
acoustic beat is produced by physical wave summation; the binaural beat is
produced inside the head, in the superior olivary complex, where signals
from the two ears first meet. **Speakers destroy the effect** because the
two channels mix in air before reaching your ears.

## The math

For a left-ear tone `sin(2*pi*f_L*t)` and a right-ear tone
`sin(2*pi*f_R*t)`, the difference frequency is simply `df = f_R - f_L`.
The brain locks onto `df` as a pulse rate. MicroTube's two oscillators
run as

- left:  `sin(2*pi * base * t)`
- right: `sin(2*pi * (base + beat) * t)`

and the **base frequency** is chosen low enough that the perceived beat is
clean and musical. 220 Hz (just above middle A) is a comfortable default;
the Journey sequence pulls it as low as 55 Hz at the singularity epoch.

## The brainwave-entrainment hypothesis

The popular claim is that the brain's dominant frequency *synchronises*
with `df`. The honest answer reading the literature: the evidence is
mixed.

- Garcia-Argibay et al. (2019) meta-analysed 22 studies and found a medium,
  significant effect (g = 0.45) on cognition, anxiety, and pain.
- Basu and Banerjee (2023) extended this to memory and attention with
  g = 0.40 across 31 effect sizes.
- Ingendoh, Posny, and Heine (2023) examined fourteen studies of EEG-level
  entrainment specifically. Five confirmed it, eight contradicted it, one
  was mixed. They declined to meta-analyse owing to methodological
  heterogeneity.

So binaural beats *probably* do something — the behavioural meta-analytic
effect is real — but the specific mechanism may be cross-frequency
coupling, attention narrowing, or the meditative effect of sustained tones
rather than literal entrainment of brainwave frequency.

## How MicroTube uses them

The five quick-presets each pin `df` to a centroid of one EEG band:

- **Deep Sleep** — 2 Hz Delta
- **Meditation** — 6 Hz Theta
- **Relaxation** — 10 Hz Alpha
- **Focus** — 18 Hz Beta
- **Flow State** — 40 Hz Gamma

Sequences interpolate `df` over time, sweeping the listener through bands
on a chosen narrative arc. The Journey sequence traces a U from 40 Hz down
to 2 Hz and back, mirroring its cosmic-zoom-out followed by compression
back into a microtubule.

## Practical notes

- Volume should be just loud enough to hear comfortably. Loud beats are
  not more effective; they are more fatiguing.
- The first minute is acclimatisation. The effect, if any, builds over
  about ten minutes.
- Stop if you feel disoriented. The sensation of mild dissociation is
  common; sharp discomfort is not.
