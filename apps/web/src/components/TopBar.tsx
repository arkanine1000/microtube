import { Pause, Play } from 'lucide-react';
import { EEG_BANDS, eegBandIndex } from '../audio/params';

function formatClock(secs: number): string {
  const total = Math.floor(secs);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = String(m).padStart(2, '0');
  const ss = String(s).padStart(2, '0');
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

/**
 * The sticky transport bar — pinned to the top of the studio on every tab.
 * Brand mark, the single play/pause control, a live band readout, and the
 * session clock. Play/pause is hoisted here so it is always one tap away.
 */
export function TopBar({
  playing,
  onToggle,
  uptimeSecs,
  beatFreq,
}: {
  playing: boolean;
  onToggle: () => void;
  uptimeSecs: number;
  beatFreq: number;
}) {
  const band = EEG_BANDS[eegBandIndex(beatFreq)];

  return (
    <header className="topbar">
      <button
        className="topbar-play"
        type="button"
        onClick={onToggle}
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
        <span className="topbar-clock">{formatClock(uptimeSecs)}</span>
      </div>
    </header>
  );
}
