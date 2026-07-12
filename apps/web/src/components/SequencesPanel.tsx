import { Play, Square } from 'lucide-react';
import { SEQUENCES, type SequenceId } from '../audio/sequences';
import type { MicroTube } from '../audio/useMicroTube';
import { useLocale } from '../i18n/LocaleProvider';

function mmss(secs: number): string {
  const total = Math.max(0, Math.round(secs));
  const m = Math.floor(total / 60);
  const s = total % 60;
  return `${m}:${String(s).padStart(2, '0')}`;
}

/**
 * The timed-program list as compact rows: inline start/stop, name and live
 * step readout, duration or remaining time, and a hairline progress bar
 * along the row's lower edge while running.
 */
export function SequencesPanel({ mt }: { mt: MicroTube }) {
  const { copy } = useLocale();

  return (
    <div className="sequences-panel">
      <p className="sequence-copy">{copy.sequences.intro}</p>
      <div className="sequence-list" role="list">
        {SEQUENCES.map((sequence) => {
          const text = copy.sequences.cards[sequence.id];
          const active =
            mt.sequence.activeId === sequence.id && mt.sequence.active;
          const progress =
            active && mt.sequence.total > 0
              ? (mt.sequence.elapsed / mt.sequence.total) * 100
              : 0;
          const stepName =
            text.steps[mt.sequence.stepIndex] ??
            `${copy.sequences.stepPrefix} ${mt.sequence.stepIndex + 1}`;

          return (
            <article
              className={`sequence-row${active ? ' active' : ''}`}
              key={sequence.id}
              role="listitem"
            >
              <button
                className={`sequence-toggle${active ? ' running' : ''}`}
                type="button"
                onClick={() =>
                  active
                    ? mt.stopSequence()
                    : mt.startSequence(sequence.id as SequenceId)
                }
                onContextMenu={(e) => e.preventDefault()}
                aria-label={`${
                  active ? copy.sequences.stop : copy.sequences.start
                } · ${text.name}`}
              >
                {active ? (
                  <Square size={15} strokeWidth={2.4} fill="currentColor" />
                ) : (
                  <Play size={15} strokeWidth={2.4} fill="currentColor" />
                )}
              </button>
              <div className="sequence-titles">
                <span className="sequence-name">{text.name}</span>
                <span className="sequence-desc">
                  {active
                    ? `${copy.sequences.stepPrefix} ${
                        mt.sequence.stepIndex + 1
                      }/${sequence.steps.length} · ${stepName}`
                    : text.description}
                </span>
              </div>
              <span className="sequence-meta">
                {active
                  ? mmss(mt.sequence.total - mt.sequence.elapsed)
                  : mmss(sequence.totalSecs)}
              </span>
              {active && (
                <div className="sequence-hairline" aria-hidden="true">
                  <div
                    className="sequence-hairline-fill"
                    style={{ width: `${progress}%` }}
                  />
                </div>
              )}
            </article>
          );
        })}
      </div>
    </div>
  );
}
