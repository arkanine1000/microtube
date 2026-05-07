# Timbre Design

A pure sine wave has the cleanest binaural beat but the dullest sound.
Adding harmonic overtones — integer-multiple frequencies above the
fundamental — gives the tone *colour* without disturbing the difference
frequency. The four timbres MicroTube ships are not emulations of real
instruments; they are spectral profiles chosen for what they do *to* the
binaural beat.

Cycle them with `t` in the Studio tab.

## The four profiles

Each profile is a fixed list of weights for the first five harmonics:

```
                   2nd    3rd    4th    5th    6th
Organ            0.500  0.250  0.125  0.063  0.031
Flute            0.000  0.500  0.000  0.125  0.000
Bell             1.000  0.000  0.500  0.000  0.250
Saw              0.500  0.333  0.250  0.200  0.166
```

The fundamental (the 1st harmonic) always has weight 1.0. The Warmth
parameter scales the entire harmonic stack — at Warmth = 0 you get a
pure sine; at Warmth = 1 you get the profile in full.

## What each profile does

### Organ
Geometric decay: every harmonic is half the previous. This is the
classical *flue-pipe* spectrum. All harmonics present, all of them
quieter than the fundamental. The result is a warm, full sound that
does not draw attention to any specific overtone. **This is the default
timbre.**

### Flute
Only odd harmonics, and weakly. The 3rd and 5th are present; the 2nd, 4th,
and 6th are zero. Square-wave-ish but heavily attenuated. The character is
breathy and hollow — a flute is a stopped pipe and odd harmonics dominate.
Pairs well with airier mist textures (White, Blue).

### Bell
*Inharmonic* — but only via integer multiples, since MicroTube's harmonic
stack is integer-locked. The 2nd and 4th are present in unusual strengths
(weight 1.0 and 0.5 respectively, *equal to or louder than the
fundamental*); the 3rd and 5th are absent. The fundamental is therefore
not the dominant frequency, which gives the tone a metallic, slightly
struck quality. Used in the Microtubule and Stellar Bells epochs of the
Journey.

### Saw
Sawtooth-like: harmonics drop as 1/n. Bright, rich, edgy. The 6th is
still 17% as loud as the fundamental, which is much more than any other
profile. Used in the Cosmic Web and Singularity epochs of the Journey,
where the listener is asked to sit with something more aggressive.

## Why integer harmonics only?

Real bells, gamelans, and woodblocks have *inharmonic* overtones —
non-integer multiples that the ear hears as part of the same tone but
that no integer-locked phase accumulator can produce. MicroTube's
audio engine is built on integer-locked phase accumulators (the maths
of which is in the **Phase Accumulator** article) precisely to keep the
binaural beat *exact*. The cost is that bell-like inharmonic spectra are
out of reach. The Bell profile compensates by emphasising the 2nd and 4th
harmonics so the resulting waveform reads as struck, even though it is
strictly periodic.

## A note on phase

Each harmonic has its own phase accumulator and its own arbitrary starting
phase. The phases drift independently, which is what produces the slow
*shimmer* you hear when the harmonics are turned up — momentary
constructive and destructive interference between partials produces an
amplitude texture that is genuinely aperiodic.
