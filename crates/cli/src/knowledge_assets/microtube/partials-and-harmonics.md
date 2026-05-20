# Partials And The Harmonics View

Two places in the studio talk about the same thing: the **Partials** panel in
the instrument column, and the **Harmonics** visualization. Both are showing
you the overtones the tone-and-warmth engine is currently producing.

## What a partial is

A partial is one of the sine waves that, when summed, make up the tone you
hear. The lowest partial is called the **fundamental**. Every partial above it
sits at an integer multiple of the fundamental's frequency.

MicroTube exposes six partials per channel: **H1** through **H6**.

| Label | Multiple   | Musical interval above the root        |
|-------|------------|----------------------------------------|
| H1    | f × 1      | the root itself (the fundamental)      |
| H2    | f × 2      | one octave up                          |
| H3    | f × 3      | one octave + perfect fifth             |
| H4    | f × 4      | two octaves up                         |
| H5    | f × 5      | two octaves + major third              |
| H6    | f × 6      | two octaves + perfect fifth            |

So if `base` reads 220 Hz, **H1** is at 220, **H2** at 440, **H3** at 660, and
so on. The same series exists on the right channel, transposed up by the beat
frequency.

## What the Partials panel shows

Each row in the Partials section of the instrument column is one partial. From
left to right:

- the **`H#`** chip (brighter when the partial is loud)
- a short label noting the interval (`root`, `oct`, `5th`, etc.)
- a gradient bar showing that partial's current contribution, normalized so
  the loudest partial fills the bar

The bars are not absolute amplitudes — they are relative to whichever partial
is loudest right now. That keeps the picture readable as `warm` changes.

The last row reports the active **timbre** (Organ / Flute / Bell / Saw). The
timbre chooses the recipe of weights that the `warm` parameter scales into the
mix; H1 always carries the fundamental, and the timbre says how much of each
overtone to layer on top.

## What the Harmonics visualization adds

The Harmonics view in the audiovisual stage shows the same six partials in a
different frame:

- a live **stereo phase trace** in the center — left vs. right amplitude
  plotted as a Lissajous-like curve, so you can *see* the binaural beat as a
  slowly rotating figure
- a ring of **partial nodes**, labeled `1` through `6`, whose brightness
  follows their current level
- a **partial floor** along the bottom mirroring the panel's bars, again
  labeled `H1`-`H6`

Use Partials when you want a quick, calm readout of the spectrum. Use the
Harmonics view when you want to *watch* the binaural relationship in motion as
well — that is the only view where the phase trace shows up.

## Why the numbers matter

Choosing a timbre is choosing which partials get a voice. Choosing `warm` is
choosing how loud their voice is. Those two controls together account for
nearly every "this tone feels right / wrong" judgment you will make.

When the sound goes harsh, glance at the Partials panel: if H4-H6 are tall and
H1-H3 are stubby, you are hearing too much overtone for too little
fundamental. Lower `warm` or step toward a darker timbre.

When the sound goes thin, the opposite: H1 dominates and the others barely
register. Raise `warm` or switch to a richer timbre to bring color back.
