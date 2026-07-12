import { EEG_BANDS, eegBandIndex } from '../audio/params';
import { PRESETS, type Preset } from '../audio/sequences';
import { useLocale } from '../i18n/LocaleProvider';

/**
 * The EEG band ladder as a compact chip row — each chip *is* its preset
 * button (the five presets map one-to-one onto the five bands). Tapping a
 * band stays the lowest-friction way to explore the engine. Mobile shows
 * greek + Hz; wider screens reveal the preset name via CSS.
 */
export function BandChips({
  beatFreq,
  onApplyPreset,
}: {
  beatFreq: number;
  onApplyPreset: (preset: Preset) => void;
}) {
  const { copy } = useLocale();
  const active = eegBandIndex(beatFreq);

  return (
    <div className="band-chips">
      {EEG_BANDS.map((band, i) => {
        const preset = PRESETS[i];
        const presetCopy = copy.presets[i];
        const inBand = i === active;
        const exact = Math.abs(preset.beatFreq - beatFreq) < 0.05;
        return (
          <button
            key={band.id}
            className={`band-chip${inBand ? ' in-band' : ''}${
              exact ? ' on' : ''
            }`}
            style={{ color: band.color }}
            type="button"
            onClick={() => onApplyPreset(preset)}
            onContextMenu={(e) => e.preventDefault()}
            title={presetCopy.description}
            aria-pressed={exact}
          >
            <span className="bc-greek">{band.greek}</span>
            <span className="bc-name">{presetCopy.name}</span>
            <span className="bc-hz">{preset.beatFreq} Hz</span>
          </button>
        );
      })}
    </div>
  );
}
