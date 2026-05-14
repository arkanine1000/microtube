# The Signal Path

MicroTube looks mysterious because several ideas move at once. Under the hood,
the sound follows a simple path.

Settings become tones. Tones become a stereo signal. Optional layers are mixed
in. A soft limiter keeps the final sound from becoming harsh. A small copy of
the audio is sent to the visualizers.

## The core carrier

The base frequency is the left-ear tone. The right-ear tone is the base plus
the beat.

If Base is 220 Hz and Beat is 10 Hz:

```text
left ear  = 220 Hz
right ear = 230 Hz
beat      = 10 Hz
```

This is the center of the whole program. Every other layer either colors this
carrier or moves around it.

## Tone color

A pure sine wave is clean, but it can feel clinical. Warmth adds harmonic
overtones above the carrier. Timbre decides which overtones are favored.

The important point: harmonic overtones are integer multiples of the carrier.
They change the color of the tone without changing the beat difference between
the ears.

## Mist

Mist is a quiet noise bed. Pink, white, brown, blue, and velvet mist all have
different shapes. Pink feels balanced. Brown leans low. Blue leans bright.
Velvet is sparse and grainy.

Mist is useful when a pure tone feels too exposed. A little texture can make
the beat easier to sit with.

## Emergence

Emergence adds small background voices that spawn at musical ratios around the
base tone. They fade in, live for a while, and fade out. Simple relationships
live longer than awkward ones.

Each voice is placed with lightweight HRTF-style cues before it is mixed into
the stereo output. This is not the main binaural beat. It is a little ecosystem
around the carrier.

## Drift

Drift is the Shepard-Risset layer. It creates the impression of a pitch that
rises forever or falls forever. It is mixed equally into both ears, so it does
not disturb the left-right beat.

Use it when you want movement without changing the main pulse.

## The limiter

When layers add together, peaks can get too high. MicroTube uses a soft clipper
instead of letting the signal hit a hard digital ceiling. That keeps loud
moments rounder and less brittle.

The limiter is not a license to turn everything up. It is a guardrail.
