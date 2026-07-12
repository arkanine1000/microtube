import { Sparkles } from 'lucide-react';
import type { MicroTubeState, SpawnMode } from '../../audio/params';
import { useLocale } from '../../i18n/LocaleProvider';
import { Segmented } from '../Segmented';
import { SectionHead, type SliderRender } from './shared';

export function EmergenceSection({
  state,
  slider,
  onOption,
  onDisable,
}: {
  state: MicroTubeState;
  slider: SliderRender;
  onOption: (v: SpawnMode) => void;
  onDisable: () => void;
}) {
  const { copy } = useLocale();
  return (
    <>
      <SectionHead icon={Sparkles} title={copy.sections.emergence.label} />
      <Segmented
        caption={copy.modes.captions.spawn}
        options={copy.modes.spawnModes}
        value={state.spawnMode}
        enabled={state.emergence > 0}
        statusLabels={copy.modes.status}
        onChange={(v) => onOption(v as SpawnMode)}
        onDisable={onDisable}
      />
      {state.emergence > 0 && (
        <div className="feature-controls">
          {slider('emergence')}
          {slider('gravity')}
        </div>
      )}
    </>
  );
}
