# Tone And Warmth

The simplest binaural beat is two pure sine waves. That is clean and precise,
but it can also feel bare. Warmth gives the tone a body.

Warmth adds harmonic overtones. These are higher tones that sit at integer
multiples of the carrier: 2x, 3x, 4x, 5x, and 6x. Because they are tied to the
carrier, they color the sound without changing the beat speed.

## Timbre

Timbre is the shape of the overtone mix.

Organ uses a smooth falling stack. Each overtone is weaker than the one before
it. It is the warm default.

Flute favors odd harmonics. It can feel hollow, airy, and less dense.

Bell emphasizes a few bright partials. It is not a realistic bell model, but it
has a struck, glassy character.

Saw keeps many partials alive. It is brighter and more forward.

## Warmth

Warmth controls how much of the selected timbre is mixed in.

At zero warmth, you hear a pure carrier. At high warmth, the timbre profile
becomes much more obvious. The beat is still the difference between left and
right carrier frequencies; warmth changes the color, not the beat number.

## What the Harmonics view shows

The Harmonics visualization shows the live stereo phase trace and the H1-H6
partial stack. H1 is the fundamental. H2 through H6 are the overtones. The
little meters show how much each partial contributes after the engine's
normalization.

This is more useful than a generic spectrum because it matches what MicroTube
actually generates.

## Listening advice

Use less warmth for sleepier or cleaner sessions. Use more warmth when the tone
feels too thin or when you want the sound to behave more like an instrument.

If the sound becomes sharp, lower warmth before lowering the beat. The beat may
be fine; the tone color may simply be too bright.
