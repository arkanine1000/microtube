import { ChevronDown, type LucideIcon } from 'lucide-react';
import { useState, type ReactNode } from 'react';

/**
 * A collapsible studio panel. Every panel starts collapsed so a first-time
 * user sees a clean list of sections rather than a wall of controls — the
 * title bar is the toggle, and the chevron rotates to point the way.
 */
export function Panel({
  icon: Icon,
  title,
  caption,
  className,
  defaultOpen = false,
  children,
}: {
  icon: LucideIcon;
  title: string;
  caption?: string;
  className?: string;
  defaultOpen?: boolean;
  children: ReactNode;
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <section
      className={`panel collapsible${open ? ' open' : ''}${
        className ? ` ${className}` : ''
      }`}
    >
      <button
        className="panel-title"
        type="button"
        onClick={() => setOpen((o) => !o)}
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
