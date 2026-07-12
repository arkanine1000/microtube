/**
 * A segmented control. Option index is the value — which lines up with the
 * Wasm enum discriminants (Timbre, MistType, SpawnMode, Direction).
 *
 * When `enabled` is supplied, the option buttons are also the on/off surface:
 * pressing an inactive option engages the feature, and pressing the active
 * option again can disable it through `onDisable`.
 */
export function Segmented({
  caption,
  options,
  value,
  onChange,
  enabled,
  statusLabels,
  onDisable,
  className,
}: {
  caption: string;
  options: readonly string[];
  value: number;
  onChange: (v: number) => void;
  enabled?: boolean;
  statusLabels?: {
    on: string;
    off: string;
  };
  onDisable?: () => void;
  className?: string;
}) {
  return (
    <div
      className={`toggle-block${enabled === false ? ' off' : ''}${
        className ? ` ${className}` : ''
      }`}
    >
      <div className="toggle-caption">
        <span className="toggle-name">{caption}</span>
        {enabled !== undefined && statusLabels && (
          <span className={`toggle-status${enabled ? ' on' : ''}`}>
            {enabled ? statusLabels.on : statusLabels.off}
          </span>
        )}
      </div>
      <div className="segmented">
        {options.map((option, i) => {
          const selected = i === value;
          const active = enabled === undefined || enabled;
          return (
            <button
              key={option}
              className={`seg${selected && active ? ' on' : ''}`}
              onClick={() => {
                if (selected && enabled && onDisable) {
                  onDisable();
                  return;
                }
                onChange(i);
              }}
              onContextMenu={(e) => e.preventDefault()}
              type="button"
              aria-pressed={selected && active}
            >
              {option}
            </button>
          );
        })}
      </div>
    </div>
  );
}
