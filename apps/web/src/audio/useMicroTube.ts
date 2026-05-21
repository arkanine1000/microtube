// useMicroTube — the Web Audio orchestration hook.
//
// Owns the AudioContext, boots the Wasm-in-worklet pipeline, mirrors engine
// state for the UI, and drives the Journey sequence executor. Components
// only ever read `state` and call the returned actions.

import { useCallback, useEffect, useRef, useState } from 'react';
import { clamp, DEFAULT_STATE, type MicroTubeState } from './params';
import { JOURNEY_TOTAL_SECS, sampleJourney } from './sequences';

export type EngineStatus = 'idle' | 'loading' | 'running' | 'error';

const WORKLET_URL = '/microtube-worklet/processor.js';
const WASM_URL = '/microtube-worklet/wasm/microtube_core_bg.wasm';

export const TIMER_DEFAULT_MINUTES = 60;
export const TIMER_MIN_MINUTES = 5;
export const TIMER_MAX_MINUTES = 120;
export const TIMER_STEP_MINUTES = 5;

const MS_PER_MINUTE = 60_000;

const snapTimerMinutes = (minutes: number) =>
  clamp(
    Math.round(minutes / TIMER_STEP_MINUTES) * TIMER_STEP_MINUTES,
    TIMER_MIN_MINUTES,
    TIMER_MAX_MINUTES,
  );

export interface JourneyStatus {
  active: boolean;
  stepIndex: number;
  stepName: string;
  elapsed: number;
  total: number;
}

export interface TimerStatus {
  enabled: boolean;
  minutes: number;
  remainingSecs: number | null;
  fired: boolean;
}

export interface MicroTube {
  status: EngineStatus;
  error: string | null;
  state: MicroTubeState;
  uptimeSecs: number;
  journey: JourneyStatus;
  timer: TimerStatus;
  start: () => Promise<void>;
  setParam: <K extends keyof MicroTubeState>(key: K, value: MicroTubeState[K]) => void;
  setPlaying: (playing: boolean) => void;
  togglePlaying: () => void;
  setTimerEnabled: (enabled: boolean) => void;
  setTimerMinutes: (minutes: number) => void;
  startJourney: () => void;
  stopJourney: () => void;
}

export function useMicroTube(): MicroTube {
  const [status, setStatus] = useState<EngineStatus>('idle');
  const [error, setError] = useState<string | null>(null);
  const [state, setState] = useState<MicroTubeState>(DEFAULT_STATE);
  const [uptimeSecs, setUptimeSecs] = useState(0);
  const [timer, setTimer] = useState<TimerStatus>({
    enabled: true,
    minutes: TIMER_DEFAULT_MINUTES,
    remainingSecs: TIMER_DEFAULT_MINUTES * 60,
    fired: false,
  });
  const [journey, setJourney] = useState<JourneyStatus>({
    active: false,
    stepIndex: 0,
    stepName: '',
    elapsed: 0,
    total: JOURNEY_TOTAL_SECS,
  });

  const ctxRef = useRef<AudioContext | null>(null);
  const nodeRef = useRef<AudioWorkletNode | null>(null);
  const stateRef = useRef<MicroTubeState>(DEFAULT_STATE);
  const sessionStartRef = useRef<number>(0);
  const journeyElapsedRef = useRef<number>(0);
  const journeyLastTickRef = useRef<number | null>(null);
  const journeyActiveRef = useRef<boolean>(false);
  const timerEnabledRef = useRef<boolean>(true);
  const timerMinutesRef = useRef<number>(TIMER_DEFAULT_MINUTES);
  const timerStartedAtRef = useRef<number | null>(null);
  const timerElapsedBeforePauseRef = useRef<number>(0);
  const timerFiredRef = useRef<boolean>(false);

  const pauseJourneyClock = useCallback((now: number) => {
    if (!journeyActiveRef.current || journeyLastTickRef.current === null) return;
    journeyElapsedRef.current += (now - journeyLastTickRef.current) / 1000;
    journeyLastTickRef.current = null;
  }, []);

  const resumeJourneyClock = useCallback((now: number) => {
    if (journeyActiveRef.current) {
      journeyLastTickRef.current = now;
    }
  }, []);

  const timerDurationMs = useCallback(
    () => timerMinutesRef.current * MS_PER_MINUTE,
    [],
  );

  const timerElapsedMs = useCallback(
    (now: number) => {
      if (timerFiredRef.current) {
        return timerDurationMs();
      }

      let elapsed = timerElapsedBeforePauseRef.current;
      if (
        timerEnabledRef.current &&
        stateRef.current.playing &&
        timerStartedAtRef.current !== null
      ) {
        elapsed += now - timerStartedAtRef.current;
      }
      return elapsed;
    },
    [timerDurationMs],
  );

  const syncTimerState = useCallback(
    (now = performance.now()) => {
      if (!timerEnabledRef.current) {
        setTimer({
          enabled: false,
          minutes: timerMinutesRef.current,
          remainingSecs: null,
          fired: timerFiredRef.current,
        });
        return;
      }

      const remainingMs = Math.max(0, timerDurationMs() - timerElapsedMs(now));
      setTimer({
        enabled: true,
        minutes: timerMinutesRef.current,
        remainingSecs: Math.ceil(remainingMs / 1000),
        fired: timerFiredRef.current,
      });
    },
    [timerDurationMs, timerElapsedMs],
  );

  // --- worklet messaging ---------------------------------------------------

  const post = useCallback((message: unknown) => {
    nodeRef.current?.port.postMessage(message);
  }, []);

  const pushParam = useCallback(
    <K extends keyof MicroTubeState>(key: K, value: MicroTubeState[K]) => {
      post({ type: 'param', name: key, value });
    },
    [post],
  );

  /** Update one parameter: local mirror, React state, and the worklet. */
  const commitParam = useCallback(
    <K extends keyof MicroTubeState>(key: K, value: MicroTubeState[K]) => {
      stateRef.current = { ...stateRef.current, [key]: value };
      setState(stateRef.current);
      pushParam(key, value);
    },
    [pushParam],
  );

  const setPlaying = useCallback(
    (playing: boolean) => {
      const now = performance.now();
      const wasPlaying = stateRef.current.playing;
      if (wasPlaying === playing) {
        syncTimerState(now);
        return;
      }

      if (playing) {
        if (timerEnabledRef.current) {
          if (timerFiredRef.current) {
            timerElapsedBeforePauseRef.current = 0;
            timerFiredRef.current = false;
          }
          timerStartedAtRef.current = now;
        }
        resumeJourneyClock(now);
      } else {
        if (timerStartedAtRef.current !== null) {
          timerElapsedBeforePauseRef.current += now - timerStartedAtRef.current;
          timerStartedAtRef.current = null;
        }
        pauseJourneyClock(now);
      }

      commitParam('playing', playing);
      syncTimerState(now);
    },
    [commitParam, pauseJourneyClock, resumeJourneyClock, syncTimerState],
  );

  const setParam = useCallback(
    <K extends keyof MicroTubeState>(key: K, value: MicroTubeState[K]) => {
      if (key === 'playing') {
        setPlaying(Boolean(value));
        return;
      }
      commitParam(key, value);
    },
    [commitParam, setPlaying],
  );

  const togglePlaying = useCallback(() => {
    setPlaying(!stateRef.current.playing);
  }, [setPlaying]);

  const setTimerEnabled = useCallback(
    (enabled: boolean) => {
      const now = performance.now();
      timerEnabledRef.current = enabled;
      timerElapsedBeforePauseRef.current = 0;
      timerFiredRef.current = false;
      timerStartedAtRef.current =
        enabled && status === 'running' && stateRef.current.playing ? now : null;
      syncTimerState(now);
    },
    [status, syncTimerState],
  );

  const setTimerMinutes = useCallback(
    (minutes: number) => {
      timerMinutesRef.current = snapTimerMinutes(minutes);
      syncTimerState(performance.now());
    },
    [syncTimerState],
  );

  // --- boot pipeline -------------------------------------------------------

  const start = useCallback(async () => {
    if (status === 'loading' || status === 'running') return;
    setStatus('loading');
    setError(null);
    try {
      const ctx = new AudioContext();
      ctxRef.current = ctx;

      // 1. Resume inside the click gesture. A suspended context never pumps
      //    the worklet thread, so its MessagePort would stay silent.
      await ctx.resume();

      // 2. Load the (self-contained, import-free) worklet module.
      await ctx.audioWorklet.addModule(WORKLET_URL);

      // 3. The worklet scope has no `fetch`, so the main thread fetches the
      //    raw Wasm bytes; they are transferred in and compiled there by
      //    `initSync`. (Transferring an ArrayBuffer is rock-solid, unlike
      //    structured-cloning a WebAssembly.Module into an AudioWorklet.)
      const wasmBytes = await fetch(WASM_URL).then((r) => {
        if (!r.ok) throw new Error(`Wasm fetch failed (${r.status})`);
        return r.arrayBuffer();
      });

      // 4. Create the node and wire up messaging. `numberOfInputs: 1`
      //    (the default) — a `0`-input AudioWorkletNode is not reliably
      //    pulled into Chromium's render graph.
      const node = new AudioWorkletNode(ctx, 'microtube-processor', {
        numberOfInputs: 1,
        numberOfOutputs: 1,
        outputChannelCount: [2],
      });
      nodeRef.current = node;
      console.info(
        `[microtube] node created — outputs: ${node.numberOfOutputs}, ` +
          `channelCount: ${node.channelCount}, ctx.sampleRate: ${ctx.sampleRate}, ` +
          `destChannels: ${ctx.destination.channelCount}`,
      );

      let settled = false;
      let constructed = false;

      node.onprocessorerror = () => {
        console.error('[microtube] AudioWorklet processor error');
        if (settled) return;
        settled = true;
        setError('audio worklet processor crashed during setup');
        setStatus('error');
      };

      node.port.onmessage = (event) => {
        const msg = event.data;
        if (msg?.type === 'hello') {
          constructed = true;
        } else if (msg?.type === 'ready') {
          settled = true;
          // Connect only once the processor is fully initialised.
          node.connect(ctx.destination);

          // Graph probe: tap the node through an analyser to confirm signal
          // actually reaches the render graph (and hence the destination).
          const analyser = ctx.createAnalyser();
          node.connect(analyser);
          const probe = new Float32Array(analyser.fftSize);
          window.setTimeout(() => {
            analyser.getFloatTimeDomainData(probe);
            let peak = 0;
            for (const v of probe) peak = Math.max(peak, Math.abs(v));
            console.info(
              `[microtube] graph probe — analyser peak: ${peak.toFixed(4)}, ` +
                `ctx.state: ${ctx.state}`,
            );
            node.disconnect(analyser);
          }, 1500);

          const now = performance.now();
          sessionStartRef.current = now;
          timerStartedAtRef.current =
            timerEnabledRef.current && stateRef.current.playing ? now : null;
          syncTimerState(now);
          setUptimeSecs(0);
          setStatus('running');
        } else if (msg?.type === 'diag') {
          // Surfaced so a silent-output problem is immediately visible.
          console.info(
            `[microtube] worklet diag — output channels: ${msg.channels}, ` +
              `frames: ${msg.frames}, engine peak: ${msg.enginePeak?.toFixed(4)}, ` +
              `context: ${ctx.state}`,
          );
        } else if (msg?.type === 'error') {
          settled = true;
          setError(msg.message ?? 'worklet error');
          setStatus('error');
        }
      };

      node.port.postMessage(
        { type: 'init', wasm: wasmBytes, params: stateRef.current },
        [wasmBytes],
      );

      // 5. Safety net — never leave the UI silently stuck on "loading".
      window.setTimeout(() => {
        if (settled) return;
        settled = true;
        setError(
          constructed
            ? 'engine init stalled — worklet loaded but never reported ready'
            : 'worklet never started — processor was not constructed',
        );
        setStatus('error');
      }, 6000);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus('error');
    }
  }, [status, syncTimerState]);

  // --- session uptime ------------------------------------------------------

  useEffect(() => {
    if (status !== 'running') return;
    const id = window.setInterval(() => {
      setUptimeSecs((performance.now() - sessionStartRef.current) / 1000);
    }, 1000);
    return () => window.clearInterval(id);
  }, [status]);

  // --- auto-stop timer ------------------------------------------------------

  useEffect(() => {
    if (status !== 'running') return;
    const id = window.setInterval(() => {
      const now = performance.now();
      if (
        timerEnabledRef.current &&
        stateRef.current.playing &&
        timerElapsedMs(now) >= timerDurationMs()
      ) {
        timerElapsedBeforePauseRef.current = timerDurationMs();
        timerStartedAtRef.current = null;
        timerFiredRef.current = true;
        pauseJourneyClock(now);
        stateRef.current = { ...stateRef.current, playing: false };
        setState(stateRef.current);
        pushParam('playing', false);
      }
      syncTimerState(now);
    }, 1000);
    return () => window.clearInterval(id);
  }, [
    pauseJourneyClock,
    pushParam,
    status,
    syncTimerState,
    timerDurationMs,
    timerElapsedMs,
  ]);

  // --- Journey sequence executor ------------------------------------------

  const startJourney = useCallback(() => {
    if (status !== 'running') return;
    const now = performance.now();
    const sample = sampleJourney(0);
    journeyElapsedRef.current = 0;
    journeyLastTickRef.current = stateRef.current.playing ? now : null;
    journeyActiveRef.current = true;
    if (stateRef.current.playing) {
      stateRef.current = { ...stateRef.current, ...sample.state };
      setState(stateRef.current);
      post({ type: 'params', value: sample.state });
    }
    setJourney((j) => ({
      ...j,
      active: true,
      elapsed: 0,
      stepIndex: sample.stepIndex,
      stepName: sample.stepName,
    }));
  }, [post, status]);

  const stopJourney = useCallback(() => {
    journeyActiveRef.current = false;
    journeyLastTickRef.current = null;
    journeyElapsedRef.current = 0;

    // The journey sweeps every parameter — leaving it mid-sweep would strand
    // the user in an arbitrary state, so revert the engine to its defaults.
    const defaults: Partial<MicroTubeState> = { ...DEFAULT_STATE };
    delete defaults.playing;
    stateRef.current = { ...stateRef.current, ...defaults };
    setState(stateRef.current);
    post({ type: 'params', value: defaults });

    setJourney({
      active: false,
      stepIndex: 0,
      stepName: '',
      elapsed: 0,
      total: JOURNEY_TOTAL_SECS,
    });
  }, [post]);

  useEffect(() => {
    if (!journey.active) return;
    // A 250 ms tick is well inside the engine's 50 ms smoothing window, so
    // lerped parameters move continuously without audible stair-stepping.
    const id = window.setInterval(() => {
      if (!journeyActiveRef.current) return;
      const now = performance.now();
      if (!stateRef.current.playing) {
        journeyLastTickRef.current = null;
        return;
      }
      if (journeyLastTickRef.current === null) {
        journeyLastTickRef.current = now;
      }
      journeyElapsedRef.current += (now - journeyLastTickRef.current) / 1000;
      journeyLastTickRef.current = now;
      const elapsed = journeyElapsedRef.current;
      const sample = sampleJourney(elapsed);

      stateRef.current = { ...stateRef.current, ...sample.state };
      setState(stateRef.current);
      post({ type: 'params', value: sample.state });

      setJourney((j) => ({
        ...j,
        elapsed: Math.min(elapsed, JOURNEY_TOTAL_SECS),
        stepIndex: sample.stepIndex,
        stepName: sample.stepName,
      }));

      if (sample.done) {
        journeyActiveRef.current = false;
        setJourney((j) => ({ ...j, active: false }));
      }
    }, 250);
    return () => window.clearInterval(id);
  }, [journey.active, post]);

  // --- teardown ------------------------------------------------------------

  useEffect(() => {
    return () => {
      nodeRef.current?.disconnect();
      ctxRef.current?.close();
    };
  }, []);

  return {
    status,
    error,
    state,
    uptimeSecs,
    journey,
    timer,
    start,
    setParam,
    setPlaying,
    togglePlaying,
    setTimerEnabled,
    setTimerMinutes,
    startJourney,
    stopJourney,
  };
}
