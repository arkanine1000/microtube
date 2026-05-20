# The Fibonacci Word — A One-Dimensional Quasicrystal

Inside MicroTube's emergence engine is a small object that, depending on
where you stand, is a number-theoretic curiosity, a lattice in a
mathematical universe, or the structural skeleton of a real physical
material that won someone the Nobel Prize. It is called the **Fibonacci
word**.

This article is about what it is, where it comes from, and how MicroTube
uses it to spawn voices.

## The word itself

Start with a single letter `L`. Apply the substitution

```
L -> L S
S -> L
```

repeatedly, replacing every letter at once. You get:

```
step 1:  L
step 2:  L S
step 3:  L S L
step 4:  L S L L S
step 5:  L S L L S L S L
step 6:  L S L L S L S L L S L L S
...
```

The lengths of these words are the Fibonacci numbers: 1, 2, 3, 5, 8, 13.
That is not a coincidence — every `L` becomes two letters and every `S`
becomes one, so if there are `f_n` letters at step n, there will be
`f_n + (number of L's)` at step n+1. And the number of L's is `f_{n-1}`,
so `f_{n+1} = f_n + f_{n-1}`. The Fibonacci recurrence drops out for free.

Iterate forever and you reach the **infinite Fibonacci word**:

```
L S L L S L S L L S L L S L S L L S L S L L S L L S ...
```

It never repeats. It is also never random — every prefix is a prefix of
the next, and the substitution rule is deterministic. This is the
**canonical 1D quasicrystal**: aperiodic, but structured.

## Equivalent: cut-and-project

Computing the n-th letter via the substitution requires iterating until
the word is at least n letters long. There is a constant-time formula:

```
s(k) = floor((k+2) / phi) - floor((k+1) / phi)
tile(k) = Long if s(k) == 1 else Short
```

where phi = 1.618... is the golden ratio. This is a *cut-and-project*:
take the integer lattice Z^2, project it onto a line of irrational slope
`1/phi`, and read off the spacings. The resulting sequence is the
Fibonacci word.

This formula is what MicroTube actually uses. Random access in O(1).

## Three remarkable properties

1. **Self-similarity.** Apply the substitution to the infinite word and
   you get the infinite word back. Pattern at scale 1 and pattern at
   scale phi are identical.

2. **Pair frequencies are golden.** Look at consecutive pairs of letters
   in the limit:

   - LL appears with frequency 1/phi^3 ~ 23.6%
   - LS appears with frequency 1/phi^2 ~ 38.2%
   - SL appears with frequency 1/phi^2 ~ 38.2%
   - SS *never appears* — every S is bracketed by L's

3. **It lives inside a Penrose tiling.** Take a Penrose P3 tiling — the
   famous aperiodic tiling of the plane by two rhombs (fat and thin) —
   and mark a row of consecutive parallel rhombs. That row is called a
   *Conway worm*. The sequence of fat (L) and thin (S) rhombs along
   the worm is the Fibonacci word.

This last property is why the spawn mode is called Penrose. A 2D
quasicrystal contains 1D quasicrystals as cross-sections; we walk one of
those cross-sections to drive harmonic ratios.

## How MicroTube spawns voices

When the emergence engine is in Penrose mode, every spawn advances one
step along the Fibonacci word. The pair `(prev, curr)` of letters at the
current position picks a frequency ratio:

```
LL -> 3:2  (perfect fifth, the rare anchor)
LS -> 5:4  (major third, descent)
SL -> 4:3  (perfect fourth, ascent)
```

Because LL is rare and LS/SL are common, the ear hears mostly thirds and
fourths with occasional fifths anchoring the sequence. Because the word
is aperiodic, the rhythm of the harmonic moves never repeats. Because it
is self-similar, the music is recognisable at every timescale — large
patterns of moves echo small ones.

This is what MicroTube means by *Penrose-mode emergence*. It is what
consciousness might sound like, if Penrose and Hameroff turn out to be
right and the substrate of awareness is also a quasicrystal.

## A historical aside

Penrose tilings were proposed by Roger Penrose in 1974 as a mathematical
curiosity — an aperiodic tiling of the plane that nonetheless has
long-range order. In 1982, Dan Shechtman observed an *actual material* —
an aluminium-manganese alloy — whose X-ray diffraction pattern had
five-fold symmetry forbidden by classical crystallography. The
mathematical and the physical pictures merged. Shechtman won the Nobel
Prize in Chemistry in 2011.

So the spawn pattern in MicroTube's emergence engine is not just an
aesthetic gesture. The structure is the same one that organises real
matter in the same family of materials.
