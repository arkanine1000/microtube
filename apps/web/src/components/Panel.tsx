import { ChevronDown, type LucideIcon } from 'lucide-react';
import { useState, type MouseEvent, type ReactNode } from 'react';

/**
 * Panel expansion state, keyed by panel id, kept in module scope so it
 * survives tab switches — leaving a tab unmounts its panels, and this map
 * carries each panel's open/closed state back when the tab remounts. It is
 * deliberately not persisted, so a page reload starts every panel collapsed.
 */
const expansionState = new Map<string, boolean>();

/**
 * A collapsible studio panel. Every panel starts collapsed so a first-time
 * user sees a clean list of sections rather than a wall of controls — the
 * panel chrome is the toggle, and the chevron rotates to point the way.
 */
export function Panel({
  id,
  icon: Icon,
  title,
  caption,
  className,
  defaultOpen = false,
  children,
}: {
  id: string;
  icon: LucideIcon;
  title: string;
  caption?: string;
  className?: string;
  defaultOpen?: boolean;
  children: ReactNode;
}) {
  const [open, setOpen] = useState(() => expansionState.get(id) ?? defaultOpen);

  const toggle = () => {
    setOpen((o) => {
      const next = !o;
      expansionState.set(id, next);
      return next;
    });
  };

  const handlePanelClick = (event: MouseEvent<HTMLElement>) => {
    const target = event.target;
    const body = event.currentTarget.querySelector('.panel-body');

    if (target instanceof Node && body?.contains(target)) {
      return;
    }

    toggle();
  };

  return (
    <section
      className={`panel collapsible${open ? ' open' : ''}${
        className ? ` ${className}` : ''
      }`}
      onClick={handlePanelClick}
    >
      <button
        className="panel-title"
        type="button"
        onContextMenu={(e) => e.preventDefault()}
        aria-expanded={open}
      >
        <Icon size={13} strokeWidth={2.3} />
        <span>{title}</span>
        {caption && <small>{caption}</small>}
        <ChevronDown className="panel-chevron" size={16} strokeWidth={2.4} />
      </button>
      {open && (
        <div className="panel-body">
          <div className="panel-body-inner">{children}</div>
        </div>
      )}
    </section>
  );
}
