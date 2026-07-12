import { RadioTower } from 'lucide-react';
import { useLocale } from '../../i18n/LocaleProvider';
import { SectionHead, type SliderRender } from './shared';

export function CarrierSection({ slider }: { slider: SliderRender }) {
  const { copy } = useLocale();
  return (
    <>
      <SectionHead icon={RadioTower} title={copy.sections.carrier.label} />
      {slider('baseFreq')}
      {slider('beatFreq')}
    </>
  );
}
