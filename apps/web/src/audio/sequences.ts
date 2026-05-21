// Presets and the "Journey Through the Cosmos" sequence executor.
//
// Ported from `crates/cli/src/presets.rs`. The Journey is a 13-step strange
// loop: continuous parameters are linearly interpolated toward the next
// step's value across the step's duration; discrete ones snap on entry.

import type { Direction, MicroTubeState, MistType, SpawnMode, Timbre } from './params';

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

export interface JourneyStep {
  beatFreq: number;
  baseFreq: number;
  durationSecs: number;
  volume: number;
  noiseLevel: number;
  harmonics: number;
  emergence: number;
  shepard: number;
  timbre: Timbre;
  mistType: MistType;
  shepardDirection: Direction;
  spawnMode: SpawnMode;
}

// timbre: Organ 0 / Flute 1 / Bell 2 / Saw 3
// mist:   Pink 0 / White 1 / Brown 2 / Blue 3 / Velvet 4
// dir:    Rising 0 / Falling 1     spawn: Canon 0 / Penrose 1
export const JOURNEY: JourneyStep[] = [
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
];

export const JOURNEY_TOTAL_SECS = JOURNEY.reduce((s, step) => s + step.durationSecs, 0);

const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

export interface SequenceSample {
  state: Partial<MicroTubeState>;
  stepIndex: number;
  done: boolean;
}

/**
 * Sample the Journey at `elapsed` seconds. Continuous fields are lerped
 * toward the next step; discrete fields snap to the current step.
 */
export function sampleJourney(elapsed: number): SequenceSample {
  let acc = 0;
  for (let i = 0; i < JOURNEY.length; i += 1) {
    const step = JOURNEY[i];
    if (elapsed < acc + step.durationSecs) {
      const progress = (elapsed - acc) / step.durationSecs;
      const next = JOURNEY[i + 1] ?? step;
      return {
        stepIndex: i,
        done: false,
        state: {
          beatFreq: lerp(step.beatFreq, next.beatFreq, progress),
          baseFreq: lerp(step.baseFreq, next.baseFreq, progress),
          volume: lerp(step.volume, next.volume, progress),
          noiseLevel: lerp(step.noiseLevel, next.noiseLevel, progress),
          harmonics: lerp(step.harmonics, next.harmonics, progress),
          emergence: lerp(step.emergence, next.emergence, progress),
          shepard: lerp(step.shepard, next.shepard, progress),
          timbre: step.timbre,
          mistType: step.mistType,
          shepardDirection: step.shepardDirection,
          spawnMode: step.spawnMode,
        },
      };
    }
    acc += step.durationSecs;
  }
  const last = JOURNEY[JOURNEY.length - 1];
  return {
    stepIndex: JOURNEY.length - 1,
    done: true,
    state: {
      beatFreq: last.beatFreq,
      baseFreq: last.baseFreq,
      volume: last.volume,
      noiseLevel: last.noiseLevel,
      harmonics: last.harmonics,
      emergence: last.emergence,
      shepard: last.shepard,
      timbre: last.timbre,
      mistType: last.mistType,
      shepardDirection: last.shepardDirection,
      spawnMode: last.spawnMode,
    },
  };
}
