import { EEG_BANDS, eegBandIndex } from '../audio/params';
import { PRESETS, type Preset } from '../audio/sequences';

/**
 * The persistent strip — the EEG band ladder, where each band cell *is* its
 * preset button (the five presets map one-to-one onto the five bands).
 * Rendered once in the shell, above the tab nav, so it stays available on
 * every tab: tapping a band is the lowest-friction way to explore the engine.
 */
export function StripDashboard({
  beatFreq,
  onApplyPreset,
}: {
  beatFreq: number;
  onApplyPreset: (preset: Preset) => void;
}) {
  const active = eegBandIndex(beatFreq);

  return (
    <section className="strip">
      <div className="preset-bands">
        {EEG_BANDS.map((band, i) => {
          const preset = PRESETS[i];
          const inBand = i === active;
          const exact = Math.abs(preset.beatFreq - beatFreq) < 0.05;
          return (
            <button
              key={band.name}
              className={`preset-band${inBand ? ' in-band' : ''}${
                exact ? ' on' : ''
              }`}
              style={{ color: band.color }}
              type="button"
              onClick={() => onApplyPreset(preset)}
              onContextMenu={(e) => e.preventDefault()}
              title={preset.description}
              aria-pressed={exact}
            >
              <span className="pb-greek">{band.greek}</span>
              <span className="pb-name">{preset.name}</span>
              <span className="pb-meta">
                {band.name} · {preset.beatFreq} Hz
              </span>
            </button>
          );
        })}
      </div>
    </section>
  );
}
