import { Orbit } from 'lucide-react';
import type { MicroTube } from '../../audio/useMicroTube';
import { useLocale } from '../../i18n/LocaleProvider';
import { SequencesPanel } from '../SequencesPanel';
import { SectionHead } from './shared';

export function SequencesSection({ mt }: { mt: MicroTube }) {
  const { copy } = useLocale();
  return (
    <>
      <SectionHead icon={Orbit} title={copy.sections.sequences.label} />
      <div className="sequences-scroll">
        <SequencesPanel mt={mt} />
      </div>
    </>
  );
}
