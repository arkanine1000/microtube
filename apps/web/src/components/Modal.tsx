import { X } from 'lucide-react';
import { useEffect, useId, useRef, type ReactNode } from 'react';
import { createPortal } from 'react-dom';

export function Modal({
  title,
  closeLabel,
  accent,
  onClose,
  children,
}: {
  title: string;
  closeLabel: string;
  accent?: string;
  onClose: () => void;
  children: ReactNode;
}) {
  const titleId = useId();
  const cardRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [onClose]);

  useEffect(() => {
    const focusTarget = cardRef.current?.querySelector<HTMLElement>(
      '[data-autofocus], input:not([disabled]), button:not([disabled])',
    );
    focusTarget?.focus();
  }, []);

  return createPortal(
    <div
      className="modal-layer"
      style={accent ? { ['--accent' as string]: accent } : undefined}
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) onClose();
      }}
    >
      <div
        ref={cardRef}
        className="modal-card"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
      >
        <div className="modal-head">
          <h2 id={titleId}>{title}</h2>
          <button
            className="modal-close"
            type="button"
            onClick={onClose}
            aria-label={closeLabel}
          >
            <X size={17} strokeWidth={2.5} />
          </button>
        </div>
        <div className="modal-content">{children}</div>
      </div>
    </div>,
    document.body,
  );
}
