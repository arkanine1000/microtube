# Phase Accumulators

MicroTube is built around the most boring oscillator design in the
literature, for the most important reason: **a phase accumulator does not
drift**. Run it for ten seconds, ten minutes, or ten hours; the frequency
it produces is exactly what you set, to within the limits of floating-point
arithmetic.

This article explains why.

## The naïve approach

Suppose you want a 440 Hz sine wave at a 48 000 Hz sample rate. You
might write:

```rust
let phase_step = 440.0 / 48_000.0;
let mut phase = 0.0;
loop {
    let sample = (2.0 * PI * phase).sin();
    output(sample);
    phase = (phase + phase_step) % 1.0;
}
```

That is a phase accumulator — and it is, in fact, what MicroTube uses,
modulo a few wrinkles below.

## What goes wrong without it

You might think you can use the *time* directly:

```rust
let mut t = 0.0;
loop {
    let sample = (2.0 * PI * 440.0 * t).sin();
    output(sample);
    t += 1.0 / 48_000.0;
}
```

This works for a few seconds. After ten minutes, `t` is around 600
seconds. Multiplying 600 by `2 * pi * 440` gives a number with eight
significant digits to the left of the decimal point; the four bits at
the bottom of the f64 significand are stomped, and the wave starts to
*phase-drift* relative to wall-clock. Worse, if you ever change the
frequency (as MicroTube does on every parameter tweak), there is no
clean way to do it without a discontinuity — `t` is shared between
old and new frequencies, and the phase you compute mid-stream depends
on which frequency was active at which time.

The accumulator approach has neither problem. The phase is bounded:
`% 1.0` keeps it in `[0, 1)` forever. And the *step* depends only on
the current frequency, so changing frequency is changing one number;
the accumulated phase carries through unbroken.

## What MicroTube actually does

The audio thread maintains separate phase accumulators for:

- the left-ear binaural carrier
- the right-ear binaural carrier
- five harmonic overtones for each carrier
- seven Shepard-Risset oscillators (each with its own log-frequency
  offset that is itself a kind of phase accumulator)
- up to twelve emergence voices

Every accumulator advances by `frequency / sample_rate` per sample, then
wraps with `phase -= phase.floor()`. Floor-wrap is preferred over modulo
for performance: on most architectures `floor()` is a single instruction.

The frequency itself is not constant — parameters arrive from the UI
thread via lock-free `AtomicU32`s — but each new frequency is read once
per buffer and then exponentially smoothed with a 50 ms time constant.
So the *step size* moves smoothly even when the user mashes `H` and
yanks the parameter from 220 Hz to 500 Hz in a single keystroke. The
phase keeps accumulating from wherever it was; the wave never jumps.

## Why this matters for the binaural beat

The whole experience depends on the **left and right phases staying
locked to the difference frequency**. Any drift between them muddies the
beat. With phase accumulators, the lock is exact: each accumulator
advances by `f/sr` per sample, and the difference between them advances
by `(fᵣ - fₗ)/sr` per sample, identically forever.

A small implementation note: the accumulators are `f64`, not `f32`. With
`f32`, accumulating 48 000 increments per second eventually produces
visible quantisation in the wrap-around — about half a cent of pitch
error. With `f64`, the error stays in the 16th decimal place even after
hours. The cost is negligible.

## The limit

There is a limit. After about `2^53 / sample_rate ≈ 187 years`, the f64
significand finally runs out of bits. We will cross that bridge if we
must.
