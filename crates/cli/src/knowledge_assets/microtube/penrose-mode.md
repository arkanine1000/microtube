# Penrose Mode

Penrose mode is named after Penrose tilings, but the engine uses one simple
slice of that world: a Conway worm.

A Conway worm is a row of fat and thin rhombs through a Penrose tiling. If you
write fat as L and thin as S, the row follows the Fibonacci word:

```text
L S L L S L S L L S L L S ...
```

That sequence is ordered but never periodic. It has structure without a short
repeat.

## Why MicroTube uses it

A repeating pattern can become predictable. Pure randomness can become mush.
The Fibonacci word sits between those extremes. It is deterministic, but it
keeps making fresh local combinations.

MicroTube reads consecutive pairs of letters:

- LL becomes a perfect fifth, 3:2.
- LS becomes a major third, 5:4.
- SL becomes a perfect fourth, 4:3.

SS does not appear in the Fibonacci word. Every short tile is surrounded by
long tiles.

## What it changes

Penrose mode changes how Emergence chooses new voice intervals. It does not
change the basic binaural beat by itself. The carrier still comes from Base and
Beat.

If Emergence is off, switching the spawn mode changes the future behavior but
does not immediately add voices.

## How it feels

Canon mode feels phrase-like. Penrose mode feels like a trail. It has recurring
shapes, but the exact path keeps moving.

That makes Penrose mode useful for long sequences, where a perfectly repeating
voice pattern would become too obvious.

## What to watch

In Emergence view, Penrose mode shows a worm ribbon. The letters are the recent
tile history. In the Penrose visualization, the same L/S idea appears as a
moving quasicrystal-like path.
