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
import { useMicroTube } from './audio/useMicroTube';
import { JourneyPanel } from './components/JourneyPanel';
import { ParameterSlider } from './components/ParameterSlider';
import { Segmented } from './components/Segmented';
import { StripDashboard } from './components/StripDashboard';

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
