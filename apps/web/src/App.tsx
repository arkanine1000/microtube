import {
  DIRECTIONS,
  MISTS,
  SLIDERS,
  SPAWN_MODES,
  TIMBRES,
  VOLUME,
  type Direction,
  type MistType,
  type SpawnMode,
  type Timbre,
} from './audio/params';
import {
  TIMER_MAX_MINUTES,
  TIMER_MIN_MINUTES,
  TIMER_STEP_MINUTES,
  useMicroTube,
} from './audio/useMicroTube';
import { JourneyPanel } from './components/JourneyPanel';
import { ParameterSlider } from './components/ParameterSlider';
import { Segmented } from './components/Segmented';
import { StripDashboard } from './components/StripDashboard';

function formatClock(secs: number | null): string {
  if (secs === null) return 'off';
  const total = Math.max(0, Math.ceil(secs));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = h > 0 ? String(m).padStart(2, '0') : String(m);
  const ss = String(s).padStart(2, '0');
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

export default function App() {
  const mt = useMicroTube();

  if (mt.status !== 'running') {
    return (
      <div className="app">
        <div className="start">
          <h1>
            micro<span style={{ color: 'var(--accent)' }}>tube</span>
          </h1>
          <p>
            A binaural-beat synthesis studio. Audio is rendered by a Rust DSP
            engine compiled to WebAssembly, running on a dedicated audio
            thread. Use headphones.
          </p>
          <button
            className="btn btn-primary"
            disabled={mt.status === 'loading'}
            onClick={mt.start}
          >
            {mt.status === 'loading' ? 'Spinning up engine…' : 'Enter studio'}
          </button>
          {mt.status === 'error' && (
            <p className="error">Engine failed to start: {mt.error}</p>
          )}
        </div>
      </div>
    );
  }

  const { state } = mt;
  const timerLabel = mt.timer.fired
    ? 'stopped'
    : formatClock(mt.timer.remainingSecs);

  return (
    <div className="app">
      <StripDashboard beatFreq={state.beatFreq} uptimeSecs={mt.uptimeSecs} />

      <section className="panel">
        <h2 className="panel-title">Transport</h2>
        <div className="transport">
          <button
            className="btn btn-play btn-primary"
            onClick={mt.togglePlaying}
            aria-label={state.playing ? 'pause' : 'play'}
          >
            {state.playing ? '⏸' : '▶'}
          </button>
          <div className="transport-vol">
            <ParameterSlider
              spec={VOLUME}
              value={state.volume}
              onChange={(v) => mt.setParam('volume', v)}
            />
          </div>
        </div>
        <div className="timer-controls">
          <label className="timer-toggle">
            <input
              type="checkbox"
              checked={mt.timer.enabled}
              onChange={(e) => mt.setTimerEnabled(e.currentTarget.checked)}
            />
            <span>Auto-stop</span>
          </label>
          <span className={`timer-readout${mt.timer.fired ? ' fired' : ''}`}>
            {timerLabel}
          </span>
        </div>
        <div className="timer-row">
          <button
            className="nudge"
            onClick={() => mt.setTimerMinutes(mt.timer.minutes - TIMER_STEP_MINUTES)}
            aria-label="decrease auto-stop timer"
          >
            ◂
          </button>
          <input
            className="timer-range"
            type="range"
            min={TIMER_MIN_MINUTES}
            max={TIMER_MAX_MINUTES}
            step={TIMER_STEP_MINUTES}
            value={mt.timer.minutes}
            onChange={(e) => mt.setTimerMinutes(Number(e.currentTarget.value))}
            aria-label="auto-stop minutes"
          />
          <button
            className="nudge"
            onClick={() => mt.setTimerMinutes(mt.timer.minutes + TIMER_STEP_MINUTES)}
            aria-label="increase auto-stop timer"
          >
            ▸
          </button>
          <span className="timer-minutes">{mt.timer.minutes} min</span>
        </div>
      </section>

      <section className="panel sliders">
        <h2 className="panel-title">Parameters</h2>
        {SLIDERS.map((spec) => (
          <ParameterSlider
            key={spec.key}
            spec={spec}
            value={state[spec.key]}
            onChange={(v) => mt.setParam(spec.key, v)}
          />
        ))}
      </section>

      <section className="panel">
        <h2 className="panel-title">Quick toggles</h2>
        <Segmented
          caption="Timbre"
          options={TIMBRES}
          value={state.timbre}
          onChange={(v) => mt.setParam('timbre', v as Timbre)}
        />
        <Segmented
          caption="Mist colour"
          options={MISTS}
          value={state.mistType}
          onChange={(v) => mt.setParam('mistType', v as MistType)}
        />
        <Segmented
          caption="Drift direction"
          options={DIRECTIONS}
          value={state.shepardDirection}
          onChange={(v) => mt.setParam('shepardDirection', v as Direction)}
        />
        <Segmented
          caption="Emergence spawn"
          options={SPAWN_MODES}
          value={state.spawnMode}
          onChange={(v) => mt.setParam('spawnMode', v as SpawnMode)}
        />
      </section>

      <JourneyPanel mt={mt} />

      <footer style={{ textAlign: 'center', color: 'var(--text-dim)', fontSize: 11 }}>
        engine · microtube-core (Rust → WebAssembly) · {Math.round(mt.uptimeSecs)}s session
      </footer>
    </div>
  );
}
