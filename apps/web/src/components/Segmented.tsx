/**
 * A segmented control. Option index is the value — which lines up with the
 * Wasm enum discriminants (Timbre, MistType, SpawnMode, Direction).
 *
 * When `enabled` is supplied the control belongs to a coupled function (mist,
 * drift, emergence): the caption carries a live on/off pip, and picking any
 * option auto-engages the function via the parent's coupling logic.
 */
export function Segmented({
  caption,
  options,
  value,
  onChange,
  enabled,
}: {
  caption: string;
  options: readonly string[];
  value: number;
  onChange: (v: number) => void;
  enabled?: boolean;
}) {
  return (
    <div className={`toggle-block${enabled === false ? ' off' : ''}`}>
      <div className="toggle-caption">
        <span className="toggle-name">{caption}</span>
        {enabled !== undefined && (
          <span className={`toggle-status${enabled ? ' on' : ''}`}>
            {enabled ? 'on' : 'off'}
          </span>
        )}
      </div>
      <div className="segmented">
        {options.map((option, i) => (
          <button
            key={option}
            className={`seg${i === value ? ' on' : ''}`}
            onClick={() => onChange(i)}
            onContextMenu={(e) => e.preventDefault()}
            type="button"
            aria-pressed={i === value}
          >
            {option}
          </button>
        ))}
      </div>
    </div>
  );
}
