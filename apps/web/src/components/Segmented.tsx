/**
 * A segmented control. Option index is the value — which lines up with the
 * Wasm enum discriminants (Timbre, MistType, SpawnMode, Direction).
 */
export function Segmented({
  caption,
  options,
  value,
  onChange,
}: {
  caption: string;
  options: readonly string[];
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <div className="toggle-block">
      <div className="toggle-caption">{caption}</div>
      <div className="segmented">
        {options.map((option, i) => (
          <button
            key={option}
            className={`seg${i === value ? ' on' : ''}`}
            onClick={() => onChange(i)}
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
