import type { LucideIcon } from 'lucide-react';
import type React from 'react';
import { useRef, useState } from 'react';
import { clamp } from '../audio/params';
import { useLocale } from '../i18n/LocaleProvider';

/**
 * The shape SlimSlider needs from a parameter spec. `SliderSpec` from
 * params.ts satisfies this structurally; ad-hoc specs (e.g. the auto-stop
 * timer minutes) can supply just these fields.
 */
export interface SlimSliderSpec {
  icon: LucideIcon;
  min: number;
  max: number;
  step: number;
  /** Coarse step for PageUp/PageDown and Shift+Arrow. */
  coarse: number;
  format: (v: number) => string;
  /** Unit suffix shown in the numeric-entry form. */
  unit?: string;
  /**
   * True for a function that is genuinely on/off (mist, drift, emergence) —
   * its minimum is 0, and the UI recedes it while it sits there.
   */
  toggle?: boolean;
  /**
   * Multiplier between the stored value and the number a human types —
   * 100 for 0..1 parameters displayed as percentages.
   */
  displayScale?: number;
}

/** Vertical distance (px) a drag can wander before fine-adjust kicks in. */
const FINE_DEAD_ZONE = 24;
/** Sensitivity floor — pulling further away never goes below ×0.08. */
const FINE_FLOOR = 0.08;
/** Movement below this (px) still counts as a tap. */
const TAP_SLOP = 8;
/** Presses longer than this (ms) are drags even without movement. */
const TAP_MAX_MS = 300;

interface DragState {
  startX: number;
  startY: number;
  lastX: number;
  /** Unsnapped accumulator so changing fine gain mid-drag never jumps. */
  raw: number;
  emitted: number;
  moved: boolean;
  t0: number;
}

/**
 * A slim single-row parameter slider. The whole row is the track: drag
 * anywhere for coarse control, pull the pointer away vertically while
 * dragging for progressively finer sensitivity, tap to jump. The value chip
 * on the right opens exact numeric entry.
 */
export function SlimSlider({
  spec,
  label,
  hint,
  value,
  onChange,
}: {
  spec: SlimSliderSpec;
  label: string;
  hint?: string;
  value: number;
  onChange: (v: number) => void;
}) {
  const trackRef = useRef<HTMLDivElement>(null);
  const drag = useRef<DragState | null>(null);
  const { copy } = useLocale();
  const [fineGain, setFineGain] = useState(1);
  const [editing, setEditing] = useState(false);
  const [text, setText] = useState('');
  const [invalid, setInvalid] = useState(false);

  const scale = spec.displayScale ?? 1;
  const editUnit = spec.displayScale === 100 ? '%' : (spec.unit ?? '');

  const snap = (v: number) => {
    const snapped = Math.round((v - spec.min) / spec.step) * spec.step + spec.min;
    return clamp(Number(snapped.toFixed(6)), spec.min, spec.max);
  };

  const onPointerDown = (e: React.PointerEvent) => {
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    drag.current = {
      startX: e.clientX,
      startY: e.clientY,
      lastX: e.clientX,
      raw: value,
      emitted: value,
      moved: false,
      t0: performance.now(),
    };
  };

  const onPointerMove = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d || e.buttons === 0) return;
    const dx = e.clientX - d.lastX;
    d.lastX = e.clientX;
    if (
      !d.moved &&
      Math.hypot(e.clientX - d.startX, e.clientY - d.startY) > TAP_SLOP
    ) {
      d.moved = true;
    }
    if (!d.moved) return;

    const dy = Math.abs(e.clientY - d.startY);
    const g = clamp(FINE_DEAD_ZONE / Math.max(dy, FINE_DEAD_ZONE), FINE_FLOOR, 1);
    setFineGain(g);

    const width = trackRef.current?.getBoundingClientRect().width ?? 1;
    d.raw = clamp(
      d.raw + (dx / width) * (spec.max - spec.min) * g,
      spec.min,
      spec.max,
    );
    const next = snap(d.raw);
    if (next !== d.emitted) {
      d.emitted = next;
      onChange(next);
    }
  };

  const onPointerEnd = (e: React.PointerEvent) => {
    const d = drag.current;
    drag.current = null;
    setFineGain(1);
    if (!d) return;
    if (
      e.type === 'pointerup' &&
      !d.moved &&
      performance.now() - d.t0 < TAP_MAX_MS
    ) {
      const rect = trackRef.current?.getBoundingClientRect();
      if (!rect) return;
      const t = clamp((e.clientX - rect.left) / rect.width, 0, 1);
      onChange(snap(spec.min + t * (spec.max - spec.min)));
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    let next: number;
    const fine = e.shiftKey ? spec.coarse : spec.step;
    switch (e.key) {
      case 'ArrowRight':
      case 'ArrowUp':
        next = value + fine;
        break;
      case 'ArrowLeft':
      case 'ArrowDown':
        next = value - fine;
        break;
      case 'PageUp':
        next = value + spec.coarse;
        break;
      case 'PageDown':
        next = value - spec.coarse;
        break;
      case 'Home':
        next = spec.min;
        break;
      case 'End':
        next = spec.max;
        break;
      case 'Enter':
        startEdit();
        e.preventDefault();
        return;
      default:
        return;
    }
    e.preventDefault();
    onChange(snap(clamp(next, spec.min, spec.max)));
  };

  const startEdit = () => {
    const stepDisp = spec.step * scale;
    const decimals = stepDisp >= 1 ? 0 : Math.min(3, -Math.floor(Math.log10(stepDisp)));
    setText(String(Number((value * scale).toFixed(decimals))));
    setInvalid(false);
    setEditing(true);
  };

  const closeEdit = () => {
    setEditing(false);
    setInvalid(false);
    // Hand focus back to the track so keyboard users keep their place.
    requestAnimationFrame(() => trackRef.current?.focus());
  };

  const commitEdit = (e: React.FormEvent) => {
    e.preventDefault();
    const parsed = Number(text.trim().replace(',', '.'));
    if (!Number.isFinite(parsed)) {
      setInvalid(true);
      return;
    }
    onChange(snap(clamp(parsed / scale, spec.min, spec.max)));
    closeEdit();
  };

  const onFormBlur = (e: React.FocusEvent<HTMLFormElement>) => {
    // Dismissing the keyboard / tapping elsewhere cancels rather than commits.
    if (!e.currentTarget.contains(e.relatedTarget as Node | null)) {
      closeEdit();
    }
  };

  const fillPct = clamp(
    ((value - spec.min) / (spec.max - spec.min)) * 100,
    0,
    100,
  );
  const active = value > spec.min;
  const off = spec.toggle === true && !active;
  const Icon = spec.icon;
  const fineBadge = fineGain > 0.35 ? '×½' : fineGain > 0.15 ? '×¼' : '×⅒';

  if (editing) {
    return (
      <div className={`sslider editing${off ? ' off' : ''}`}>
        <form
          className={`sslider-edit${invalid ? ' invalid' : ''}`}
          onSubmit={commitEdit}
          onBlur={onFormBlur}
        >
          <span className="sslider-edit-label">
            <Icon size={15} strokeWidth={2.1} />
            {label}
          </span>
          <input
            className="sslider-edit-input"
            type="text"
            inputMode="decimal"
            value={text}
            onChange={(e) => {
              setText(e.currentTarget.value);
              setInvalid(false);
            }}
            onKeyDown={(e) => {
              if (e.key === 'Escape') closeEdit();
            }}
            aria-label={label}
            aria-invalid={invalid || undefined}
            // eslint-disable-next-line jsx-a11y/no-autofocus
            autoFocus
            onFocus={(e) => e.currentTarget.select()}
          />
          {editUnit && <span className="sslider-edit-unit">{editUnit}</span>}
          <button className="sslider-edit-ok" type="submit">
            {copy.slider.apply}
          </button>
          <button
            className="sslider-edit-cancel"
            type="button"
            onClick={closeEdit}
          >
            {copy.slider.cancel}
          </button>
        </form>
      </div>
    );
  }

  return (
    <div className={`sslider${off ? ' off' : ''}`}>
      <div
        className="sslider-track"
        ref={trackRef}
        role="slider"
        tabIndex={0}
        aria-label={label}
        aria-valuemin={spec.min}
        aria-valuemax={spec.max}
        aria-valuenow={value}
        aria-valuetext={spec.format(value)}
        title={hint}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerEnd}
        onPointerCancel={onPointerEnd}
        onKeyDown={onKeyDown}
        onContextMenu={(e) => e.preventDefault()}
      >
        <div className="sslider-fill" style={{ width: `${fillPct}%` }} />
        <span className="sslider-head">
          <span className={`sslider-icon${active ? ' active' : ''}`}>
            <Icon size={15} strokeWidth={2.1} />
          </span>
          <span className="sslider-label">{label}</span>
        </span>
        {fineGain < 1 && (
          <span className="sslider-fine" aria-hidden="true">
            {fineBadge} {copy.slider.fine}
          </span>
        )}
      </div>
      <button
        className="sslider-value"
        type="button"
        onClick={startEdit}
        onContextMenu={(e) => e.preventDefault()}
        aria-label={`${label} · ${copy.slider.editValue}`}
      >
        {spec.format(value)}
      </button>
    </div>
  );
}
