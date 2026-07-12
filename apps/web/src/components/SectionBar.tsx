import { useEffect, useRef } from 'react';
import type { MicroTubeState } from '../audio/params';
import { useLocale } from '../i18n/LocaleProvider';
import { SECTIONS, type SectionId } from '../sections';

const LONG_PRESS_MS = 520;
const QUICK_TOGGLE_SECTIONS = new Set<SectionId>(['mist', 'emergence', 'drift']);

/**
 * The bottom dock — section switching in the thumb zone. Each tab carries a
 * status dot for its on/off-able function so the whole engine state stays
 * glanceable no matter which section is open. Hidden on desktop, where all
 * sections render at once.
 */
export function SectionBar({
  active,
  onSelect,
  onQuickToggle,
  state,
  sequenceActive,
}: {
  active: SectionId;
  onSelect: (id: SectionId) => void;
  onQuickToggle: (id: SectionId) => void;
  state: MicroTubeState;
  sequenceActive: boolean;
}) {
  const { copy } = useLocale();
  const pressTimer = useRef<number | null>(null);
  const longPressed = useRef(false);

  const clearPressTimer = () => {
    if (pressTimer.current !== null) {
      window.clearTimeout(pressTimer.current);
      pressTimer.current = null;
    }
  };

  const startQuickToggle = (id: SectionId) => {
    clearPressTimer();
    longPressed.current = false;
    if (!QUICK_TOGGLE_SECTIONS.has(id)) {
      return;
    }
    pressTimer.current = window.setTimeout(() => {
      longPressed.current = true;
      onQuickToggle(id);
      pressTimer.current = null;
    }, LONG_PRESS_MS);
  };

  useEffect(() => clearPressTimer, []);

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
            onPointerDown={() => startQuickToggle(id)}
            onPointerUp={clearPressTimer}
            onPointerCancel={clearPressTimer}
            onPointerLeave={clearPressTimer}
            onClick={(e) => {
              if (longPressed.current) {
                e.preventDefault();
                longPressed.current = false;
                return;
              }
              onSelect(id);
            }}
            onContextMenu={(e) => {
              e.preventDefault();
              clearPressTimer();
            }}
            aria-current={active === id ? 'true' : undefined}
            aria-label={label}
          >
            <span className="dock-icon">
              <Icon size={20} strokeWidth={2.2} />
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
