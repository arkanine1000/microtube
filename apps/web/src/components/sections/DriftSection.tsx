import { Waves } from 'lucide-react';
import type { Direction, MicroTubeState } from '../../audio/params';
import { useLocale } from '../../i18n/LocaleProvider';
import { Segmented } from '../Segmented';
import { SectionHead, type SliderRender } from './shared';

export function DriftSection({
  state,
  slider,
  onOption,
  onDisable,
}: {
  state: MicroTubeState;
  slider: SliderRender;
  onOption: (v: Direction) => void;
  onDisable: () => void;
}) {
  const { copy } = useLocale();
  return (
    <>
      <SectionHead icon={Waves} title={copy.sections.drift.label} />
      <Segmented
        caption={copy.modes.captions.direction}
        options={copy.modes.directions}
        value={state.shepardDirection}
        enabled={state.shepard > 0}
        statusLabels={copy.modes.status}
        onChange={(v) => onOption(v as Direction)}
        onDisable={onDisable}
      />
      {state.shepard > 0 && (
        <div className="feature-controls drift-controls">
          {slider('shepardBase')}
          {slider('shepard')}
        </div>
      )}
    </>
  );
}
