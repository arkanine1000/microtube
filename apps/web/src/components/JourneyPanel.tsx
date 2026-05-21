import { Orbit, Square } from 'lucide-react';
import type { MicroTube } from '../audio/useMicroTube';

function mmss(secs: number): string {
  const total = Math.floor(secs);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

/**
 * "Journey Through the Cosmos" — the 13-step strange loop, executed on the
 * frontend with a 250 ms scheduler. Quick presets now live in the strip;
 * this panel is the guided long-form experience.
 */
export function JourneyPanel({ mt }: { mt: MicroTube }) {
  const { journey } = mt;
  const progress =
    journey.total > 0 ? (journey.elapsed / journey.total) * 100 : 0;

  return (
    <section className="panel journey-panel">
      <div className="journey-head">
        <h2 className="panel-title">
          <Orbit size={13} strokeWidth={2.3} />
          <span>Journey Through the Cosmos</span>
        </h2>
        <span className="journey-meta">13-step strange loop · ~25 min</span>
      </div>

      <p className="journey-copy">
        A guided descent and return — every parameter automated, interpolating
        between thirteen named worlds. Hand the controls to the sequence and
        listen.
      </p>

      <div className="journey-bar">
        <div className="journey-bar-fill" style={{ width: `${progress}%` }} />
      </div>
      <div className="journey-status">
        <span>
          {journey.active
            ? `Step ${journey.stepIndex + 1}/13 · ${journey.stepName}`
            : 'Idle — press begin to set off'}
        </span>
        <span>
          {mmss(journey.elapsed)} / {mmss(journey.total)}
        </span>
      </div>

      <button
        className={`btn journey-action${journey.active ? '' : ' btn-primary'}`}
        type="button"
        onClick={journey.active ? mt.stopJourney : mt.startJourney}
      >
        {journey.active ? (
          <>
            <Square size={15} strokeWidth={2.6} fill="currentColor" />
            Stop journey
          </>
        ) : (
          <>
            <Orbit size={16} strokeWidth={2.2} />
            Begin journey
          </>
        )}
      </button>
    </section>
  );
}
