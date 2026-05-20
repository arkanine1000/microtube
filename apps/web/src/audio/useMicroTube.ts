// useMicroTube — the Web Audio orchestration hook.
//
// Owns the AudioContext, boots the Wasm-in-worklet pipeline, mirrors engine
// state for the UI, and drives the Journey sequence executor. Components
// only ever read `state` and call the returned actions.

import { useCallback, useEffect, useRef, useState } from 'react';
import { DEFAULT_STATE, type MicroTubeState } from './params';
import { JOURNEY_TOTAL_SECS, sampleJourney } from './sequences';

export type EngineStatus = 'idle' | 'loading' | 'running' | 'error';

const WORKLET_URL = '/microtube-worklet/processor.js';
const WASM_URL = '/microtube-worklet/wasm/microtube_core_bg.wasm';

export interface JourneyStatus {
  active: boolean;
  stepIndex: number;
  stepName: string;
  elapsed: number;
  total: number;
}

export interface MicroTube {
  status: EngineStatus;
  error: string | null;
  state: MicroTubeState;
  uptimeSecs: number;
  journey: JourneyStatus;
  start: () => Promise<void>;
  setParam: <K extends keyof MicroTubeState>(key: K, value: MicroTubeState[K]) => void;
  togglePlaying: () => void;
  startJourney: () => void;
  stopJourney: () => void;
}

export function useMicroTube(): MicroTube {
  const [status, setStatus] = useState<EngineStatus>('idle');
  const [error, setError] = useState<string | null>(null);
  const [state, setState] = useState<MicroTubeState>(DEFAULT_STATE);
  const [uptimeSecs, setUptimeSecs] = useState(0);
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
  const journeyStartRef = useRef<number>(0);
  const journeyActiveRef = useRef<boolean>(false);

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
  const setParam = useCallback(
    <K extends keyof MicroTubeState>(key: K, value: MicroTubeState[K]) => {
      stateRef.current = { ...stateRef.current, [key]: value };
      setState(stateRef.current);
      pushParam(key, value);
    },
    [pushParam],
  );

  const togglePlaying = useCallback(() => {
    setParam('playing', !stateRef.current.playing);
  }, [setParam]);

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

          sessionStartRef.current = performance.now();
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
  }, [status]);

  // --- session uptime ------------------------------------------------------

  useEffect(() => {
    if (status !== 'running') return;
    const id = window.setInterval(() => {
      setUptimeSecs((performance.now() - sessionStartRef.current) / 1000);
    }, 1000);
    return () => window.clearInterval(id);
  }, [status]);

  // --- Journey sequence executor ------------------------------------------

  const startJourney = useCallback(() => {
    if (status !== 'running') return;
    journeyStartRef.current = performance.now();
    journeyActiveRef.current = true;
    setJourney((j) => ({ ...j, active: true, elapsed: 0, stepIndex: 0 }));
  }, [status]);

  const stopJourney = useCallback(() => {
    journeyActiveRef.current = false;
    setJourney((j) => ({ ...j, active: false }));
  }, []);

  useEffect(() => {
    if (!journey.active) return;
    // A 250 ms tick is well inside the engine's 50 ms smoothing window, so
    // lerped parameters move continuously without audible stair-stepping.
    const id = window.setInterval(() => {
      if (!journeyActiveRef.current) return;
      const elapsed = (performance.now() - journeyStartRef.current) / 1000;
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
    start,
    setParam,
    togglePlaying,
    startJourney,
    stopJourney,
  };
}
