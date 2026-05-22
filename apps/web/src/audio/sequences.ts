// Presets and executable web sequences.
//
// Ported from `crates/cli/src/presets.rs`. Legacy sequences automate only the
// binaural pair; narrative sequences can opt into extra sound-shaping fields.

import type {
  Direction,
  MicroTubeState,
  MistType,
  SpawnMode,
  Timbre,
} from './params';

export interface Preset {
  beatFreq: number;
  baseFreq: number;
  noiseLevel: number;
}

export const PRESETS: Preset[] = [
  { beatFreq: 2, baseFreq: 180, noiseLevel: 0.15 },
  { beatFreq: 6, baseFreq: 200, noiseLevel: 0.1 },
  { beatFreq: 10, baseFreq: 220, noiseLevel: 0 },
  { beatFreq: 18, baseFreq: 250, noiseLevel: 0 },
  { beatFreq: 40, baseFreq: 300, noiseLevel: 0 },
];

export type SequenceId =
  | 'deep-focus'
  | 'wake-up'
  | 'power-nap'
  | 'deep-meditation'
  | 'orch-or'
  | 'journey-through-cosmos';

export interface SequenceStep {
  beatFreq: number;
  baseFreq: number;
  durationSecs: number;
  volume?: number;
  noiseLevel?: number;
  harmonics?: number;
  emergence?: number;
  shepard?: number;
  timbre?: Timbre;
  mistType?: MistType;
  shepardDirection?: Direction;
  spawnMode?: SpawnMode;
}

export interface MicroTubeSequence {
  id: SequenceId;
  steps: readonly SequenceStep[];
  totalSecs: number;
}

const legacyStep = (
  beatFreq: number,
  baseFreq: number,
  durationSecs: number,
): SequenceStep => ({
  beatFreq,
  baseFreq,
  durationSecs,
});

const totalSecs = (steps: readonly SequenceStep[]) =>
  steps.reduce((sum, step) => sum + step.durationSecs, 0);

const DEEP_FOCUS_STEPS = [
  legacyStep(18, 250, 600),
  legacyStep(10, 220, 600),
  legacyStep(6, 200, 300),
] as const;

const WAKE_UP_STEPS = [
  legacyStep(2, 180, 120),
  legacyStep(6, 200, 180),
  legacyStep(10, 220, 180),
  legacyStep(15, 240, 120),
] as const;

const POWER_NAP_STEPS = [
  legacyStep(10, 220, 300),
  legacyStep(5, 200, 600),
  legacyStep(10, 220, 180),
  legacyStep(14, 240, 120),
] as const;

const DEEP_MEDITATION_STEPS = [
  legacyStep(10, 220, 300),
  legacyStep(6, 200, 900),
  legacyStep(4, 190, 300),
  legacyStep(10, 220, 300),
] as const;

const ORCH_OR_STEPS = [
  legacyStep(40, 280, 300),
  legacyStep(7.83, 220, 600),
  legacyStep(40, 280, 300),
  legacyStep(6, 200, 300),
] as const;

// timbre: Organ 0 / Flute 1 / Bell 2 / Saw 3
// mist:   Pink 0 / White 1 / Brown 2 / Blue 3 / Velvet 4
// dir:    Rising 0 / Falling 1     spawn: Canon 0 / Penrose 1
const JOURNEY_STEPS = [
  { beatFreq: 40, baseFreq: 432, durationSecs: 21, volume: 0.4, noiseLevel: 0.1, harmonics: 0.85, emergence: 0.55, shepard: 0, timbre: 2, mistType: 4, shepardDirection: 0, spawnMode: 1 },
  { beatFreq: 22, baseFreq: 384, durationSecs: 34, volume: 0.5, noiseLevel: 0.15, harmonics: 0.7, emergence: 0.6, shepard: 0.1, timbre: 2, mistType: 4, shepardDirection: 0, spawnMode: 0 },
  { beatFreq: 14, baseFreq: 320, durationSecs: 55, volume: 0.6, noiseLevel: 0.2, harmonics: 0.55, emergence: 0.45, shepard: 0.2, timbre: 1, mistType: 0, shepardDirection: 0, spawnMode: 0 },
  { beatFreq: 10, baseFreq: 256, durationSecs: 89, volume: 0.65, noiseLevel: 0.25, harmonics: 0.5, emergence: 0.35, shepard: 0.25, timbre: 1, mistType: 0, shepardDirection: 0, spawnMode: 0 },
  { beatFreq: 7.83, baseFreq: 196, durationSecs: 144, volume: 0.7, noiseLevel: 0.4, harmonics: 0.45, emergence: 0.3, shepard: 0.3, timbre: 0, mistType: 2, shepardDirection: 0, spawnMode: 0 },
  { beatFreq: 5, baseFreq: 165, durationSecs: 233, volume: 0.7, noiseLevel: 0.35, harmonics: 0.5, emergence: 0.4, shepard: 0.45, timbre: 0, mistType: 0, shepardDirection: 0, spawnMode: 0 },
  { beatFreq: 3, baseFreq: 130.81, durationSecs: 377, volume: 0.7, noiseLevel: 0.3, harmonics: 0.6, emergence: 0.55, shepard: 0.6, timbre: 0, mistType: 0, shepardDirection: 0, spawnMode: 1 },
  { beatFreq: 2, baseFreq: 110, durationSecs: 233, volume: 0.65, noiseLevel: 0.25, harmonics: 0.7, emergence: 0.7, shepard: 0.7, timbre: 2, mistType: 3, shepardDirection: 0, spawnMode: 1 },
  { beatFreq: 4, baseFreq: 87.31, durationSecs: 144, volume: 0.6, noiseLevel: 0.3, harmonics: 0.8, emergence: 0.8, shepard: 0.8, timbre: 2, mistType: 3, shepardDirection: 0, spawnMode: 1 },
  { beatFreq: 8, baseFreq: 73.42, durationSecs: 89, volume: 0.55, noiseLevel: 0.45, harmonics: 0.85, emergence: 0.9, shepard: 0.85, timbre: 3, mistType: 1, shepardDirection: 0, spawnMode: 1 },
  { beatFreq: 18, baseFreq: 65.41, durationSecs: 55, volume: 0.45, noiseLevel: 0.75, harmonics: 0.5, emergence: 0.55, shepard: 0.7, timbre: 3, mistType: 1, shepardDirection: 0, spawnMode: 1 },
  { beatFreq: 60, baseFreq: 55, durationSecs: 34, volume: 0.25, noiseLevel: 0.85, harmonics: 0.35, emergence: 0.25, shepard: 0.4, timbre: 3, mistType: 4, shepardDirection: 1, spawnMode: 1 },
  { beatFreq: 40, baseFreq: 432, durationSecs: 21, volume: 0.5, noiseLevel: 0.1, harmonics: 0.85, emergence: 0.55, shepard: 0, timbre: 2, mistType: 4, shepardDirection: 0, spawnMode: 1 },
] as const satisfies readonly SequenceStep[];

export const SEQUENCES: readonly MicroTubeSequence[] = [
  {
    id: 'deep-focus',
    steps: DEEP_FOCUS_STEPS,
    totalSecs: totalSecs(DEEP_FOCUS_STEPS),
  },
  {
    id: 'wake-up',
    steps: WAKE_UP_STEPS,
    totalSecs: totalSecs(WAKE_UP_STEPS),
  },
  {
    id: 'power-nap',
    steps: POWER_NAP_STEPS,
    totalSecs: totalSecs(POWER_NAP_STEPS),
  },
  {
    id: 'deep-meditation',
    steps: DEEP_MEDITATION_STEPS,
    totalSecs: totalSecs(DEEP_MEDITATION_STEPS),
  },
  {
    id: 'orch-or',
    steps: ORCH_OR_STEPS,
    totalSecs: totalSecs(ORCH_OR_STEPS),
  },
  {
    id: 'journey-through-cosmos',
    steps: JOURNEY_STEPS,
    totalSecs: totalSecs(JOURNEY_STEPS),
  },
] as const;

export const DEFAULT_SEQUENCE_ID: SequenceId = 'journey-through-cosmos';

export function getSequence(id: SequenceId): MicroTubeSequence {
  return SEQUENCES.find((sequence) => sequence.id === id) ?? SEQUENCES[0];
}

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

const setContinuous = <K extends keyof Pick<
  MicroTubeState,
  | 'volume'
  | 'noiseLevel'
  | 'harmonics'
  | 'emergence'
  | 'shepard'
>>(
  state: Partial<MicroTubeState>,
  step: SequenceStep,
  next: SequenceStep,
  key: K,
  progress: number,
) => {
  const value = step[key];
  if (value === undefined) return;
  const nextValue = next[key];
  state[key] =
    nextValue === undefined ? value : lerp(value, nextValue, progress);
};

const setDiscrete = <K extends keyof Pick<
  MicroTubeState,
  'timbre' | 'mistType' | 'shepardDirection' | 'spawnMode'
>>(
  state: Partial<MicroTubeState>,
  step: SequenceStep,
  key: K,
) => {
  const value = step[key];
  if (value !== undefined) {
    state[key] = value;
  }
};

export interface SequenceSample {
  state: Partial<MicroTubeState>;
  stepIndex: number;
  done: boolean;
}

/**
 * Sample a sequence at `elapsed` seconds. Continuous automated fields are
 * lerped toward the next step; discrete automated fields snap on entry.
 */
export function sampleSequence(
  sequence: MicroTubeSequence,
  elapsed: number,
): SequenceSample {
  let acc = 0;
  for (let i = 0; i < sequence.steps.length; i += 1) {
    const step = sequence.steps[i];
    if (elapsed < acc + step.durationSecs) {
      const progress = (elapsed - acc) / step.durationSecs;
      const next = sequence.steps[i + 1] ?? step;
      const state: Partial<MicroTubeState> = {
        beatFreq: lerp(step.beatFreq, next.beatFreq, progress),
        baseFreq: lerp(step.baseFreq, next.baseFreq, progress),
      };

      setContinuous(state, step, next, 'volume', progress);
      setContinuous(state, step, next, 'noiseLevel', progress);
      setContinuous(state, step, next, 'harmonics', progress);
      setContinuous(state, step, next, 'emergence', progress);
      setContinuous(state, step, next, 'shepard', progress);
      setDiscrete(state, step, 'timbre');
      setDiscrete(state, step, 'mistType');
      setDiscrete(state, step, 'shepardDirection');
      setDiscrete(state, step, 'spawnMode');

      return {
        stepIndex: i,
        done: false,
        state,
      };
    }
    acc += step.durationSecs;
  }

  const last = sequence.steps[sequence.steps.length - 1];
  const state: Partial<MicroTubeState> = {
    beatFreq: last.beatFreq,
    baseFreq: last.baseFreq,
  };

  setContinuous(state, last, last, 'volume', 1);
  setContinuous(state, last, last, 'noiseLevel', 1);
  setContinuous(state, last, last, 'harmonics', 1);
  setContinuous(state, last, last, 'emergence', 1);
  setContinuous(state, last, last, 'shepard', 1);
  setDiscrete(state, last, 'timbre');
  setDiscrete(state, last, 'mistType');
  setDiscrete(state, last, 'shepardDirection');
  setDiscrete(state, last, 'spawnMode');

  return {
    stepIndex: sequence.steps.length - 1,
    done: true,
    state,
  };
}
