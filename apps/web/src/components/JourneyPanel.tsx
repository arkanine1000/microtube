import { PRESETS, type Preset } from '../audio/sequences';
import type { MicroTube } from '../audio/useMicroTube';

function mmss(secs: number): string {
  const total = Math.floor(secs);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

/**
 * Quick presets plus the "Journey Through the Cosmos" sequence — the
 * 13-step strange loop, executed on the frontend with a 250 ms scheduler.
 */
export function JourneyPanel({ mt }: { mt: MicroTube }) {
  const { journey } = mt;

  const applyPreset = (preset: Preset) => {
    mt.setParam('beatFreq', preset.beatFreq);
    mt.setParam('baseFreq', preset.baseFreq);
    mt.setParam('noiseLevel', preset.noiseLevel);
  };

  const progress = journey.total > 0 ? (journey.elapsed / journey.total) * 100 : 0;

  return (
    <section className="panel">
      <h2 className="panel-title">Presets &amp; Journey</h2>

      <div className="journey-grid">
        {PRESETS.map((preset) => (
          <button
            key={preset.name}
            className="preset"
            onClick={() => applyPreset(preset)}
          >
            <b>{preset.name}</b>
            <small>{preset.description}</small>
          </button>
        ))}
      </div>

      <div className="journey-bar">
        <div className="journey-bar-fill" style={{ width: `${progress}%` }} />
      </div>
      <div className="journey-status">
        <span>
          {journey.active
            ? `Step ${journey.stepIndex + 1}/13 · ${journey.stepName}`
            : 'Journey Through the Cosmos — 13-step strange loop'}
        </span>
        <span>
          {mmss(journey.elapsed)} / {mmss(journey.total)}
        </span>
      </div>

      <button
        className={`btn ${journey.active ? '' : 'btn-primary'}`}
        style={{ width: '100%', marginTop: 10 }}
        onClick={journey.active ? mt.stopJourney : mt.startJourney}
      >
        {journey.active ? 'Stop journey' : 'Begin journey'}
      </button>
    </section>
  );
}
