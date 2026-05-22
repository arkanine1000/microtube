import {
  Bookmark,
  CloudFog,
  Flame,
  Gauge,
  Headphones,
  Minus,
  Orbit,
  Plus,
  RadioTower,
  Sparkles,
  Waves,
} from 'lucide-react';
import { useState } from 'react';
import { loadLocalPresets, snapshotFromState } from './audio/localPresets';
import {
  EEG_BANDS,
  SLIDERS,
  VOLUME,
  eegBandIndex,
  type Direction,
  type MicroTubeState,
  type MistType,
  type SliderKey,
  type SliderSpec,
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
import { LanguageSelector } from './components/LanguageSelector';
import { LocalPresetsPanel } from './components/LocalPresetsPanel';
import { Panel } from './components/Panel';
import { ParameterSlider } from './components/ParameterSlider';
import { SequencesPanel } from './components/SequencesPanel';
import { Segmented } from './components/Segmented';
import { StripDashboard } from './components/StripDashboard';
import { TopBar } from './components/TopBar';
import { useLocale } from './i18n/LocaleProvider';
import type { StudioTab } from './i18n/copy';

const STUDIO_TABS: StudioTab[] = ['main', 'sequences'];

/** The level a coupled function jumps to when auto-engaged from silence. */
const AUTO_ON_VALUE = {
  noiseLevel: 0.35,
  emergence: 0.45,
  shepard: 0.4,
} as const;

const SLIDER_BY_KEY = Object.fromEntries(
  SLIDERS.map((spec) => [spec.key, spec]),
) as Record<SliderKey, SliderSpec>;

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
  const [activeTab, setActiveTab] = useState<StudioTab>('main');
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

  const setParam = <K extends keyof MicroTubeState>(
    key: K,
    value: MicroTubeState[K],
  ) => {
    mt.setParam(key, value);
  };

  const setFeatureOption = <
    OptionKey extends 'mistType' | 'spawnMode' | 'shepardDirection',
    GainKey extends keyof typeof AUTO_ON_VALUE,
  >(
    optionKey: OptionKey,
    optionValue: MicroTubeState[OptionKey],
    gainKey: GainKey,
  ) => {
    mt.setParam(optionKey, optionValue);
    if (state[gainKey] === 0) {
      mt.setParam(gainKey, AUTO_ON_VALUE[gainKey]);
    }
  };

  const disableFeature = (gainKey: keyof typeof AUTO_ON_VALUE) => {
    mt.setParam(gainKey, 0);
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
          {activeTab === 'main' && (
            <div className="tab-panel main-tab">
              <Panel
                id="local-presets"
                icon={Bookmark}
                title={copy.panels.presets}
                className="presets-panel"
                defaultOpen={localPresets.length > 0}
              >
                <LocalPresetsPanel
                  mt={mt}
                  presets={localPresets}
                  onPresetsChange={setLocalPresets}
                />
              </Panel>

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
                id="carrier"
                icon={RadioTower}
                title={copy.panels.carrier}
                className="slider-group carrier-panel"
              >
                <div className="slider-stack">
                  <ParameterSlider
                    spec={SLIDER_BY_KEY.baseFreq}
                    value={state.baseFreq}
                    onChange={(v) => setParam('baseFreq', v)}
                  />
                  <ParameterSlider
                    spec={SLIDER_BY_KEY.beatFreq}
                    value={state.beatFreq}
                    onChange={(v) => setParam('beatFreq', v)}
                  />
                </div>
              </Panel>

              <Panel
                id="tone"
                icon={Flame}
                title={copy.panels.tone}
                className="tone-panel"
              >
                <Segmented
                  caption={copy.modes.captions.timbre}
                  options={copy.modes.timbres}
                  value={state.timbre}
                  onChange={(v) => setParam('timbre', v as Timbre)}
                />
                <ParameterSlider
                  spec={SLIDER_BY_KEY.harmonics}
                  value={state.harmonics}
                  onChange={(v) => setParam('harmonics', v)}
                />
              </Panel>

              <Panel
                id="mist"
                icon={CloudFog}
                title={copy.panels.mist}
                className={`feature-panel mist-panel${
                  state.noiseLevel > 0 ? '' : ' off'
                }`}
              >
                <Segmented
                  caption={copy.modes.captions.mist}
                  options={copy.modes.mists}
                  value={state.mistType}
                  enabled={state.noiseLevel > 0}
                  statusLabels={copy.modes.status}
                  onChange={(v) =>
                    setFeatureOption('mistType', v as MistType, 'noiseLevel')
                  }
                  onDisable={() => disableFeature('noiseLevel')}
                />
                {state.noiseLevel > 0 && (
                  <div className="feature-controls">
                    <ParameterSlider
                      spec={SLIDER_BY_KEY.noiseLevel}
                      value={state.noiseLevel}
                      onChange={(v) => setParam('noiseLevel', v)}
                    />
                  </div>
                )}
              </Panel>

              <Panel
                id="emergence"
                icon={Sparkles}
                title={copy.panels.emergence}
                className={`feature-panel emergence-panel${
                  state.emergence > 0 ? '' : ' off'
                }`}
              >
                <Segmented
                  caption={copy.modes.captions.spawn}
                  options={copy.modes.spawnModes}
                  value={state.spawnMode}
                  enabled={state.emergence > 0}
                  statusLabels={copy.modes.status}
                  onChange={(v) =>
                    setFeatureOption('spawnMode', v as SpawnMode, 'emergence')
                  }
                  onDisable={() => disableFeature('emergence')}
                />
                {state.emergence > 0 && (
                  <div className="feature-controls">
                    <ParameterSlider
                      spec={SLIDER_BY_KEY.emergence}
                      value={state.emergence}
                      onChange={(v) => setParam('emergence', v)}
                    />
                  </div>
                )}
              </Panel>

              <Panel
                id="drift"
                icon={Waves}
                title={copy.panels.drift}
                className={`feature-panel drift-panel${
                  state.shepard > 0 ? '' : ' off'
                }`}
              >
                <Segmented
                  caption={copy.modes.captions.direction}
                  options={copy.modes.directions}
                  value={state.shepardDirection}
                  enabled={state.shepard > 0}
                  statusLabels={copy.modes.status}
                  onChange={(v) =>
                    setFeatureOption(
                      'shepardDirection',
                      v as Direction,
                      'shepard',
                    )
                  }
                  onDisable={() => disableFeature('shepard')}
                />
                {state.shepard > 0 && (
                  <div className="feature-controls drift-controls">
                    <ParameterSlider
                      spec={SLIDER_BY_KEY.shepardBase}
                      value={state.shepardBase}
                      onChange={(v) => setParam('shepardBase', v)}
                    />
                    <ParameterSlider
                      spec={SLIDER_BY_KEY.shepard}
                      value={state.shepard}
                      onChange={(v) => setParam('shepard', v)}
                    />
                  </div>
                )}
              </Panel>
            </div>
          )}

          {activeTab === 'sequences' && (
            <div className="tab-panel sequences-tab">
              <Panel
                id="sequences"
                icon={Orbit}
                title={copy.tabs.sequences.label}
                className="sequences-shell-panel"
                defaultOpen
              >
                <SequencesPanel mt={mt} />
              </Panel>
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
