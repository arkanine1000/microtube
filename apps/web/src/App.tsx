import {
  CloudFog,
  Flame,
  Headphones,
  Orbit,
  RadioTower,
  Sparkles,
  Waves,
} from 'lucide-react';
import { useState } from 'react';
import { loadLocalPresets, snapshotFromState } from './audio/localPresets';
import {
  EEG_BANDS,
  SLIDERS,
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
import { useMicroTube } from './audio/useMicroTube';
import { HeaderBar } from './components/HeaderBar';
import { LanguageSelector } from './components/LanguageSelector';
import { Panel } from './components/Panel';
import { SlimSlider } from './components/SlimSlider';
import { SequencesPanel } from './components/SequencesPanel';
import { Segmented } from './components/Segmented';
import { StripDashboard } from './components/StripDashboard';
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
  const accent = EEG_BANDS[eegBandIndex(state.beatFreq)].color;

  const setParam = <K extends keyof MicroTubeState>(
    key: K,
    value: MicroTubeState[K],
  ) => {
    mt.setParam(key, value);
  };

  const paramSlider = (key: SliderKey) => (
    <SlimSlider
      spec={SLIDER_BY_KEY[key]}
      label={copy.sliders[key].label}
      hint={copy.sliders[key].hint}
      value={state[key]}
      onChange={(v) => setParam(key, v)}
    />
  );

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
        <HeaderBar
          mt={mt}
          presets={localPresets}
          onPresetsChange={setLocalPresets}
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
                id="carrier"
                icon={RadioTower}
                title={copy.panels.carrier}
                className="slider-group carrier-panel"
              >
                <div className="slider-stack">
                  {paramSlider('baseFreq')}
                  {paramSlider('beatFreq')}
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
                {paramSlider('harmonics')}
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
                    {paramSlider('noiseLevel')}
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
                    {paramSlider('emergence')}
                    {paramSlider('gravity')}
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
                    {paramSlider('shepardBase')}
                    {paramSlider('shepard')}
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
