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

export function SequencesPanel({ mt }: { mt: MicroTube }) {
  const { copy } = useLocale();

  return (
    <div className="sequences-panel">
      <p className="sequence-copy">{copy.sequences.intro}</p>
      <div className="sequence-list" role="list">
        {SEQUENCES.map((sequence) => {
          const text = copy.sequences.cards[sequence.id];
          const active = mt.sequence.activeId === sequence.id && mt.sequence.active;
          const progress =
            active && mt.sequence.total > 0
              ? (mt.sequence.elapsed / mt.sequence.total) * 100
              : 0;
          const stepName =
            text.steps[mt.sequence.stepIndex] ??
            `${copy.sequences.stepPrefix} ${mt.sequence.stepIndex + 1}`;

          return (
            <article
              className={`sequence-card${active ? ' active' : ''}`}
              key={sequence.id}
              role="listitem"
            >
              <div className="sequence-card-head">
                <div>
                  <h2>{text.name}</h2>
                  <p>{text.description}</p>
                </div>
                <span className="sequence-duration">
                  {mmss(sequence.totalSecs)}
                </span>
              </div>
              <div className="sequence-progress">
                <div className="sequence-bar">
                  <div
                    className="sequence-bar-fill"
                    style={{ width: `${progress}%` }}
                  />
                </div>
                <div className="sequence-status">
                  <span>
                    {active
                      ? `${copy.sequences.running} · ${
                          copy.sequences.stepPrefix
                        } ${mt.sequence.stepIndex + 1}/${sequence.steps.length}`
                      : copy.sequences.idle}
                  </span>
                  <span>{active ? stepName : mmss(sequence.totalSecs)}</span>
                </div>
              </div>
              <button
                className={`btn sequence-action${active ? '' : ' btn-primary'}`}
                type="button"
                onClick={() =>
                  active
                    ? mt.stopSequence()
                    : mt.startSequence(sequence.id as SequenceId)
                }
              >
                {active ? (
                  <>
                    <Square size={16} strokeWidth={2.4} />
                    {copy.sequences.stop}
                  </>
                ) : (
                  <>
                    <Play size={16} strokeWidth={2.4} />
                    {copy.sequences.start}
                  </>
                )}
              </button>
            </article>
          );
        })}
      </div>
    </div>
  );
}
