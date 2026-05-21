import {
  Bookmark,
  Gauge,
  Headphones,
  Minus,
  Orbit,
  Plus,
  Shapes,
} from 'lucide-react';
import { useState } from 'react';
import { loadLocalPresets, snapshotFromState } from './audio/localPresets';
import {
  EEG_BANDS,
  SLIDER_GROUPS,
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
import { LanguageSelector } from './components/LanguageSelector';
import { LocalPresetsPanel } from './components/LocalPresetsPanel';
import { Panel } from './components/Panel';
import { ParameterSlider } from './components/ParameterSlider';
import { Segmented } from './components/Segmented';
import { StripDashboard } from './components/StripDashboard';
import { TopBar } from './components/TopBar';
import { useLocale } from './i18n/LocaleProvider';
import type { StudioTab } from './i18n/copy';

const STUDIO_TABS: StudioTab[] = ['play', 'shape'];

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

function formatClock(secs: number | null, offLabel: string): string {
  if (secs === null) return offLabel;
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
  const { copy } = useLocale();
  const [activeTab, setActiveTab] = useState<StudioTab>('play');
  const [localPresets, setLocalPresets] = useState(loadLocalPresets);

  if (mt.status !== 'running') {
    return (
      <div className="app">
        <div className="start" aria-live="polite">
          <LanguageSelector />
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
          <p className="start-tagline">{copy.start.tagline}</p>
          <button
            className="btn btn-primary btn-enter"
            type="button"
            disabled={mt.status === 'loading'}
            onClick={mt.start}
          >
            {mt.status === 'loading' ? copy.start.loading : copy.start.enter}
          </button>
          <div className="start-headphones">
            <Headphones size={15} strokeWidth={2.1} />
            {copy.start.headphones}
          </div>
          {mt.status === 'error' && (
            <p className="error">
              {copy.start.errorPrefix} {mt.error}
            </p>
          )}
        </div>
      </div>
    );
  }

  const { state } = mt;
  const timerLabel = mt.timer.fired
    ? copy.timer.stopped
    : formatClock(mt.timer.remainingSecs, copy.timer.off);
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
    mt.applySnapshot({
      ...snapshotFromState(mt.state),
      beatFreq: preset.beatFreq,
      baseFreq: preset.baseFreq,
      noiseLevel: preset.noiseLevel,
    });
  };

  return (
    <div className="app">
      <div className="studio-shell" style={{ ['--accent' as string]: accent }}>
        <TopBar
          playing={state.playing}
          onToggle={mt.togglePlaying}
          onBrandClick={mt.returnToStart}
          beatFreq={state.beatFreq}
          timer={mt.timer}
        />

        <StripDashboard beatFreq={state.beatFreq} onApplyPreset={applyPreset} />

        <nav className="studio-tabs" aria-label={copy.studioSectionsLabel}>
          {STUDIO_TABS.map((tab) => (
            <button
              key={tab}
              className={`studio-tab${activeTab === tab ? ' active' : ''}`}
              type="button"
              onClick={() => setActiveTab(tab)}
              onContextMenu={(e) => e.preventDefault()}
              aria-current={activeTab === tab ? 'page' : undefined}
            >
              <span>{copy.tabs[tab].label}</span>
              <small>{copy.tabs[tab].caption}</small>
            </button>
          ))}
        </nav>

        <main className="studio-stage">
          {activeTab === 'play' && (
            <div className="tab-panel play-tab">
              <Panel
                id="transport"
                icon={Gauge}
                title={copy.panels.transport}
                className="transport-panel"
              >
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
                      <span>{copy.timer.autoStop}</span>
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
                      aria-label={copy.timer.decrease}
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
                      aria-label={copy.timer.minutes}
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
                      aria-label={copy.timer.increase}
                    >
                      <Plus size={16} strokeWidth={2.6} />
                    </button>
                    <span className="timer-minutes">
                      {mt.timer.minutes} {copy.timer.minutesAbbrev}
                    </span>
                  </div>
                </div>
              </Panel>

              <Panel
                id="local-presets"
                icon={Bookmark}
                title={copy.panels.presets}
                className="presets-panel"
              >
                <LocalPresetsPanel
                  mt={mt}
                  presets={localPresets}
                  onPresetsChange={setLocalPresets}
                />
              </Panel>

              <Panel
                id="modes"
                icon={Shapes}
                title={copy.panels.modes}
                className="modes-panel"
              >
                <Segmented
                  caption={copy.modes.captions.timbre}
                  options={copy.modes.timbres}
                  value={state.timbre}
                  statusLabels={copy.modes.status}
                  onChange={(v) => setParam('timbre', v as Timbre)}
                />
                <Segmented
                  caption={copy.modes.captions.mist}
                  options={copy.modes.mists}
                  value={state.mistType}
                  enabled={state.noiseLevel > 0}
                  statusLabels={copy.modes.status}
                  onChange={(v) => setParam('mistType', v as MistType)}
                />
                <Segmented
                  caption={copy.modes.captions.direction}
                  options={copy.modes.directions}
                  value={state.shepardDirection}
                  enabled={state.shepard > 0}
                  statusLabels={copy.modes.status}
                  onChange={(v) =>
                    setParam('shepardDirection', v as Direction)
                  }
                />
                <Segmented
                  caption={copy.modes.captions.spawn}
                  options={copy.modes.spawnModes}
                  value={state.spawnMode}
                  enabled={state.emergence > 0}
                  statusLabels={copy.modes.status}
                  onChange={(v) => setParam('spawnMode', v as SpawnMode)}
                />
              </Panel>

              <Panel
                id="journey"
                icon={Orbit}
                title={copy.panels.journey}
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
                  id={`slider-${group.id}`}
                  icon={group.icon}
                  title={copy.sliderGroups[group.id].label}
                  caption={copy.sliderGroups[group.id].caption}
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
          <a
            href="https://github.com/arkanine1000/microtube"
            target="_blank"
            rel="noopener noreferrer"
          >
            microtube
          </a>{' '}
          {copy.footer.engine} · ars gratia artis
        </footer>
      </div>
    </div>
  );
}
