import { AudioWaveform } from 'lucide-react';
import type { MicroTubeState, Timbre } from '../../audio/params';
import { useLocale } from '../../i18n/LocaleProvider';
import { Segmented } from '../Segmented';
import { SectionHead, type SliderRender } from './shared';

export function SignalSection({
  state,
  slider,
  onTimbre,
}: {
  state: MicroTubeState;
  slider: SliderRender;
  onTimbre: (v: Timbre) => void;
}) {
  const { copy } = useLocale();
  return (
    <>
      <SectionHead icon={AudioWaveform} title={copy.sections.signal.label} />
      {slider('baseFreq')}
      {slider('beatFreq')}
      <Segmented
        caption={copy.modes.captions.timbre}
        options={copy.modes.timbres}
        value={state.timbre}
        onChange={(v) => onTimbre(v as Timbre)}
      />
      {slider('harmonics')}
    </>
  );
}
