import { EEG_BANDS, eegBandIndex } from '../audio/params';

function formatUptime(secs: number): string {
  const total = Math.floor(secs);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = String(m).padStart(2, '0');
  const ss = String(s).padStart(2, '0');
  return h > 0 ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
}

/**
 * The strip dashboard — the EEG band ladder plus session uptime, sitting
 * prominently at the top of the app.
 */
export function StripDashboard({
  beatFreq,
  uptimeSecs,
}: {
  beatFreq: number;
  uptimeSecs: number;
}) {
  const active = eegBandIndex(beatFreq);

  return (
    <header className="panel strip">
      <div className="strip-top">
        <div className="brand">
          micro<span>tube</span>
        </div>
        <div className="uptime">
          <span className="uptime-label">session</span>
          {formatUptime(uptimeSecs)}
        </div>
      </div>
      <div className="bands">
        {EEG_BANDS.map((band, i) => (
          <div
            key={band.name}
            className={`band${i === active ? ' active' : ''}`}
            style={{ color: band.color }}
          >
            <div className="band-greek">{band.greek}</div>
            <div className="band-name">{band.name}</div>
            <div className="band-blurb">{band.blurb}</div>
          </div>
        ))}
      </div>
    </header>
  );
}
