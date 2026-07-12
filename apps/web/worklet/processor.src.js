// MicroTube AudioWorkletProcessor — source fragment.
//
// This file is NOT loaded directly. `scripts/build-worklet.mjs` concatenates
// the wasm-bindgen glue ahead of it and writes the combined, import-free
// `public/microtube-worklet/processor.js`. A worklet module with no nested
// `import` is the only shape that loads reliably across browsers — so
// `WasmEngine` / `initSync` below resolve from the prepended glue's scope.

// Current AudioWorklet quanta are usually 128 frames. Keep a larger initial
// Wasm buffer so render stays allocation-free in normal playback, while still
// allowing rare growth if a browser supplies larger blocks.
const INITIAL_RENDER_FRAMES = 2048;

class MicrotubeProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.engine = null;
    this.ready = false;
    this.renderCapacity = INITIAL_RENDER_FRAMES;
    this.memoryBuffer = null;
    this.leftPtr = 0;
    this.rightPtr = 0;
    this.viewFrames = 0;
    this.leftView = null;
    this.rightView = null;
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
          this.engine = new WasmEngine(sampleRate, INITIAL_RENDER_FRAMES);
          this.renderCapacity = INITIAL_RENDER_FRAMES;
          this.invalidateViews();
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

  invalidateViews() {
    this.memoryBuffer = null;
    this.leftPtr = 0;
    this.rightPtr = 0;
    this.viewFrames = 0;
    this.leftView = null;
    this.rightView = null;
  }

  ensureCapacity(frames) {
    if (frames <= this.renderCapacity) return;
    this.engine.ensure_capacity(frames);
    this.renderCapacity = frames;
    this.invalidateViews();
  }

  renderViews(frames) {
    const memoryBuffer = wasm.memory.buffer;
    const leftPtr = this.engine.left_ptr();
    const rightPtr = this.engine.right_ptr();
    if (
      this.leftView === null ||
      this.memoryBuffer !== memoryBuffer ||
      this.leftPtr !== leftPtr ||
      this.rightPtr !== rightPtr ||
      this.viewFrames !== frames
    ) {
      this.memoryBuffer = memoryBuffer;
      this.leftPtr = leftPtr;
      this.rightPtr = rightPtr;
      this.viewFrames = frames;
      this.leftView = new Float32Array(memoryBuffer, leftPtr, frames);
      this.rightView = new Float32Array(memoryBuffer, rightPtr, frames);
    }
    return { left: this.leftView, right: this.rightView };
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
      case 'gravity': e.set_gravity(value); break;
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
    const frames = out[0]?.length ?? 0;
    if (frames === 0) {
      return true;
    }

    // Render into Wasm-owned buffers, then fan out to whatever channels the
    // output bus exposes (channel 0 -> left, others -> right). This avoids
    // wasm-bindgen's per-quantum typed-array marshaling allocations.
    this.ensureCapacity(frames);
    this.engine.render(frames);
    const views = this.renderViews(frames);
    for (let ch = 0; ch < out.length; ch += 1) {
      out[ch].set(ch === 0 ? views.left : views.right);
    }
    return true;
  }
}

registerProcessor('microtube-processor', MicrotubeProcessor);
