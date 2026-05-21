import { Gauge, Headphones, Minus, Orbit, Plus, Shapes } from 'lucide-react';
import { useState } from 'react';
import {
  DIRECTIONS,
  EEG_BANDS,
  MISTS,
  SLIDER_GROUPS,
  SPAWN_MODES,
  TIMBRES,
  VOLUME,
  eegBandIndex,
  type Direction,
  type MicroTubeState,
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
import { Panel } from './components/Panel';
import { ParameterSlider } from './components/ParameterSlider';
import { Segmented } from './components/Segmented';
import { StripDashboard } from './components/StripDashboard';
import { TopBar } from './components/TopBar';

type StudioTab = 'play' | 'shape';

const TABS: Array<{ id: StudioTab; label: string; caption: string }> = [
  { id: 'play', label: 'Play', caption: 'basic' },
  { id: 'shape', label: 'Shape', caption: 'advanced' },
];

/**
 * Mode-style controls coupled to a gain parameter — touching one of these
 * while its function is silent should engage the function automatically, so
 * the user never has to hunt for an on/off switch on another tab.
 */
const GAIN_FOR = {
  mistType: 'noiseLevel',
  spawnMode: 'emergence',
  shepardDirection: 'shepard',
  shepardBase: 'shepard',
} as const;

/** The level a coupled function jumps to when auto-engaged from silence. */
const AUTO_ON_VALUE = {
  noiseLevel: 0.35,
  emergence: 0.45,
  shepard: 0.4,
} as const;

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
          <div className="start-stars" aria-hidden="true" />
          <div className="start-orbits" aria-hidden="true">
            <span className="start-orbit start-orbit-1" />
            <span className="start-orbit start-orbit-2" />
            <span className="start-orbit start-orbit-3" />
          </div>
          <div className="start-mark">
            micro<span>tube</span>
          </div>
          <p className="start-tagline">Tune your mind to a frequency.</p>
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
            Headphones recommended. The binaural effect lives in the gap between
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

  /**
   * Set a parameter, then engage its coupled function if it was silent —
   * e.g. picking a mist colour turns the mist layer on by itself.
   */
  const setParam = <K extends keyof MicroTubeState>(
    key: K,
    value: MicroTubeState[K],
  ) => {
    mt.setParam(key, value);
    const gainKey = GAIN_FOR[key as keyof typeof GAIN_FOR] as
      | keyof typeof AUTO_ON_VALUE
      | undefined;
    if (gainKey && state[gainKey] === 0) {
      mt.setParam(gainKey, AUTO_ON_VALUE[gainKey]);
    }
  };

  const applyPreset = (preset: Preset) => {
    mt.setParam('beatFreq', preset.beatFreq);
    mt.setParam('baseFreq', preset.baseFreq);
    mt.setParam('noiseLevel', preset.noiseLevel);
  };

  return (
    <div className="app">
      <div className="studio-shell" style={{ ['--accent' as string]: accent }}>
        <TopBar
          playing={state.playing}
          onToggle={mt.togglePlaying}
          beatFreq={state.beatFreq}
          timer={mt.timer}
        />

        <StripDashboard beatFreq={state.beatFreq} onApplyPreset={applyPreset} />

        <nav className="studio-tabs" aria-label="Studio sections">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              className={`studio-tab${activeTab === tab.id ? ' active' : ''}`}
              type="button"
              onClick={() => setActiveTab(tab.id)}
              onContextMenu={(e) => e.preventDefault()}
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
              <Panel icon={Gauge} title="Transport" className="transport-panel">
                <div className="transport-vol">
                  <ParameterSlider
                    spec={VOLUME}
                    value={state.volume}
                    onChange={(v) => setParam('volume', v)}
                  />
                </div>
                <div className="timer-block">
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
                        mt.setTimerMinutes(
                          mt.timer.minutes - TIMER_STEP_MINUTES,
                        )
                      }
                      onContextMenu={(e) => e.preventDefault()}
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
                        mt.setTimerMinutes(
                          mt.timer.minutes + TIMER_STEP_MINUTES,
                        )
                      }
                      onContextMenu={(e) => e.preventDefault()}
                      aria-label="increase auto-stop timer"
                    >
                      <Plus size={16} strokeWidth={2.6} />
                    </button>
                    <span className="timer-minutes">
                      {mt.timer.minutes} min
                    </span>
                  </div>
                </div>
              </Panel>

              <Panel icon={Shapes} title="Modes">
                <Segmented
                  caption="Timbre"
                  options={TIMBRES}
                  value={state.timbre}
                  onChange={(v) => setParam('timbre', v as Timbre)}
                />
                <Segmented
                  caption="Mist colour"
                  options={MISTS}
                  value={state.mistType}
                  enabled={state.noiseLevel > 0}
                  onChange={(v) => setParam('mistType', v as MistType)}
                />
                <Segmented
                  caption="Drift direction"
                  options={DIRECTIONS}
                  value={state.shepardDirection}
                  enabled={state.shepard > 0}
                  onChange={(v) =>
                    setParam('shepardDirection', v as Direction)
                  }
                />
                <Segmented
                  caption="Emergence spawn"
                  options={SPAWN_MODES}
                  value={state.spawnMode}
                  enabled={state.emergence > 0}
                  onChange={(v) => setParam('spawnMode', v as SpawnMode)}
                />
              </Panel>

              <Panel
                icon={Orbit}
                title="Journey Through the Cosmos"
                className="journey-panel"
              >
                <JourneyPanel mt={mt} />
              </Panel>
            </div>
          )}

          {activeTab === 'shape' && (
            <div className="tab-panel shape-tab">
              {SLIDER_GROUPS.map((group) => (
                <Panel
                  key={group.id}
                  icon={group.icon}
                  title={group.label}
                  caption={group.caption}
                  className="slider-group"
                >
                  <div className="slider-stack">
                    {group.sliders.map((spec) => (
                      <ParameterSlider
                        key={spec.key}
                        spec={spec}
                        value={state[spec.key]}
                        onChange={(v) => setParam(spec.key, v)}
                      />
                    ))}
                  </div>
                </Panel>
              ))}
            </div>
          )}
        </main>

        <footer className="studio-footer">
          <a href="https://github.com/arkanine1000/microtube" target='_blank' rel="noopener noreferrer">microtube</a> engine · ars gratia artis
        </footer>
      </div>
    </div>
  );
}
