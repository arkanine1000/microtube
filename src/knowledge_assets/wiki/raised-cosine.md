# The Raised-Cosine Bell

The Shepard–Risset glissando is an audio illusion of a tone that climbs
forever. Its trick is a stack of seven octave-spaced sine oscillators
sweeping in parallel through a fixed amplitude window. When the highest
oscillator reaches the top of the window it wraps to the bottom — but the
window is **zero at the edges**, so the wrap is inaudible. The pitch
appears to rise without ever arriving.

The window MicroTube uses is a *raised cosine*, raised to the fourth power.

## The formula

For a log-frequency offset `x` in the range `[0, N]` where `N = 7`
octaves:

```
envelope(x) = sin(pi * x / N) ^ 4
```

That is, take a half-period of a sine wave from 0 to pi, square it, then
square again. The result is a smooth bell, zero at the edges, peak in the
middle.

## Why sin^4 and not sin or sin^2?

Three properties matter, and `sin^4` happens to satisfy all three:

1. **Edges are silent enough.** A bare `sin` has a *visible* slope at
   `x = 0`, and listeners hear a ghost of the wrap-around. `sin^2` is
   better. `sin^4` is better still — at `x = 0.05 * N`, the envelope is
   already down by 24 dB.

2. **Total energy is stable as the oscillators drift.** This is a property
   of the *seven* evenly-spaced oscillators *summed together*, not of any
   one of them. The sum of `sin^4(2*pi*i/N)` over `i = 0, 1, ..., N-1` is a
   constant — it does not depend on a global phase shift. So the layer
   does not get louder or quieter as it drifts.

3. **A single normalisation pre-computes the right loudness.** Because
   the energy is constant, MicroTube computes one normaliser at start-up:
   `1 / sqrt(sum of envelope^2)`, applies it to every output, and the
   layer stays at constant RMS for as long as it runs.

A bell that is too sharp (e.g. a Gaussian with small sigma) reduces audible
spectrum so much that the layer becomes a thin whistle. A bell that is
too soft (raw `sin`) leaks at the edges. `sin^4` is the locally-optimal
trade-off.

## The seven oscillators

MicroTube spaces seven sine oscillators evenly across the bell:

```
osc i:   log_offset = (i + drift) mod N        for i = 0 to 6
freq i:  f_min * 2 ^ log_offset                 where f_min ~ 32.7 Hz
```

The lowest possible oscillator frequency is C1 (32.7 Hz); the highest at
the top edge is `f_min * 2^N`, around C8. The *audible* span the listener
actually perceives is narrower because the bell tapers — most of the
energy lives within the central two octaves.

## Direction

`drift` is advanced at a fixed octaves-per-second rate. Positive drift
makes every oscillator climb, and a new oscillator fades in at the bottom
each time the highest one fades out at the top. Negative drift does the
opposite. The Studio app exposes `r` to toggle the layer on/off and `R`
to reverse direction.

In the Journey sequence, the direction flips from Up to Down for exactly
34 seconds at the **Singularity** epoch. For those 34 seconds the cosmos
is contracting; for the other 25 minutes it is expanding.
