//! The WebAssembly bridge.
//!
//! Compiled only under the `wasm` feature (which `wasm-pack` turns on).
//! [`WasmEngine`] is a thin `#[wasm_bindgen]` wrapper around [`Engine`]:
//! plain setters marshal parameter changes, and two scratch buffers let
//! JavaScript read rendered PCM straight out of the Wasm linear memory
//! without allocating a new array per audio quantum.

use wasm_bindgen::prelude::*;

use crate::emergence::SpawnMode;
use crate::engine::Engine;
use crate::shepard::Direction;
use crate::synth::{MistType, Timbre};

/// The engine, as seen from JavaScript / the `AudioWorklet`.
#[wasm_bindgen]
pub struct WasmEngine {
    engine: Engine,
    buf_l: Vec<f32>,
    buf_r: Vec<f32>,
}

#[wasm_bindgen]
impl WasmEngine {
    /// Build an engine for `sample_rate` Hz, with scratch buffers sized for
    /// blocks of up to `max_block` frames. Current `AudioWorklet` quanta are
    /// usually 128 frames, but browsers may vary this block size.
    #[wasm_bindgen(constructor)]
    pub fn new(sample_rate: f32, max_block: usize) -> WasmEngine {
        WasmEngine {
            engine: Engine::new(sample_rate as f64),
            buf_l: vec![0.0; max_block.max(1)],
            buf_r: vec![0.0; max_block.max(1)],
        }
    }

    /// Alternative named constructor mirroring the blueprint's `init`.
    pub fn init(sample_rate: f32, max_block: usize) -> WasmEngine {
        WasmEngine::new(sample_rate, max_block)
    }

    /// Grow the internal render buffers if a browser supplies larger blocks.
    /// This is only expected to run on rare block-size changes, not per render.
    pub fn ensure_capacity(&mut self, max_block: usize) {
        let max_block = max_block.max(1);
        if self.buf_l.len() < max_block {
            self.buf_l.resize(max_block, 0.0);
            self.buf_r.resize(max_block, 0.0);
        }
    }

    // --- Zero-copy rendering -------------------------------------------------
    //
    // `render` fills the internal buffers; JavaScript then constructs
    // `Float32Array` views over Wasm memory at `left_ptr` / `right_ptr`.
    // No array is allocated or copied across the JS boundary per quantum.

    /// Render `len` frames into the internal scratch buffers.
    pub fn render(&mut self, len: usize) {
        let len = len.min(self.buf_l.len());
        self.engine
            .process_block(&mut self.buf_l[..len], &mut self.buf_r[..len]);
    }

    /// Pointer to the left-channel scratch buffer inside Wasm linear memory.
    pub fn left_ptr(&self) -> *const f32 {
        self.buf_l.as_ptr()
    }

    /// Pointer to the right-channel scratch buffer inside Wasm linear memory.
    pub fn right_ptr(&self) -> *const f32 {
        self.buf_r.as_ptr()
    }

    /// Copying render path — fills caller-owned slices directly. Simpler for
    /// callers that do not want to manage memory views.
    pub fn process(&mut self, output_left: &mut [f32], output_right: &mut [f32]) {
        self.engine.process_block(output_left, output_right);
    }

    // --- Parameter setters ---------------------------------------------------

    pub fn set_playing(&mut self, playing: bool) {
        self.engine.targets_mut().playing = playing;
    }

    pub fn set_base_freq(&mut self, hz: f32) {
        self.engine.targets_mut().base_freq = hz;
    }

    pub fn set_beat_freq(&mut self, hz: f32) {
        self.engine.targets_mut().beat_freq = hz;
    }

    pub fn set_volume(&mut self, v: f32) {
        self.engine.targets_mut().volume = v.clamp(0.0, 1.0);
    }

    pub fn set_noise_level(&mut self, v: f32) {
        self.engine.targets_mut().noise_level = v.clamp(0.0, 1.0);
    }

    pub fn set_harmonics(&mut self, v: f32) {
        self.engine.targets_mut().harmonics = v.clamp(0.0, 1.0);
    }

    pub fn set_emergence(&mut self, v: f32) {
        self.engine.targets_mut().emergence = v.clamp(0.0, 1.0);
    }

    pub fn set_gravity(&mut self, v: f32) {
        self.engine.targets_mut().gravity = v.clamp(0.0, 1.0);
    }

    /// Convenience used by the blueprint — emergence on (1.0) or off (0.0).
    pub fn toggle_emergence(&mut self, on: bool) {
        self.engine.targets_mut().emergence = if on { 1.0 } else { 0.0 };
    }

    pub fn set_shepard(&mut self, v: f32) {
        self.engine.targets_mut().shepard = v.clamp(0.0, 1.0);
    }

    pub fn set_shepard_base_freq(&mut self, hz: f32) {
        self.engine.targets_mut().shepard_base_freq = hz;
    }

    pub fn set_mist_type(&mut self, mist: u32) {
        self.engine.targets_mut().mist_type = MistType::from_u32(mist);
    }

    pub fn set_timbre(&mut self, timbre: u32) {
        self.engine.targets_mut().timbre = Timbre::from_u32(timbre);
    }

    pub fn set_spawn_mode(&mut self, mode: u32) {
        self.engine.targets_mut().spawn_mode = SpawnMode::from_u32(mode);
    }

    pub fn set_shepard_direction(&mut self, dir: u32) {
        self.engine.targets_mut().shepard_direction = Direction::from_u32(dir);
    }
}
