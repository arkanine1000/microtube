import { Pause, Play } from 'lucide-react';
import { EEG_BANDS, clamp, eegBandIndex } from '../audio/params';
import type { TimerStatus } from '../audio/useMicroTube';

function mmss(secs: number): string {
  const total = Math.max(0, Math.floor(secs));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = h > 0 ? String(m).padStart(2, '0') : String(m);
  return `${h > 0 ? `${h}:` : ''}${mm}:${String(s).padStart(2, '0')}`;
}

/**
 * The sticky transport bar — pinned to the top of the studio on every tab.
 * Brand mark, the single play/pause control, a live band readout, and — when
 * auto-stop is armed — a minimal progress bar counting the session down.
 */
export function TopBar({
  playing,
  onToggle,
  beatFreq,
  timer,
}: {
  playing: boolean;
  onToggle: () => void;
  beatFreq: number;
  timer: TimerStatus;
}) {
  const band = EEG_BANDS[eegBandIndex(beatFreq)];

  const showTimer = timer.enabled && !timer.fired;
  const totalSecs = timer.minutes * 60;
  const remaining = timer.remainingSecs ?? totalSecs;
  const frac =
    showTimer && totalSecs > 0 ? clamp(remaining / totalSecs, 0, 1) : 0;

  return (
    <header className="topbar">
      <button
        className="topbar-play"
        type="button"
        onClick={onToggle}
        onContextMenu={(e) => e.preventDefault()}
        data-playing={playing}
        aria-label={playing ? 'pause' : 'play'}
      >
        {playing ? (
          <Pause size={20} strokeWidth={2.4} fill="currentColor" />
        ) : (
          <Play size={20} strokeWidth={2.4} fill="currentColor" />
        )}
      </button>

      <div className="topbar-id">
        <div className="topbar-brand">
          micro<span>tube</span>
        </div>
        <div className="topbar-status" data-playing={playing}>
          {playing ? 'signal active' : 'signal paused'}
        </div>
      </div>

      <div className="topbar-meta">
        <span className="topbar-band" style={{ color: band.color }}>
          <span className="topbar-band-greek">{band.greek}</span>
          {band.name} · {beatFreq.toFixed(1)} Hz
        </span>
        {showTimer && (
          <span className="topbar-timer">{mmss(remaining)} left</span>
        )}
      </div>

      {showTimer && (
        <div
          className="topbar-progress"
          role="progressbar"
          aria-label="session time remaining"
          aria-valuemin={0}
          aria-valuemax={totalSecs}
          aria-valuenow={Math.round(remaining)}
        >
          <div
            className="topbar-progress-fill"
            style={{ width: `${frac * 100}%` }}
          />
        </div>
      )}
    </header>
  );
}
