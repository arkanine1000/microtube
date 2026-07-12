import { useEffect, useRef, type ReactNode, type RefObject } from 'react';

/**
 * A small anchored popover for header controls. Light-dismiss: pointerdown
 * outside the anchor wrap (trigger + popover) or Escape closes it. Focus
 * moves to the first control on open; the caller restores it on close.
 */
export function Popover({
  label,
  wrapRef,
  onClose,
  children,
}: {
  label: string;
  /** The container holding both the trigger and this popover. */
  wrapRef: RefObject<HTMLElement | null>;
  onClose: () => void;
  children: ReactNode;
}) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    const onPointerDown = (e: PointerEvent) => {
      const wrap = wrapRef.current;
      if (wrap && !wrap.contains(e.target as Node)) onClose();
    };
    window.addEventListener('keydown', onKeyDown);
    window.addEventListener('pointerdown', onPointerDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      window.removeEventListener('pointerdown', onPointerDown);
    };
  }, [onClose, wrapRef]);

  useEffect(() => {
    ref.current
      ?.querySelector<HTMLElement>('input, button, [tabindex]')
      ?.focus();
  }, []);

  return (
    <div className="popover" role="dialog" aria-label={label} ref={ref}>
      {children}
    </div>
  );
}
