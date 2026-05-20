#![allow(dead_code)]
//! MicroTube core — the pure DSP engine.
//!
//! This crate has no terminal- or browser-specific code. It is linked
//! directly by the native CLI (`rlib`) and compiled to WebAssembly by
//! `wasm-pack` (`cdylib`) for the web app. The two front-ends share one
//! sample-accurate synthesis path: phase accumulators, exponential
//! parameter smoothing, the emergence engine, the Shepard-Risset drift,
//! the Fibonacci-word quasicrystal, the noise-colour generators, and the
//! soft limiter all live here.

pub mod emergence;
pub mod engine;
pub mod penrose;
pub mod shepard;
pub mod synth;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use engine::{Engine, Params};
pub use synth::{MistType, Timbre};
