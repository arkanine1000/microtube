import { EEG_BANDS, eegBandIndex } from '../audio/params';
import { PRESETS, type Preset } from '../audio/sequences';

/**
 * The persistent strip — preset launchpad plus the EEG band ladder. Rendered
 * once in the shell, above the tab nav, so it stays available on every tab.
 * Tapping a preset chip is the lowest-friction way to explore the engine.
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
      <div className="strip-section">
        <span className="strip-label">Presets</span>
        <div className="preset-chips">
          {PRESETS.map((preset) => {
            const on = Math.abs(preset.beatFreq - beatFreq) < 0.05;
            return (
              <button
                key={preset.name}
                className={`preset-chip${on ? ' on' : ''}`}
                type="button"
                onClick={() => onApplyPreset(preset)}
                title={preset.description}
                aria-pressed={on}
              >
                <b>{preset.name}</b>
                <small>{preset.beatFreq} Hz</small>
              </button>
            );
          })}
        </div>
      </div>

      <div className="strip-section">
        <span className="strip-label">EEG band</span>
        <div className="bands">
          {EEG_BANDS.map((band, i) => (
            <div
              key={band.name}
              className={`band${i === active ? ' active' : ''}`}
              style={{ color: band.color }}
            >
              <div className="band-greek">{band.greek}</div>
              <div className="band-name">{band.name}</div>
              <div className="band-blurb">{band.blurb}</div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
