//! Penrose Geometric Audio Mapping
//!
//! A Conway worm — a row of parallel thick/thin rhombs through a P3
//! Penrose tiling — has its tile sequence ordered as the Fibonacci word,
//! the canonical one-dimensional quasicrystal. We sample that sequence
//! and use it to drive emergence-engine spawn ratios.
//!
//! Substitution rule (Penrose / Fibonacci):
//!   L → L S
//!   S → L
//! Iterating yields  L, LS, LSL, LSLLS, LSLLSLSL, LSLLSLSLLSLLS, ...
//! whose limit is the infinite Fibonacci word L S L L S L S L L S L L S ...
//!
//! Cut-and-project equivalent (O(1) random access):
//!   s(k) = floor((k+2)/φ) - floor((k+1)/φ)
//!   tile(k) = Long if s(k) == 1 else Short
//!
//! In the Fibonacci word every S is bracketed by Ls, so SS never occurs.
//! Three pair types appear with asymptotic frequencies:
//!   LL  ≈ 1/φ³ ≈ 23.6%   — perfect fifth (3:2)   — the rare anchor
//!   LS  ≈ 1/φ² ≈ 38.2%   — major third  (5:4)   — descent
//!   SL  ≈ 1/φ² ≈ 38.2%   — perfect fourth (4:3) — ascent

const PHI: f64 = 1.618_033_988_749_895;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tile {
    Long,
    Short,
}

/// Fibonacci-word letter at position k via cut-and-project.
pub fn tile_at(k: usize) -> Tile {
    let kf = k as f64;
    let diff = ((kf + 2.0) / PHI).floor() - ((kf + 1.0) / PHI).floor();
    if diff as i64 == 1 { Tile::Long } else { Tile::Short }
}

/// Stateful walk along the Fibonacci word, advancing one tile at a time.
pub struct PenroseWalk {
    position: usize,
}

impl PenroseWalk {
    pub fn new() -> Self {
        Self { position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn current(&self) -> Tile {
        tile_at(self.position)
    }

    /// Advance one tile and return the (previous, current) pair after the move.
    /// The pair drives the harmonic-ratio choice for the next spawn.
    pub fn step(&mut self) -> (Tile, Tile) {
        let prev = self.current();
        self.position = self.position.wrapping_add(1);
        (prev, self.current())
    }

    /// The last `n` tiles ending at the current position (inclusive).
    /// Used by the visualization to draw a rhombus ribbon.
    pub fn recent(&self, n: usize) -> Vec<Tile> {
        if n == 0 {
            return Vec::new();
        }
        let span = n.saturating_sub(1);
        let start = self.position.saturating_sub(span);
        (start..=self.position).map(tile_at).collect()
    }
}

/// Map a (prev, curr) tile pair to a frequency ratio relative to the base.
///
/// SS is mathematically impossible in the Fibonacci word; the φ fallback
/// keeps the function total without ever firing in practice.
pub fn pair_ratio(prev: Tile, curr: Tile) -> f64 {
    match (prev, curr) {
        (Tile::Long, Tile::Long) => 3.0 / 2.0,
        (Tile::Long, Tile::Short) => 5.0 / 4.0,
        (Tile::Short, Tile::Long) => 4.0 / 3.0,
        (Tile::Short, Tile::Short) => PHI,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// First 13 letters of the Fibonacci word (substitution-derived):
    ///   L S L L S L S L L S L L S
    /// Verifies that the cut-and-project formula matches the canonical word.
    #[test]
    fn fibonacci_word_prefix_matches_substitution() {
        use Tile::*;
        let expected = [
            Long, Short, Long, Long, Short, Long, Short, Long, Long, Short, Long, Long, Short,
        ];
        for (k, &want) in expected.iter().enumerate() {
            assert_eq!(tile_at(k), want, "mismatch at position {k}");
        }
    }

    #[test]
    fn no_two_shorts_adjacent() {
        // The Fibonacci word never contains SS — verify across a long prefix.
        let mut prev = tile_at(0);
        for k in 1..1000 {
            let curr = tile_at(k);
            assert!(
                !(prev == Tile::Short && curr == Tile::Short),
                "SS appeared at {k}"
            );
            prev = curr;
        }
    }

    #[test]
    fn long_frequency_approaches_inverse_phi() {
        let n = 10_000;
        let longs = (0..n).filter(|&k| tile_at(k) == Tile::Long).count();
        let frac = longs as f64 / n as f64;
        let target = 1.0 / PHI;
        assert!((frac - target).abs() < 0.01, "frac {frac} vs target {target}");
    }
}
