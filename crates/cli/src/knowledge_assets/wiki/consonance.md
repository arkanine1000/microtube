# The Consonance Score

MicroTube's emergence engine spawns voices at frequency ratios and lets
the more *consonant* ones live longer. The notion of consonance used here
is operational — a small numerical function that takes a ratio and returns
a score from 0 (dissonant) to roughly 1 (perfect).

The function lives in `crates/core/src/emergence.rs` and is reused,
byte-for-byte, by any other code in the project that needs it. There is one
source of truth.

## The formula

Given a ratio *r*, score it against a fixed table of simple ratios:

```
ratio       peak_score
1/1   unison         1.00
2/1   octave         0.95
3/2   perfect fifth  0.90
4/3   perfect fourth 0.85
5/4   major third    0.80
6/5   minor third    0.75
9/8   major second   0.68
16/15 minor second   0.62
phi   golden ratio   0.70
phi^2 golden square  0.65
```

For each table entry *(r_i, peak_i)*, compute the **logarithmic distance**
`d_i = |ln(r / r_i)|`. If `d_i < 0.1`, the entry contributes a triangular
kernel:

```
contribution = peak_i * (1 - d_i / 0.1)
```

The score is the **maximum** contribution across the table. Outside any
0.1-radius neighborhood, the score is zero.

## Why log distance?

Pitch perception is logarithmic — an octave is a doubling, a fifth is a
ratio of 3:2, and so on. Linear distance would punish ratios above 1
unfairly: 1.5 vs. 1.6 differ by 0.1 in linear distance, but so do 0.5 vs.
0.4. In log space those are equivalent musical intervals away from their
neighbour. `ln(r / r_i)` symmetrises the metric.

## Why 0.1?

A log-distance of 0.1 corresponds to roughly **10%** in frequency space —
about one and a half semitones. That is the auditory width within which
the ear still hears a ratio as a *deformed* version of the simple one. Go
further and you fall into the no-man's-land between intervals, where
nothing scores.

## Why these particular peaks?

The first eight are classical just-intonation consonances and stepwise intervals,
with peak scores ordered by their position in the harmonic series and by how
stable they sound as sustained background voices. The next two are the golden
ratio phi = 1.618... and phi squared. Phi is by construction the *most*
irrational number — its continued-fraction expansion is `[1; 1, 1, 1, ...]`.
A ratio of phi is mathematically the *least* approximable by a small
fraction, which makes it audibly *strange* but not dissonant. Its score
sits below the simple ratios deliberately: it is consonance with an
asterisk.

## How the audio engine uses it

When a voice spawns, its lifetime is partly weighted by its consonance
score. Pure 3:2 voices get longer lifetimes; voices on awkward ratios
fade quickly. Energy conservation enforces a cap on total amplitude, so
the system has the property that **simple ratios live, complex ratios
die** — a Darwinian filter expressed in ten lines of code. Fuxian mode also
uses this same table as its interval pool before applying leap, parallel, and
gravity constraints.
