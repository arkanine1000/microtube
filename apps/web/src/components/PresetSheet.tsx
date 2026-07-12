import type { LocalPreset } from '../audio/localPresets';
import { EEG_BANDS, eegBandIndex } from '../audio/params';
import type { MicroTube } from '../audio/useMicroTube';
import { useLocale } from '../i18n/LocaleProvider';
import { LocalPresetsPanel } from './LocalPresetsPanel';
import { Modal } from './Modal';

/** The header's quick save/load surface — LocalPresetsPanel in a bottom sheet. */
export function PresetSheet({
  mt,
  presets,
  onPresetsChange,
  onClose,
}: {
  mt: MicroTube;
  presets: LocalPreset[];
  onPresetsChange: (presets: LocalPreset[]) => void;
  onClose: () => void;
}) {
  const { copy } = useLocale();
  const accent = EEG_BANDS[eegBandIndex(mt.state.beatFreq)].color;

  return (
    <Modal
      title={copy.header.presets}
      closeLabel={copy.localPresets.close}
      accent={accent}
      variant="sheet"
      onClose={onClose}
    >
      <LocalPresetsPanel
        mt={mt}
        presets={presets}
        onPresetsChange={onPresetsChange}
      />
    </Modal>
  );
}
