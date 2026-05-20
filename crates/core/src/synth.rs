//! Timbre / mist enums, the noise-colour generators, and the soft limiter.
//!
//! These were previously inlined in the CLI's audio callback; lifting them
//! here lets the web build share the exact same texture and gain staging.

/// Harmonic timbre profile — a fixed weight vector over partials 2..=6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Timbre {
    Organ = 0,
    Flute = 1,
    Bell = 2,
    Saw = 3,
}

impl Timbre {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::Flute,
            2 => Self::Bell,
            3 => Self::Saw,
            _ => Self::Organ,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Organ => Self::Flute,
            Self::Flute => Self::Bell,
            Self::Bell => Self::Saw,
            Self::Saw => Self::Organ,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Organ => "Organ",
            Self::Flute => "Flute",
            Self::Bell => "Bell",
            Self::Saw => "Saw",
        }
    }

    /// Relative amplitude of partials 2..=6 (the fundamental is implicit 1.0).
    pub fn weights(self) -> [f64; 5] {
        match self {
            Self::Organ => [0.5, 0.25, 0.125, 0.0625, 0.03125],
            Self::Flute => [0.0, 0.5, 0.0, 0.125, 0.0],
            Self::Bell => [1.0, 0.0, 0.5, 0.0, 0.25],
            Self::Saw => [0.5, 0.333, 0.25, 0.2, 0.166],
        }
    }
}

/// Noise-colour profile for the ambient "mist" layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MistType {
    Pink = 0,
    White = 1,
    Brown = 2,
    Blue = 3,
    Velvet = 4,
}

impl MistType {
    pub fn from_u32(value: u32) -> Self {
        match value {
            1 => Self::White,
            2 => Self::Brown,
            3 => Self::Blue,
            4 => Self::Velvet,
            _ => Self::Pink,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Pink => Self::White,
            Self::White => Self::Brown,
            Self::Brown => Self::Blue,
            Self::Blue => Self::Velvet,
            Self::Velvet => Self::Pink,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pink => "Pink",
            Self::White => "White",
            Self::Brown => "Brown",
            Self::Blue => "Blue",
            Self::Velvet => "Velvet",
        }
    }

    pub fn texture(self) -> &'static str {
        match self {
            Self::Pink => "warm",
            Self::White => "air",
            Self::Brown => "surf",
            Self::Blue => "glass",
            Self::Velvet => "sparks",
        }
    }
}

/// Per-mist output gain so every colour sits at a comparable loudness.
#[inline]
pub fn mist_gain(mist_type: MistType) -> f64 {
    match mist_type {
        MistType::Pink => 0.30,
        MistType::White => 0.24,
        MistType::Brown => 0.22,
        MistType::Blue => 0.20,
        MistType::Velvet => 0.26,
    }
}

/// Soft clipper using `tanh` — prevents harsh digital clipping.
#[inline]
pub fn soft_clip(x: f64) -> f64 {
    if x.abs() < 0.9 {
        x // Fast path: no processing needed for normal levels
    } else {
        x.tanh()
    }
}

/// Stateful generator for the five noise colours.
pub struct NoiseGen {
    rng: u64,
    pink_state: [f64; 7],
    pink_counter: u32,
    brown_state: f64,
    last_white: f64,
    last_white_2: f64,
    velvet_state: f64,
}

impl NoiseGen {
    pub fn new() -> Self {
        Self {
            rng: 0xDEAD_BEEF_CAFE_1234,
            pink_state: [0.0; 7],
            pink_counter: 0,
            brown_state: 0.0,
            last_white: 0.0,
            last_white_2: 0.0,
            velvet_state: 0.0,
        }
    }

    fn xorshift64(&mut self) -> f64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng as f64) / (u64::MAX as f64) * 2.0 - 1.0
    }

    fn pink_noise(&mut self) -> f64 {
        self.pink_counter = self.pink_counter.wrapping_add(1);
        let mut sum = 0.0;
        for i in 0..7 {
            if self.pink_counter & (1 << i) == 0 {
                self.pink_state[i] = self.xorshift64();
            }
            sum += self.pink_state[i];
        }
        sum / 7.0
    }

    /// One noise sample of the requested colour, before [`mist_gain`].
    pub fn sample(&mut self, mist_type: MistType) -> f64 {
        match mist_type {
            MistType::Pink => self.pink_noise(),
            MistType::White => self.xorshift64() * 0.58,
            MistType::Brown => {
                let white = self.xorshift64();
                self.brown_state = (self.brown_state * 0.996 + white * 0.035).clamp(-1.0, 1.0);
                self.brown_state * 1.4
            }
            MistType::Blue => {
                let white = self.xorshift64();
                let blue = white - self.last_white * 0.72 + self.last_white_2 * 0.12;
                self.last_white_2 = self.last_white;
                self.last_white = white;
                blue * 0.42
            }
            MistType::Velvet => {
                let trigger = (self.xorshift64() + 1.0) * 0.5;
                if trigger < 0.0025 {
                    self.velvet_state = if self.xorshift64() >= 0.0 { 1.0 } else { -1.0 };
                }
                let sample = self.velvet_state;
                self.velvet_state *= 0.88;
                sample * 0.75
            }
        }
    }
}

impl Default for NoiseGen {
    fn default() -> Self {
        Self::new()
    }
}
