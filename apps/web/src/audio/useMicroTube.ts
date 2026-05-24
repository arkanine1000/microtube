// useMicroTube — the Web Audio orchestration hook.
//
// Owns the AudioContext, boots the Wasm-in-worklet pipeline, mirrors engine
// state for the UI, and drives the sequence executor. Components
// only ever read `state` and call the returned actions.

import { useCallback, useEffect, useRef, useState } from 'react';
import {
  clamp,
  DEFAULT_STATE,
  type MicroTubeState,
  type PresetSnapshot,
} from './params';
import {
  DEFAULT_SEQUENCE_ID,
  getSequence,
  sampleSequence,
  type MicroTubeSequence,
  type SequenceId,
} from './sequences';

export type EngineStatus = 'idle' | 'loading' | 'running' | 'error';

const WORKLET_URL = '/microtube-worklet/processor.js';
const WASM_URL = '/microtube-worklet/wasm/microtube_core_bg.wasm';
const MEDIA_SESSION_ARTWORK: MediaImage[] = [
  { src: '/pwa-192.png', sizes: '192x192', type: 'image/png' },
  { src: '/pwa-512.png', sizes: '512x512', type: 'image/png' },
];
const MEDIA_SESSION_ACTIONS = ['play', 'pause', 'stop', 'playpause'] as const;
const MEDIA_KEY_ACTIONS = new Set(['MediaPlay', 'MediaPause', 'MediaPlayPause']);

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

function createPlaybackAudioContext(): AudioContext {
  try {
    return new AudioContext({ latencyHint: 'playback' });
  } catch {
    return new AudioContext();
  }
}

function mediaSession(): MediaSession | null {
  return 'mediaSession' in navigator ? navigator.mediaSession : null;
}

function setMediaSessionHandler(
  action: (typeof MEDIA_SESSION_ACTIONS)[number],
  handler: MediaSessionActionHandler | null,
) {
  const session = mediaSession();
  if (!session) return;
  try {
    session.setActionHandler(action as MediaSessionAction, handler);
  } catch {
    // Some browsers expose Media Session but not every action handler.
  }
}

function clearMediaSessionHandlers() {
  for (const action of MEDIA_SESSION_ACTIONS) {
    setMediaSessionHandler(action, null);
  }
}

function setMediaPlaybackState(state: MediaSessionPlaybackState) {
  const session = mediaSession();
  if (!session) return;
  try {
    session.playbackState = state;
  } catch {
    // Some implementations expose a partial Media Session surface.
  }
}

export interface SequenceStatus {
  active: boolean;
  activeId: SequenceId | null;
  stepIndex: number;
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
  sequence: SequenceStatus;
  timer: TimerStatus;
  start: () => Promise<void>;
  setParam: <K extends keyof MicroTubeState>(key: K, value: MicroTubeState[K]) => void;
  setPlaying: (playing: boolean) => void;
  togglePlaying: () => void;
  setTimerEnabled: (enabled: boolean) => void;
  setTimerMinutes: (minutes: number) => void;
  applySnapshot: (snapshot: PresetSnapshot) => void;
  startSequence: (id: SequenceId) => void;
  stopSequence: () => void;
  returnToStart: () => Promise<void>;
}

const DEFAULT_SEQUENCE = getSequence(DEFAULT_SEQUENCE_ID);

const sequenceDefaults = (
  sequence: MicroTubeSequence,
): Partial<MicroTubeState> => {
  if (sequence.id === DEFAULT_SEQUENCE_ID) {
    const defaults: Partial<MicroTubeState> = { ...DEFAULT_STATE };
    delete defaults.playing;
    return defaults;
  }

  const defaults: Partial<MicroTubeState> = {
    beatFreq: DEFAULT_STATE.beatFreq,
    baseFreq: DEFAULT_STATE.baseFreq,
  };
  if (sequence.steps.some((step) => step.volume !== undefined)) {
    defaults.volume = DEFAULT_STATE.volume;
  }
  if (sequence.steps.some((step) => step.noiseLevel !== undefined)) {
    defaults.noiseLevel = DEFAULT_STATE.noiseLevel;
  }
  if (sequence.steps.some((step) => step.harmonics !== undefined)) {
    defaults.harmonics = DEFAULT_STATE.harmonics;
  }
  if (sequence.steps.some((step) => step.emergence !== undefined)) {
    defaults.emergence = DEFAULT_STATE.emergence;
  }
  if (sequence.steps.some((step) => step.gravity !== undefined)) {
    defaults.gravity = DEFAULT_STATE.gravity;
  }
  if (sequence.steps.some((step) => step.shepard !== undefined)) {
    defaults.shepard = DEFAULT_STATE.shepard;
  }
  if (sequence.steps.some((step) => step.timbre !== undefined)) {
    defaults.timbre = DEFAULT_STATE.timbre;
  }
  if (sequence.steps.some((step) => step.mistType !== undefined)) {
    defaults.mistType = DEFAULT_STATE.mistType;
  }
  if (sequence.steps.some((step) => step.shepardDirection !== undefined)) {
    defaults.shepardDirection = DEFAULT_STATE.shepardDirection;
  }
  if (sequence.steps.some((step) => step.spawnMode !== undefined)) {
    defaults.spawnMode = DEFAULT_STATE.spawnMode;
  }

  return defaults;
};

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
  const [sequence, setSequence] = useState<SequenceStatus>({
    active: false,
    activeId: null,
    stepIndex: 0,
    elapsed: 0,
    total: DEFAULT_SEQUENCE.totalSecs,
  });

  const ctxRef = useRef<AudioContext | null>(null);
  const nodeRef = useRef<AudioWorkletNode | null>(null);
  const stateRef = useRef<MicroTubeState>(DEFAULT_STATE);
  const sessionStartRef = useRef<number>(0);
  const sequenceElapsedRef = useRef<number>(0);
  const sequenceLastTickRef = useRef<number | null>(null);
  const sequenceActiveRef = useRef<boolean>(false);
  const activeSequenceRef = useRef<MicroTubeSequence | null>(null);
  const timerEnabledRef = useRef<boolean>(true);
  const timerMinutesRef = useRef<number>(TIMER_DEFAULT_MINUTES);
  const timerStartedAtRef = useRef<number | null>(null);
  const timerElapsedBeforePauseRef = useRef<number>(0);
  const timerFiredRef = useRef<boolean>(false);
  const shuttingDownRef = useRef<boolean>(false);

  const pauseSequenceClock = useCallback((now: number) => {
    if (!sequenceActiveRef.current || sequenceLastTickRef.current === null) return;
    sequenceElapsedRef.current += (now - sequenceLastTickRef.current) / 1000;
    sequenceLastTickRef.current = null;
  }, []);

  const resumeSequenceClock = useCallback((now: number) => {
    if (sequenceActiveRef.current) {
      sequenceLastTickRef.current = now;
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
        setMediaPlaybackState(playing ? 'playing' : 'paused');
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
        resumeSequenceClock(now);
      } else {
        if (timerStartedAtRef.current !== null) {
          timerElapsedBeforePauseRef.current += now - timerStartedAtRef.current;
          timerStartedAtRef.current = null;
        }
        pauseSequenceClock(now);
      }

      commitParam('playing', playing);
      setMediaPlaybackState(playing ? 'playing' : 'paused');
      syncTimerState(now);
    },
    [commitParam, pauseSequenceClock, resumeSequenceClock, syncTimerState],
  );

  const resumeAudioContext = useCallback(() => {
    const ctx = ctxRef.current;
    if (!ctx || ctx.state !== 'suspended') return;
    void ctx.resume().catch(() => {
      // Media keys can arrive while the browser is tearing down the context.
    });
  }, []);

  const mediaPlay = useCallback(() => {
    resumeAudioContext();
    setPlaying(true);
  }, [resumeAudioContext, setPlaying]);

  const mediaPause = useCallback(() => {
    if (stateRef.current.playing) {
      setPlaying(false);
    } else {
      mediaPlay();
    }
  }, [mediaPlay, setPlaying]);

  const mediaToggle = useCallback(() => {
    if (stateRef.current.playing) {
      setPlaying(false);
    } else {
      mediaPlay();
    }
  }, [mediaPlay, setPlaying]);

  const mediaStop = useCallback(() => {
    setPlaying(false);
  }, [setPlaying]);

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

  const cancelSequence = useCallback(() => {
    sequenceActiveRef.current = false;
    sequenceLastTickRef.current = null;
    sequenceElapsedRef.current = 0;
    activeSequenceRef.current = null;
    setSequence({
      active: false,
      activeId: null,
      stepIndex: 0,
      elapsed: 0,
      total: DEFAULT_SEQUENCE.totalSecs,
    });
  }, []);

  const applySnapshot = useCallback(
    (snapshot: PresetSnapshot) => {
      cancelSequence();
      stateRef.current = { ...stateRef.current, ...snapshot };
      setState(stateRef.current);
      post({ type: 'params', value: snapshot });
    },
    [cancelSequence, post],
  );

  // --- boot pipeline -------------------------------------------------------

  const start = useCallback(async () => {
    if (status === 'loading' || status === 'running' || shuttingDownRef.current) return;
    setStatus('loading');
    setError(null);
    try {
      const ctx = createPlaybackAudioContext();
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

          const now = performance.now();
          sessionStartRef.current = now;
          timerStartedAtRef.current =
            timerEnabledRef.current && stateRef.current.playing ? now : null;
          syncTimerState(now);
          setUptimeSecs(0);
          setStatus('running');
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

  // --- media session -------------------------------------------------------

  useEffect(() => {
    const session = mediaSession();
    if (!session || status !== 'running') return;

    if ('MediaMetadata' in window) {
      try {
        session.metadata = new MediaMetadata({
          title: 'MicroTube',
          artist: 'MicroTube',
          album: 'Binaural beat synthesis studio',
          artwork: MEDIA_SESSION_ARTWORK,
        });
      } catch {
        // Metadata artwork support varies; action handlers are the key part.
      }
    }

    setMediaSessionHandler('play', mediaPlay);
    setMediaSessionHandler('pause', mediaPause);
    setMediaSessionHandler('stop', mediaStop);
    setMediaSessionHandler('playpause', mediaToggle);

    return () => {
      clearMediaSessionHandlers();
      try {
        session.playbackState = 'none';
        session.metadata = null;
      } catch {
        // Ignore browser-specific teardown behavior.
      }
    };
  }, [mediaPause, mediaPlay, mediaStop, mediaToggle, status]);

  useEffect(() => {
    setMediaPlaybackState(
      status === 'running' ? (state.playing ? 'playing' : 'paused') : 'none',
    );
  }, [state.playing, status]);

  useEffect(() => {
    if (status !== 'running') return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (!MEDIA_KEY_ACTIONS.has(event.key)) return;
      event.preventDefault();
      if (event.repeat) return;
      if (event.key === 'MediaPlay') {
        mediaPlay();
      } else if (event.key === 'MediaPause') {
        mediaPause();
      } else {
        mediaToggle();
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [mediaPause, mediaPlay, mediaToggle, status]);

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
        pauseSequenceClock(now);
        stateRef.current = { ...stateRef.current, playing: false };
        setState(stateRef.current);
        pushParam('playing', false);
      }
      syncTimerState(now);
    }, 1000);
    return () => window.clearInterval(id);
  }, [
    pauseSequenceClock,
    pushParam,
    status,
    syncTimerState,
    timerDurationMs,
    timerElapsedMs,
  ]);

  // --- sequence executor ---------------------------------------------------

  const startSequence = useCallback((id: SequenceId) => {
    if (status !== 'running') return;
    const now = performance.now();
    const selected = getSequence(id);
    const sample = sampleSequence(selected, 0);
    sequenceElapsedRef.current = 0;
    sequenceLastTickRef.current = stateRef.current.playing ? now : null;
    sequenceActiveRef.current = true;
    activeSequenceRef.current = selected;
    if (stateRef.current.playing) {
      stateRef.current = { ...stateRef.current, ...sample.state };
      setState(stateRef.current);
      post({ type: 'params', value: sample.state });
    }
    setSequence({
      active: true,
      activeId: selected.id,
      elapsed: 0,
      stepIndex: sample.stepIndex,
      total: selected.totalSecs,
    });
  }, [post, status]);

  const stopSequence = useCallback(() => {
    const selected = activeSequenceRef.current;
    cancelSequence();

    if (!selected) return;
    const defaults = sequenceDefaults(selected);
    stateRef.current = { ...stateRef.current, ...defaults };
    setState(stateRef.current);
    post({ type: 'params', value: defaults });

  }, [cancelSequence, post]);

  const returnToStart = useCallback(async () => {
    if (status !== 'running' || shuttingDownRef.current) return;
    shuttingDownRef.current = true;
    setStatus('idle');

    sequenceActiveRef.current = false;
    sequenceLastTickRef.current = null;
    sequenceElapsedRef.current = 0;
    activeSequenceRef.current = null;

    timerEnabledRef.current = true;
    timerMinutesRef.current = TIMER_DEFAULT_MINUTES;
    timerElapsedBeforePauseRef.current = 0;
    timerStartedAtRef.current = null;
    timerFiredRef.current = false;

    stateRef.current = { ...DEFAULT_STATE };
    setState(stateRef.current);
    setUptimeSecs(0);
    setError(null);
    setTimer({
      enabled: true,
      minutes: TIMER_DEFAULT_MINUTES,
      remainingSecs: TIMER_DEFAULT_MINUTES * 60,
      fired: false,
    });
    setSequence({
      active: false,
      activeId: null,
      stepIndex: 0,
      elapsed: 0,
      total: DEFAULT_SEQUENCE.totalSecs,
    });

    const node = nodeRef.current;
    const ctx = ctxRef.current;
    nodeRef.current = null;
    ctxRef.current = null;

    try {
      node?.disconnect();
    } catch {
      // Ignore teardown races while the worklet is being torn down.
    }

    try {
      await ctx?.close();
    } catch {
      // Closing a context can fail if the browser has already discarded it.
    } finally {
      shuttingDownRef.current = false;
    }
  }, [status]);

  useEffect(() => {
    if (!sequence.active) return;
    // A 250 ms tick is well inside the engine's 50 ms smoothing window, so
    // lerped parameters move continuously without audible stair-stepping.
    const id = window.setInterval(() => {
      if (!sequenceActiveRef.current) return;
      const selected = activeSequenceRef.current;
      if (!selected) return;
      const now = performance.now();
      if (!stateRef.current.playing) {
        sequenceLastTickRef.current = null;
        return;
      }
      if (sequenceLastTickRef.current === null) {
        sequenceLastTickRef.current = now;
      }
      sequenceElapsedRef.current += (now - sequenceLastTickRef.current) / 1000;
      sequenceLastTickRef.current = now;
      const elapsed = sequenceElapsedRef.current;
      const sample = sampleSequence(selected, elapsed);

      stateRef.current = { ...stateRef.current, ...sample.state };
      setState(stateRef.current);
      post({ type: 'params', value: sample.state });

      setSequence((current) => ({
        ...current,
        elapsed: Math.min(elapsed, selected.totalSecs),
        stepIndex: sample.stepIndex,
      }));

      if (sample.done) {
        sequenceActiveRef.current = false;
        activeSequenceRef.current = null;
        setSequence((current) => ({ ...current, active: false }));
      }
    }, 250);
    return () => window.clearInterval(id);
  }, [sequence.active, post]);

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
    sequence,
    timer,
    start,
    setParam,
    setPlaying,
    togglePlaying,
    setTimerEnabled,
    setTimerMinutes,
    applySnapshot,
    startSequence,
    stopSequence,
    returnToStart,
  };
}
