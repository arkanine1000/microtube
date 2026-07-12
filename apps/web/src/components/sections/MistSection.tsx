import { CloudFog } from 'lucide-react';
import type { MicroTubeState, MistType } from '../../audio/params';
import { useLocale } from '../../i18n/LocaleProvider';
import { Segmented } from '../Segmented';
import { SectionHead, type SliderRender } from './shared';

export function MistSection({
  state,
  slider,
  onOption,
  onDisable,
}: {
  state: MicroTubeState;
  slider: SliderRender;
  onOption: (v: MistType) => void;
  onDisable: () => void;
}) {
  const { copy } = useLocale();
  return (
    <>
      <SectionHead icon={CloudFog} title={copy.sections.mist.label} />
      <Segmented
        className="mist-options"
        caption={copy.modes.captions.mist}
        options={copy.modes.mists}
        value={state.mistType}
        enabled={state.noiseLevel > 0}
        statusLabels={copy.modes.status}
        onChange={(v) => onOption(v as MistType)}
        onDisable={onDisable}
      />
      {state.noiseLevel > 0 && (
        <div className="feature-controls">{slider('noiseLevel')}</div>
      )}
    </>
  );
}
