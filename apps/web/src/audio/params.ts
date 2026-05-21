// Parameter model shared by the React UI and the audio worklet.
//
// The enum orderings here mirror `microtube_core::synth` / `shepard` /
// `emergence` exactly — the worklet forwards these integers straight into
// the Wasm `set_*` calls, so they must not drift apart.

export type Timbre = 0 | 1 | 2 | 3;
export type MistType = 0 | 1 | 2 | 3 | 4;
export type SpawnMode = 0 | 1;
export type Direction = 0 | 1;

export const TIMBRES = ['Organ', 'Flute', 'Bell', 'Saw'] as const;
export const MISTS = ['Pink', 'White', 'Brown', 'Blue', 'Velvet'] as const;
export const SPAWN_MODES = ['Canon', 'Penrose'] as const;
export const DIRECTIONS = ['Rising', 'Falling'] as const;

/** The full live engine state the UI mirrors. */
export interface MicroTubeState {
  playing: boolean;
  baseFreq: number;
  beatFreq: number;
  volume: number;
  noiseLevel: number;
  mistType: MistType;
  harmonics: number;
  emergence: number;
  spawnMode: SpawnMode;
  shepard: number;
  shepardBase: number;
  shepardDirection: Direction;
  timbre: Timbre;
}

/** Matches `microtube_core::engine::Params::default()` + the CLI's startup. */
export const DEFAULT_STATE: MicroTubeState = {
  playing: true,
  baseFreq: 220,
  beatFreq: 10,
  volume: 0.5,
  noiseLevel: 0,
  mistType: 2, // Brown
  harmonics: 0.3,
  emergence: 0,
  spawnMode: 0, // Canon
  shepard: 0,
  shepardBase: 32.70319566257483,
  shepardDirection: 1, // Falling
  timbre: 0, // Organ
};

/** Shepard base-frequency range — C0..C3 around DEFAULT_BASE_FREQ_HZ. */
export const SHEPARD_BASE_MIN = 32.70319566257483 * 0.5;
export const SHEPARD_BASE_MAX = 32.70319566257483 * 4;

/** Keys of `MicroTubeState` that a continuous slider drives. */
export type SliderKey =
  | 'baseFreq'
  | 'beatFreq'
  | 'harmonics'
  | 'emergence'
  | 'noiseLevel'
  | 'shepard'
  | 'shepardBase'
  | 'volume';

/** A continuous slider-backed parameter. */
export interface SliderSpec {
  key: SliderKey;
  label: string;
  hint: string;
  min: number;
  max: number;
  step: number;
  /** Coarse step for the fine/coarse touch states. */
  coarse: number;
  unit: string;
  format: (v: number) => string;
}

const pct = (v: number) => `${Math.round(v * 100)}%`;

export const SLIDERS: SliderSpec[] = [
  {
    key: 'baseFreq',
    label: 'Base frequency',
    hint: 'Carrier pitch of the binaural pair',
    min: 50,
    max: 500,
    step: 1,
    coarse: 10,
    unit: 'Hz',
    format: (v) => `${v.toFixed(0)} Hz`,
  },
  {
    key: 'beatFreq',
    label: 'Beat frequency',
    hint: 'L/R offset — sets the EEG band',
    min: 0.5,
    max: 100,
    step: 0.1,
    coarse: 1,
    unit: 'Hz',
    format: (v) => `${v.toFixed(1)} Hz`,
  },
  {
    key: 'harmonics',
    label: 'Warmth',
    hint: 'Harmonic partials mixed into the carrier',
    min: 0,
    max: 1,
    step: 0.01,
    coarse: 0.1,
    unit: '',
    format: pct,
  },
  {
    key: 'emergence',
    label: 'Emergence',
    hint: 'Generative canon / quasicrystal voices',
    min: 0,
    max: 1,
    step: 0.01,
    coarse: 0.1,
    unit: '',
    format: pct,
  },
  {
    key: 'noiseLevel',
    label: 'Noise',
    hint: 'Ambient coloured-noise mist layer',
    min: 0,
    max: 1,
    step: 0.01,
    coarse: 0.1,
    unit: '',
    format: pct,
  },
  {
    key: 'shepard',
    label: 'Drift gain',
    hint: 'Shepard-Risset endless-glissando level',
    min: 0,
    max: 1,
    step: 0.01,
    coarse: 0.1,
    unit: '',
    format: pct,
  },
  {
    key: 'shepardBase',
    label: 'Drift base',
    hint: 'Lowest oscillator in the Shepard stack',
    min: SHEPARD_BASE_MIN,
    max: SHEPARD_BASE_MAX,
    step: 0.1,
    coarse: 1,
    unit: 'Hz',
    format: (v) => `${v.toFixed(1)} Hz`,
  },
];

/** Master volume — rendered separately in the transport bar. */
export const VOLUME: SliderSpec = {
  key: 'volume',
  label: 'Master volume',
  hint: 'Overall output level',
  min: 0,
  max: 1,
  step: 0.01,
  coarse: 0.1,
  unit: '',
  format: pct,
};

export interface EegBand {
  name: string;
  greek: string;
  blurb: string;
  color: string;
}

export const EEG_BANDS: EegBand[] = [
  { name: 'Delta', greek: 'δ', blurb: 'deep sleep', color: '#b478ff' },
  { name: 'Theta', greek: 'θ', blurb: 'meditation', color: '#8c64ff' },
  { name: 'Alpha', greek: 'α', blurb: 'calm focus', color: '#50e6e6' },
  { name: 'Beta', greek: 'β', blurb: 'alertness', color: '#50ff8c' },
  { name: 'Gamma', greek: 'γ', blurb: 'peak insight', color: '#ffdc50' },
];

/** Index into EEG_BANDS for a given beat frequency. Mirrors `freq_band_name`. */
export function eegBandIndex(beat: number): number {
  if (beat < 4) return 0;
  if (beat < 8) return 1;
  if (beat < 13) return 2;
  if (beat < 30) return 3;
  return 4;
}

export const clamp = (v: number, lo: number, hi: number) =>
  Math.min(hi, Math.max(lo, v));
