// MicroTube AudioWorkletProcessor — source fragment.
//
// This file is NOT loaded directly. `scripts/build-worklet.mjs` concatenates
// the wasm-bindgen glue ahead of it and writes the combined, import-free
// `public/microtube-worklet/processor.js`. A worklet module with no nested
// `import` is the only shape that loads reliably across browsers — so
// `WasmEngine` / `initSync` below resolve from the prepended glue's scope.

// One audio quantum is always 128 frames.
const QUANTUM = 128;

class MicrotubeProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.engine = null;
    this.ready = false;
    // Persistent render scratch — the engine always gets two valid 128-frame
    // buffers, regardless of how many channels the output bus exposes.
    this.scratchL = new Float32Array(QUANTUM);
    this.scratchR = new Float32Array(QUANTUM);
    this.calls = 0;
    this.diagSent = false;
    this.port.onmessage = (event) => this.handleMessage(event.data);
    // Diagnostic heartbeat: tells the main thread the processor was built
    // and its port is live, even before the engine is initialised.
    this.port.postMessage({ type: 'hello' });
  }

  handleMessage(msg) {
    switch (msg.type) {
      case 'init': {
        try {
          // The main thread transfers in the raw `.wasm` bytes; `initSync`
          // compiles + instantiates them synchronously, right here.
          // Newer wasm-bindgen wants `{ module }`; older takes it directly.
          try {
            initSync({ module: msg.wasm });
          } catch {
            initSync(msg.wasm);
          }
          this.engine = new WasmEngine(sampleRate, QUANTUM);
          if (msg.params) this.applyAll(msg.params);
          this.ready = true;
          this.port.postMessage({ type: 'ready' });
        } catch (err) {
          this.port.postMessage({ type: 'error', message: String(err) });
        }
        break;
      }
      case 'param':
        if (this.engine) this.setParam(msg.name, msg.value);
        break;
      case 'params':
        if (this.engine) this.applyAll(msg.value);
        break;
      default:
        break;
    }
  }

  applyAll(params) {
    for (const [name, value] of Object.entries(params)) {
      this.setParam(name, value);
    }
  }

  setParam(name, value) {
    const e = this.engine;
    switch (name) {
      case 'playing': e.set_playing(!!value); break;
      case 'baseFreq': e.set_base_freq(value); break;
      case 'beatFreq': e.set_beat_freq(value); break;
      case 'volume': e.set_volume(value); break;
      case 'noiseLevel': e.set_noise_level(value); break;
      case 'harmonics': e.set_harmonics(value); break;
      case 'emergence': e.set_emergence(value); break;
      case 'shepard': e.set_shepard(value); break;
      case 'shepardBase': e.set_shepard_base_freq(value); break;
      case 'mistType': e.set_mist_type(value >>> 0); break;
      case 'timbre': e.set_timbre(value >>> 0); break;
      case 'spawnMode': e.set_spawn_mode(value >>> 0); break;
      case 'shepardDirection': e.set_shepard_direction(value >>> 0); break;
      default: break;
    }
  }

  process(_inputs, outputs) {
    const out = outputs[0];
    if (!this.ready || !out || out.length === 0) {
      // Keep the node alive even before the engine is wired up.
      return true;
    }

    // Render into our own buffers, then fan out to whatever channels the
    // output bus actually has (channel 0 -> left, others -> right).
    this.engine.process(this.scratchL, this.scratchR);
    for (let ch = 0; ch < out.length; ch += 1) {
      out[ch].set(ch === 0 ? this.scratchL : this.scratchR);
    }

    // One-shot diagnostic once the engine has had time to ramp up.
    this.calls += 1;
    if (!this.diagSent && this.calls === 50) {
      this.diagSent = true;
      let peak = 0;
      for (let i = 0; i < this.scratchL.length; i += 1) {
        const a = Math.abs(this.scratchL[i]);
        if (a > peak) peak = a;
      }
      this.port.postMessage({
        type: 'diag',
        channels: out.length,
        frames: out[0].length,
        enginePeak: peak,
      });
    }
    return true;
  }
}

registerProcessor('microtube-processor', MicrotubeProcessor);
