import { Headphones } from 'lucide-react';
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
import { BandChips } from './components/BandChips';
import { HeaderBar } from './components/HeaderBar';
import { LanguageSelector } from './components/LanguageSelector';
import { SectionBar } from './components/SectionBar';
import { SlimSlider } from './components/SlimSlider';
import { DriftSection } from './components/sections/DriftSection';
import { EmergenceSection } from './components/sections/EmergenceSection';
import { MistSection } from './components/sections/MistSection';
import { SequencesSection } from './components/sections/SequencesSection';
import { SignalSection } from './components/sections/SignalSection';
import { useLocale } from './i18n/LocaleProvider';
import { SECTIONS, type SectionId } from './sections';

/** The level a coupled function jumps to when auto-engaged from silence. */
const AUTO_ON_VALUE = {
  noiseLevel: 0.35,
  emergence: 0.45,
  shepard: 0.4,
} as const;

const SLIDER_BY_KEY = Object.fromEntries(
  SLIDERS.map((spec) => [spec.key, spec]),
) as Record<SliderKey, SliderSpec>;

/**
 * The active deck section, kept in module scope so it survives start-screen
 * round trips (the studio unmounts entirely). Deliberately not persisted —
 * a page reload starts back at the signal controls.
 */
let lastSection: SectionId = 'signal';

export default function App() {
  const mt = useMicroTube();
  const { copy } = useLocale();
  const [section, setSection] = useState<SectionId>(lastSection);
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

  const selectSection = (id: SectionId) => {
    lastSection = id;
    setSection(id);
  };

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

  const sectionBody: Record<SectionId, () => JSX.Element> = {
    signal: () => (
      <SignalSection
        state={state}
        slider={paramSlider}
        onTimbre={(v: Timbre) => setParam('timbre', v)}
      />
    ),
    mist: () => (
      <MistSection
        state={state}
        slider={paramSlider}
        onOption={(v: MistType) => setFeatureOption('mistType', v, 'noiseLevel')}
        onDisable={() => disableFeature('noiseLevel')}
      />
    ),
    emergence: () => (
      <EmergenceSection
        state={state}
        slider={paramSlider}
        onOption={(v: SpawnMode) => setFeatureOption('spawnMode', v, 'emergence')}
        onDisable={() => disableFeature('emergence')}
      />
    ),
    drift: () => (
      <DriftSection
        state={state}
        slider={paramSlider}
        onOption={(v: Direction) =>
          setFeatureOption('shepardDirection', v, 'shepard')
        }
        onDisable={() => disableFeature('shepard')}
      />
    ),
    sequences: () => <SequencesSection mt={mt} />,
  };

  return (
    <div className="app">
      <div className="studio-shell" style={{ ['--accent' as string]: accent }}>
        <HeaderBar
          mt={mt}
          presets={localPresets}
          onPresetsChange={setLocalPresets}
        />

        <BandChips beatFreq={state.beatFreq} onApplyPreset={applyPreset} />

        <main className="deck-stage">
          {SECTIONS.map(({ id, isOn }) => (
            <section
              key={id}
              className={`deck-section ${id}-section${
                id === section ? ' active' : ''
              }${isOn && !isOn(state, mt.sequence.active) ? ' off' : ''}`}
            >
              {sectionBody[id]()}
            </section>
          ))}
        </main>

        <SectionBar
          active={section}
          onSelect={selectSection}
          state={state}
          sequenceActive={mt.sequence.active}
        />

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
