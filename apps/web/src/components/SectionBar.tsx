import type { MicroTubeState } from '../audio/params';
import { useLocale } from '../i18n/LocaleProvider';
import { SECTIONS, type SectionId } from '../sections';

/**
 * The bottom dock — section switching in the thumb zone. Each tab carries a
 * status dot for its on/off-able function so the whole engine state stays
 * glanceable no matter which section is open. Hidden on desktop, where all
 * sections render at once.
 */
export function SectionBar({
  active,
  onSelect,
  state,
  sequenceActive,
}: {
  active: SectionId;
  onSelect: (id: SectionId) => void;
  state: MicroTubeState;
  sequenceActive: boolean;
}) {
  const { copy } = useLocale();

  return (
    <nav className="dock" aria-label={copy.studioSectionsLabel}>
      {SECTIONS.map(({ id, icon: Icon, isOn }) => {
        const on = isOn?.(state, sequenceActive) ?? false;
        const text = copy.sections[id];
        const label = isOn
          ? `${text.label} · ${on ? copy.modes.status.on : copy.modes.status.off}`
          : text.label;
        return (
          <button
            key={id}
            className={`dock-tab${active === id ? ' active' : ''}`}
            type="button"
            onClick={() => onSelect(id)}
            onContextMenu={(e) => e.preventDefault()}
            aria-current={active === id ? 'true' : undefined}
            aria-label={label}
          >
            <span className="dock-icon">
              <Icon size={18} strokeWidth={2.2} />
              {isOn && (
                <span
                  className={`dock-dot${on ? ' on' : ''}${
                    id === 'sequences' && on ? ' pulse' : ''
                  }`}
                  aria-hidden="true"
                />
              )}
            </span>
            <span className="dock-label">{text.short}</span>
          </button>
        );
      })}
    </nav>
  );
}
