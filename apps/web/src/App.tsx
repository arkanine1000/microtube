import { Headphones, Minus, Plus } from 'lucide-react';
import { useState } from 'react';
import {
  DIRECTIONS,
  EEG_BANDS,
  MISTS,
  SLIDERS,
  SPAWN_MODES,
  TIMBRES,
  VOLUME,
  eegBandIndex,
  type Direction,
  type MistType,
  type SpawnMode,
  type Timbre,
} from './audio/params';
import type { Preset } from './audio/sequences';
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
import { TopBar } from './components/TopBar';

type StudioTab = 'play' | 'shape';

const TABS: Array<{ id: StudioTab; label: string; caption: string }> = [
  { id: 'play', label: 'Play', caption: 'transport · modes · journey' },
  { id: 'shape', label: 'Shape', caption: 'tone parameters' },
];

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
  const [activeTab, setActiveTab] = useState<StudioTab>('play');

  if (mt.status !== 'running') {
    return (
      <div className="app">
        <div className="start" aria-live="polite">
          <div className="start-aurora" aria-hidden="true" />
          <div className="start-mark">
            micro<span>tube</span>
          </div>
          <p className="start-tagline">Tune your mind to a frequency.</p>
          <p className="start-blurb">
            A binaural-beat synthesis studio. A Rust DSP engine, compiled to
            WebAssembly, renders every tone on a dedicated audio thread.
          </p>
          <button
            className="btn btn-primary btn-enter"
            type="button"
            disabled={mt.status === 'loading'}
            onClick={mt.start}
          >
            {mt.status === 'loading' ? 'Spinning up engine…' : 'Enter studio'}
          </button>
          <div className="start-headphones">
            <Headphones size={15} strokeWidth={2.1} />
            Headphones required — the binaural effect lives in the gap between
            your ears.
          </div>
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
  const accent = EEG_BANDS[eegBandIndex(state.beatFreq)].color;

  const applyPreset = (preset: Preset) => {
    mt.setParam('beatFreq', preset.beatFreq);
    mt.setParam('baseFreq', preset.baseFreq);
    mt.setParam('noiseLevel', preset.noiseLevel);
  };

  return (
    <div className="app">
      <div
        className="studio-shell"
        style={{ ['--accent' as string]: accent }}
      >
        <TopBar
          playing={state.playing}
          onToggle={mt.togglePlaying}
          uptimeSecs={mt.uptimeSecs}
          beatFreq={state.beatFreq}
        />

        <StripDashboard beatFreq={state.beatFreq} onApplyPreset={applyPreset} />

        <nav className="studio-tabs" aria-label="Studio sections">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className={`studio-tab${activeTab === tab.id ? ' active' : ''}`}
              type="button"
              onClick={() => setActiveTab(tab.id)}
              aria-current={activeTab === tab.id ? 'page' : undefined}
            >
              <span>{tab.label}</span>
              <small>{tab.caption}</small>
            </button>
          ))}
        </nav>

        <main className="studio-stage">
          {activeTab === 'play' && (
            <div className="tab-panel play-tab">
              <section className="panel transport-panel">
                <h2 className="panel-title">Transport</h2>
                <div className="transport-vol">
                  <ParameterSlider
                    spec={VOLUME}
                    value={state.volume}
                    onChange={(v) => mt.setParam('volume', v)}
                  />
                </div>
                <div className="timer-controls">
                  <label className="timer-toggle">
                    <input
                      type="checkbox"
                      checked={mt.timer.enabled}
                      onChange={(e) =>
                        mt.setTimerEnabled(e.currentTarget.checked)
                      }
                    />
                    <span>Auto-stop</span>
                  </label>
                  <span
                    className={`timer-readout${
                      mt.timer.fired ? ' fired' : ''
                    }`}
                  >
                    {timerLabel}
                  </span>
                </div>
                <div className="timer-row">
                  <button
                    className="nudge"
                    type="button"
                    onClick={() =>
                      mt.setTimerMinutes(mt.timer.minutes - TIMER_STEP_MINUTES)
                    }
                    aria-label="decrease auto-stop timer"
                  >
                    <Minus size={16} strokeWidth={2.6} />
                  </button>
                  <input
                    className="timer-range"
                    type="range"
                    min={TIMER_MIN_MINUTES}
                    max={TIMER_MAX_MINUTES}
                    step={TIMER_STEP_MINUTES}
                    value={mt.timer.minutes}
                    onChange={(e) =>
                      mt.setTimerMinutes(Number(e.currentTarget.value))
                    }
                    aria-label="auto-stop minutes"
                  />
                  <button
                    className="nudge"
                    type="button"
                    onClick={() =>
                      mt.setTimerMinutes(mt.timer.minutes + TIMER_STEP_MINUTES)
                    }
                    aria-label="increase auto-stop timer"
                  >
                    <Plus size={16} strokeWidth={2.6} />
                  </button>
                  <span className="timer-minutes">{mt.timer.minutes} min</span>
                </div>
              </section>

              <section className="panel mode-panel">
                <h2 className="panel-title">Modes</h2>
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
                  onChange={(v) =>
                    mt.setParam('shepardDirection', v as Direction)
                  }
                />
                <Segmented
                  caption="Emergence spawn"
                  options={SPAWN_MODES}
                  value={state.spawnMode}
                  onChange={(v) => mt.setParam('spawnMode', v as SpawnMode)}
                />
              </section>

              <JourneyPanel mt={mt} />
            </div>
          )}

          {activeTab === 'shape' && (
            <section className="tab-panel panel sliders">
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
          )}
        </main>

        <footer className="studio-footer">
          engine · microtube-core (Rust → WebAssembly) ·{' '}
          {Math.round(mt.uptimeSecs)}s session
        </footer>
      </div>
    </div>
  );
}
