# Emergence

Emergence is a small generative voice system that grows around the main
carrier. It is not random decoration. It follows a few simple rules.

Voices spawn at musical ratios. They fade in, live for a while, and fade out.
Consonant relationships last longer. Too many voices are kept under control so
the carrier does not disappear.

## The root

The engine begins with a root voice at the base frequency. New voices branch
from the strongest live voice. That makes the constellation behave like a
family tree instead of a pile of unrelated notes.

Each voice has:

- An interval from the root.
- A trajectory from its parent.
- A generation number.
- A stereo pan position.
- A binaural spatial position.
- A smooth lifetime envelope.
- The same timbre flavor as the carrier.

The spatial position uses a built-in HRTF-style approximation. The engine adds
tiny left-right timing differences, level differences, and far-ear softening.
It is designed for headphones and real-time safety; it does not use measured
HRIR files or convolution.

## Canon mode

Canon mode follows a repeating pattern of musical moves: fifths, thirds,
fourths, octaves, and golden-ratio turns. Every few spawns, the pattern shifts
register, like a phrase answering itself from somewhere else.

Canon mode tends to feel intentional and musical.

## Penrose mode

Penrose mode uses the Fibonacci word to choose the next move. Consecutive tile
pairs choose one of three intervals:

- LL chooses a perfect fifth.
- LS chooses a major third.
- SL chooses a perfect fourth.

The pattern never settles into a simple loop, but it also never becomes pure
noise.

## Fuxian mode

Fuxian mode uses the same consonance pool, but it listens to the recent motion
of the strongest living voice before choosing the next interval.

If the parent made a leap larger than a minor third, the next voice must answer
with a step in the opposite direction. If the parent is already sitting on a
perfect fifth or octave, Fuxian avoids spawning the same perfect interval again.

Gravity controls how strongly Fuxian pulls the next choice back toward the
root. Low gravity leaves more room for wandering. High gravity makes the line
resolve toward 1:1 more often.

## What the Emergence view shows

The Emergence view is a constellation. Nodes are voices. Lines connect voices
with simple relationships. Brightness follows amplitude. Generation changes
the node shape. The status line shows mode, total energy, voice count,
generation depth, and epoch.

In Penrose mode, the view also shows a small worm ribbon: the recent L/S tile
history that is driving the spawn pattern.

## How to use it

Turn emergence on with `e`. Start around 30 to 50 percent. If the sound becomes
too busy, reduce emergence before changing everything else. It is meant to be a
living layer, not a replacement for the carrier.
