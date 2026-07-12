import { useEffect, useRef } from 'react';
import type { MicroTubeState } from '../audio/params';

const TAU = Math.PI * 2;

function hexToRgb(hex: string): [number, number, number] {
  const normalized = hex.replace('#', '');
  const value = Number.parseInt(normalized, 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

function rgba(hex: string, alpha: number): string {
  const [r, g, b] = hexToRgb(hex);
  return `rgba(${r}, ${g}, ${b}, ${alpha})`;
}

export function WaveformVisualizer({
  state,
  accent,
}: {
  state: MicroTubeState;
  accent: string;
}) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const stateRef = useRef(state);

  stateRef.current = state;

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let raf = 0;
    let width = 0;
    let height = 0;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const reducedMotion = window.matchMedia(
      '(prefers-reduced-motion: reduce)',
    ).matches;

    const resize = () => {
      const rect = canvas.getBoundingClientRect();
      width = Math.max(1, Math.floor(rect.width));
      height = Math.max(1, Math.floor(rect.height));
      canvas.width = Math.floor(width * dpr);
      canvas.height = Math.floor(height * dpr);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };

    const drawLine = (
      time: number,
      offset: number,
      alpha: number,
      lineWidth: number,
    ) => {
      const s = stateRef.current;
      const phase = reducedMotion || !s.playing ? 0 : time * 0.0017;
      const beatCycles = Math.min(12, Math.max(2.4, s.beatFreq / 3.5));
      const warmth = s.harmonics;
      const texture = s.noiseLevel;
      const emergence = s.emergence;
      const drift = s.shepard;
      const gain = (s.playing ? 0.55 : 0.2) * (0.45 + s.volume * 0.85);
      const mid = height * (0.5 + offset);
      const scale = height * 0.26 * gain;

      ctx.beginPath();
      for (let px = 0; px <= width; px += 2) {
        const x = px / width;
        const carrier = Math.sin(TAU * (x * beatCycles + phase));
        const harmonic =
          warmth *
          (0.48 * Math.sin(TAU * (x * beatCycles * 2 + phase * 1.35)) +
            0.22 * Math.sin(TAU * (x * beatCycles * 3 - phase * 0.7)));
        const mist =
          texture *
          0.28 *
          Math.sin(TAU * (x * (beatCycles + 5.7) - phase * 0.45));
        const voices =
          emergence *
          0.34 *
          Math.sin(TAU * (x * (beatCycles * 0.5 + 1.8) + phase * 0.42));
        const glide = drift * 0.25 * Math.sin(TAU * (x * 1.1 - phase * 0.18));
        const y = mid + (carrier * 0.72 + harmonic + mist + voices + glide) * scale;

        if (px === 0) {
          ctx.moveTo(px, y);
        } else {
          ctx.lineTo(px, y);
        }
      }
      ctx.strokeStyle = rgba(accent, alpha);
      ctx.lineWidth = lineWidth;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';
      ctx.stroke();
    };

    const draw = (time: number) => {
      if (width === 0 || height === 0) resize();
      ctx.clearRect(0, 0, width, height);

      const gradient = ctx.createLinearGradient(0, 0, width, 0);
      gradient.addColorStop(0, 'rgba(255, 255, 255, 0)');
      gradient.addColorStop(0.5, rgba(accent, 0.18));
      gradient.addColorStop(1, 'rgba(255, 255, 255, 0)');
      ctx.strokeStyle = gradient;
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.moveTo(0, height / 2);
      ctx.lineTo(width, height / 2);
      ctx.stroke();

      drawLine(time, 0.04, 0.24, 5);
      drawLine(time + 140, 0, 0.82, 1.7);

      if (!reducedMotion) {
        raf = window.requestAnimationFrame(draw);
      }
    };

    const observer = new ResizeObserver(resize);
    observer.observe(canvas);
    resize();
    raf = window.requestAnimationFrame(draw);

    return () => {
      observer.disconnect();
      window.cancelAnimationFrame(raf);
    };
  }, [accent]);

  return (
    <div className="waveform-strip" aria-hidden="true">
      <canvas ref={canvasRef} />
    </div>
  );
}
