// Parameter model shared by the React UI and the audio worklet.
//
// The enum orderings here mirror `microtube_core::synth` / `shepard` /
// `emergence` exactly — the worklet forwards these integers straight into
// the Wasm `set_*` calls, so they must not drift apart.

import {
  Anchor,
  AudioWaveform,
  CloudFog,
  Flame,
  Magnet,
  type LucideIcon,
  Radio,
  Sparkles,
  Volume2,
  Waves,
} from 'lucide-react';

export type Timbre = 0 | 1 | 2 | 3;
export type MistType = 0 | 1 | 2 | 3 | 4;
export type SpawnMode = 0 | 1 | 2;
export type Direction = 0 | 1;

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
  gravity: number;
  spawnMode: SpawnMode;
  shepard: number;
  shepardBase: number;
  shepardDirection: Direction;
  timbre: Timbre;
}

export type PresetSnapshot = Omit<MicroTubeState, 'playing'>;

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
  gravity: 0.5,
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
  | 'gravity'
  | 'noiseLevel'
  | 'shepard'
  | 'shepardBase'
  | 'volume';

/** A continuous slider-backed parameter. */
export interface SliderSpec {
  key: SliderKey;
  /** Glyph shown beside the label so a long list stays scannable. */
  icon: LucideIcon;
  min: number;
  max: number;
  step: number;
  /** Coarse step for the fine/coarse touch states. */
  coarse: number;
  unit: string;
  format: (v: number) => string;
  /**
   * True for a function that is genuinely on/off (mist, drift, emergence) —
   * its minimum is 0, and the UI recedes it while it sits there.
   */
  toggle?: boolean;
  /**
   * Multiplier between the stored value and the number a human types in the
   * slider's exact-entry form — 100 for 0..1 parameters shown as percentages.
   */
  displayScale?: number;
}

const pct = (v: number) => `${Math.round(v * 100)}%`;

const BASE_FREQ: SliderSpec = {
  key: 'baseFreq',
  icon: Radio,
  min: 50,
  max: 500,
  step: 1,
  coarse: 10,
  unit: 'Hz',
  format: (v) => `${v.toFixed(0)} Hz`,
};

const BEAT_FREQ: SliderSpec = {
  key: 'beatFreq',
  icon: AudioWaveform,
  min: 0.5,
  max: 100,
  step: 0.1,
  coarse: 1,
  unit: 'Hz',
  format: (v) => `${v.toFixed(1)} Hz`,
};

const WARMTH: SliderSpec = {
  key: 'harmonics',
  icon: Flame,
  min: 0,
  max: 1,
  step: 0.01,
  coarse: 0.1,
  unit: '',
  format: pct,
  displayScale: 100,
};

const NOISE: SliderSpec = {
  key: 'noiseLevel',
  icon: CloudFog,
  min: 0,
  max: 1,
  step: 0.01,
  coarse: 0.1,
  unit: '',
  format: pct,
  displayScale: 100,
  toggle: true,
};

const EMERGENCE: SliderSpec = {
  key: 'emergence',
  icon: Sparkles,
  min: 0,
  max: 1,
  step: 0.01,
  coarse: 0.1,
  unit: '',
  format: pct,
  displayScale: 100,
  toggle: true,
};

const GRAVITY: SliderSpec = {
  key: 'gravity',
  icon: Magnet,
  min: 0,
  max: 1,
  step: 0.01,
  coarse: 0.1,
  unit: '',
  format: pct,
  displayScale: 100,
};

const DRIFT_GAIN: SliderSpec = {
  key: 'shepard',
  icon: Waves,
  min: 0,
  max: 1,
  step: 0.01,
  coarse: 0.1,
  unit: '',
  format: pct,
  displayScale: 100,
  toggle: true,
};

const DRIFT_BASE: SliderSpec = {
  key: 'shepardBase',
  icon: Anchor,
  min: SHEPARD_BASE_MIN,
  max: SHEPARD_BASE_MAX,
  step: 0.1,
  coarse: 1,
  unit: 'Hz',
  format: (v) => `${v.toFixed(1)} Hz`,
};

export const SLIDERS: SliderSpec[] = [
  BASE_FREQ,
  BEAT_FREQ,
  WARMTH,
  EMERGENCE,
  GRAVITY,
  NOISE,
  DRIFT_GAIN,
  DRIFT_BASE,
];

/** Master volume — rendered separately in the header. */
export const VOLUME: SliderSpec = {
  key: 'volume',
  icon: Volume2,
  min: 0,
  max: 1,
  step: 0.01,
  coarse: 0.1,
  unit: '',
  format: pct,
  displayScale: 100,
};

export interface EegBand {
  id: EegBandId;
  greek: string;
  color: string;
}

export type EegBandId = 'delta' | 'theta' | 'alpha' | 'beta' | 'gamma';

export const EEG_BANDS: EegBand[] = [
  { id: 'delta', greek: 'δ', color: '#b478ff' },
  { id: 'theta', greek: 'θ', color: '#8c64ff' },
  { id: 'alpha', greek: 'α', color: '#50e6e6' },
  { id: 'beta', greek: 'β', color: '#50ff8c' },
  { id: 'gamma', greek: 'γ', color: '#ffdc50' },
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
