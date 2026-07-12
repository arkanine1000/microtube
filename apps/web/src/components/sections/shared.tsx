import type { LucideIcon } from 'lucide-react';
import type { ReactNode } from 'react';
import type { SliderKey } from '../../audio/params';

/** App's copy-wired SlimSlider factory, threaded into every section. */
export type SliderRender = (key: SliderKey) => ReactNode;

/** The slim non-interactive title row at the top of a deck section. */
export function SectionHead({
  icon: Icon,
  title,
}: {
  icon: LucideIcon;
  title: string;
}) {
  return (
    <div className="section-head">
      <Icon size={13} strokeWidth={2.3} />
      <span>{title}</span>
    </div>
  );
}
