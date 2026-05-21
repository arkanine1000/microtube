import { Orbit, Square } from 'lucide-react';
import type { MicroTube } from '../audio/useMicroTube';
import { useLocale } from '../i18n/LocaleProvider';

function mmss(secs: number): string {
  const total = Math.floor(secs);
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

/**
 * "Journey Through the Cosmos" — the 13-step strange loop, executed on the
 * frontend with a 250 ms scheduler. Rendered inside a collapsible Panel, so
 * this component only supplies the body content.
 */
export function JourneyPanel({ mt }: { mt: MicroTube }) {
  const { copy } = useLocale();
  const { journey } = mt;
  const progress =
    journey.total > 0 ? (journey.elapsed / journey.total) * 100 : 0;
  const stepName = copy.journey.steps[journey.stepIndex];

  return (
    <>
      <p className="journey-copy">
        {copy.journey.copy}
      </p>

      <div className="journey-progress">
        <div className="journey-bar">
          <div
            className="journey-bar-fill"
            style={{ width: `${progress}%` }}
          />
        </div>
        <div className="journey-status">
          <span>
            {journey.active
              ? `${copy.journey.stepPrefix} ${journey.stepIndex + 1}/${
                  copy.journey.steps.length
                } · ${stepName}`
              : copy.journey.idle}
          </span>
          <span>
            {mmss(journey.elapsed)} / {mmss(journey.total)}
          </span>
        </div>
      </div>

      <button
        className={`btn journey-action${journey.active ? '' : ' btn-primary'}`}
        type="button"
        onClick={journey.active ? mt.stopJourney : mt.startJourney}
      >
        {journey.active ? (
          <>
            <Square size={15} strokeWidth={2.6} fill="currentColor" />
            {copy.journey.stop}
          </>
        ) : (
          <>
            <Orbit size={16} strokeWidth={2.2} />
            {copy.journey.begin}
          </>
        )}
      </button>
    </>
  );
}
