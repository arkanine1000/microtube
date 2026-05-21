import { ChevronLeft, ChevronRight } from 'lucide-react';
import type React from 'react';
import { useRef } from 'react';
import { clamp, type SliderSpec } from '../audio/params';

/**
 * A touch-friendly parameter slider. Dragging the track gives fine control
 * (snapped to `spec.step`); the ◂ ▸ buttons nudge by the coarse step.
 */
export function ParameterSlider({
  spec,
  value,
  onChange,
}: {
  spec: SliderSpec;
  value: number;
  onChange: (v: number) => void;
}) {
  const trackRef = useRef<HTMLDivElement>(null);

  const snap = (v: number, step: number) => {
    const snapped = Math.round((v - spec.min) / step) * step + spec.min;
    return clamp(Number(snapped.toFixed(6)), spec.min, spec.max);
  };

  const fromClientX = (clientX: number) => {
    const el = trackRef.current;
    if (!el) return value;
    const rect = el.getBoundingClientRect();
    const t = clamp((clientX - rect.left) / rect.width, 0, 1);
    return snap(spec.min + t * (spec.max - spec.min), spec.step);
  };

  const onPointerDown = (e: React.PointerEvent) => {
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    onChange(fromClientX(e.clientX));
  };

  const onPointerMove = (e: React.PointerEvent) => {
    if (e.buttons === 0) return;
    onChange(fromClientX(e.clientX));
  };

  const nudge = (dir: number) => onChange(snap(value + dir * spec.coarse, spec.step));

  const fillPct = clamp(
    ((value - spec.min) / (spec.max - spec.min)) * 100,
    0,
    100,
  );

  return (
    <div className="slider">
      <div className="slider-head">
        <div>
          <span className="slider-label">{spec.label}</span>{' '}
          <span className="slider-hint">{spec.hint}</span>
        </div>
        <span className="slider-value">{spec.format(value)}</span>
      </div>
      <div className="slider-row">
        <button
          className="nudge"
          type="button"
          onClick={() => nudge(-1)}
          aria-label={`decrease ${spec.label}`}
        >
          <ChevronLeft size={18} strokeWidth={2.4} />
        </button>
        <div
          className="track"
          ref={trackRef}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          role="slider"
          aria-label={spec.label}
          aria-valuemin={spec.min}
          aria-valuemax={spec.max}
          aria-valuenow={value}
        >
          <div className="track-fill" style={{ width: `${fillPct}%` }} />
          <div className="track-knob" style={{ left: `${fillPct}%` }} />
        </div>
        <button
          className="nudge"
          type="button"
          onClick={() => nudge(1)}
          aria-label={`increase ${spec.label}`}
        >
          <ChevronRight size={18} strokeWidth={2.4} />
        </button>
      </div>
    </div>
  );
}
