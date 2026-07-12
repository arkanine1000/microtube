import {
  Bookmark,
  Pause,
  Play,
  Timer,
  Volume2,
  VolumeX,
} from 'lucide-react';
import { useRef, useState } from 'react';
import type { LocalPreset } from '../audio/localPresets';
import { EEG_BANDS, VOLUME, clamp, eegBandIndex } from '../audio/params';
import {
  TIMER_MAX_MINUTES,
  TIMER_MIN_MINUTES,
  TIMER_STEP_MINUTES,
  type MicroTube,
} from '../audio/useMicroTube';
import { useLocale } from '../i18n/LocaleProvider';
import { Popover } from './Popover';
import { PresetSheet } from './PresetSheet';
import { SlimSlider } from './SlimSlider';

type HeaderPop = 'volume' | 'timer' | null;

function mmss(secs: number): string {
  const total = Math.max(0, Math.ceil(secs));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const mm = h > 0 ? String(m).padStart(2, '0') : String(m);
  return `${h > 0 ? `${h}:` : ''}${mm}:${String(s).padStart(2, '0')}`;
}

/**
 * The slim fixed header: the single play/pause control, brand + live band
 * readout, and quick access to the three "set rarely" controls — master
 * volume, the auto-stop timer, and saved presets. When auto-stop is armed a
 * hairline progress bar counts the session down along the lower edge.
 */
export function HeaderBar({
  mt,
  presets,
  onPresetsChange,
}: {
  mt: MicroTube;
  presets: LocalPreset[];
  onPresetsChange: (presets: LocalPreset[]) => void;
}) {
  const { copy } = useLocale();
  const [pop, setPop] = useState<HeaderPop>(null);
  const [sheetOpen, setSheetOpen] = useState(false);
  const volWrapRef = useRef<HTMLDivElement>(null);
  const timerWrapRef = useRef<HTMLDivElement>(null);
  const volBtnRef = useRef<HTMLButtonElement>(null);
  const timerBtnRef = useRef<HTMLButtonElement>(null);

  const { state, timer } = mt;
  const band = EEG_BANDS[eegBandIndex(state.beatFreq)];

  const showTimer = timer.enabled && !timer.fired;
  const totalSecs = timer.minutes * 60;
  const remaining = timer.remainingSecs ?? totalSecs;
  const frac =
    showTimer && totalSecs > 0 ? clamp(remaining / totalSecs, 0, 1) : 0;

  const toggle = (which: Exclude<HeaderPop, null>) =>
    setPop((p) => (p === which ? null : which));

  const closePop = (which: Exclude<HeaderPop, null>) => {
    setPop(null);
    (which === 'volume' ? volBtnRef : timerBtnRef).current?.focus();
  };

  const timerLabel = timer.fired
    ? copy.timer.stopped
    : timer.remainingSecs === null
      ? copy.timer.off
      : mmss(timer.remainingSecs);

  const minutesSpec = {
    icon: Timer,
    min: TIMER_MIN_MINUTES,
    max: TIMER_MAX_MINUTES,
    step: TIMER_STEP_MINUTES,
    coarse: TIMER_STEP_MINUTES * 3,
    format: (v: number) => `${v} ${copy.timer.minutesAbbrev}`,
  };

  return (
    <header className="header">
      <button
        className="header-play"
        type="button"
        onClick={mt.togglePlaying}
        onContextMenu={(e) => e.preventDefault()}
        data-playing={state.playing}
        aria-label={state.playing ? copy.header.pause : copy.header.play}
      >
        {state.playing ? (
          <Pause size={20} strokeWidth={2.4} fill="currentColor" />
        ) : (
          <Play size={20} strokeWidth={2.4} fill="currentColor" />
        )}
      </button>

      <button
        className="header-id"
        type="button"
        onClick={mt.returnToStart}
        aria-label={copy.header.backToStart}
      >
        <div className="header-brand">
          micro<span>tube</span>
        </div>
        <div className="header-status" data-playing={state.playing}>
          <span className="header-band" style={{ color: band.color }}>
            {band.greek} {state.beatFreq.toFixed(1)} Hz
          </span>
          {' · '}
          {state.playing ? copy.header.signalActive : copy.header.signalPaused}
        </div>
      </button>

      <div className="header-actions">
        <div className="header-pop-wrap" ref={volWrapRef}>
          <button
            ref={volBtnRef}
            className={`header-icon-btn${pop === 'volume' ? ' open' : ''}`}
            type="button"
            onClick={() => toggle('volume')}
            onContextMenu={(e) => e.preventDefault()}
            aria-label={copy.header.volume}
            aria-haspopup="dialog"
            aria-expanded={pop === 'volume'}
          >
            {state.volume === 0 ? (
              <VolumeX size={18} strokeWidth={2.2} />
            ) : (
              <Volume2 size={18} strokeWidth={2.2} />
            )}
          </button>
          {pop === 'volume' && (
            <Popover
              label={copy.header.volume}
              wrapRef={volWrapRef}
              onClose={() => closePop('volume')}
            >
              <SlimSlider
                spec={VOLUME}
                label={copy.sliders.volume.label}
                hint={copy.sliders.volume.hint}
                value={state.volume}
                onChange={(v) => mt.setParam('volume', v)}
              />
            </Popover>
          )}
        </div>

        <div className="header-pop-wrap" ref={timerWrapRef}>
          <button
            ref={timerBtnRef}
            className={`header-icon-btn${pop === 'timer' ? ' open' : ''}${
              showTimer ? ' armed' : ''
            }`}
            type="button"
            onClick={() => toggle('timer')}
            onContextMenu={(e) => e.preventDefault()}
            aria-label={copy.header.timer}
            aria-haspopup="dialog"
            aria-expanded={pop === 'timer'}
          >
            {showTimer && timer.remainingSecs !== null ? (
              <span className="header-countdown">
                {mmss(timer.remainingSecs)}
              </span>
            ) : (
              <Timer size={18} strokeWidth={2.2} />
            )}
          </button>
          {pop === 'timer' && (
            <Popover
              label={copy.header.timer}
              wrapRef={timerWrapRef}
              onClose={() => closePop('timer')}
            >
              <div className="timer-block">
                <div className="timer-controls">
                  <label className="timer-toggle">
                    <input
                      type="checkbox"
                      checked={timer.enabled}
                      onChange={(e) =>
                        mt.setTimerEnabled(e.currentTarget.checked)
                      }
                    />
                    <span>{copy.timer.autoStop}</span>
                  </label>
                  <span
                    className={`timer-readout${timer.fired ? ' fired' : ''}`}
                  >
                    {timerLabel}
                  </span>
                </div>
                <SlimSlider
                  spec={minutesSpec}
                  label={copy.timer.minutes}
                  value={timer.minutes}
                  onChange={mt.setTimerMinutes}
                />
              </div>
            </Popover>
          )}
        </div>

        <button
          className="header-icon-btn"
          type="button"
          onClick={() => setSheetOpen(true)}
          onContextMenu={(e) => e.preventDefault()}
          aria-label={copy.header.presets}
          aria-haspopup="dialog"
        >
          <Bookmark size={18} strokeWidth={2.2} />
        </button>
      </div>

      {showTimer && (
        <div
          className="header-progress"
          role="progressbar"
          aria-label={copy.header.timeRemaining}
          aria-valuemin={0}
          aria-valuemax={totalSecs}
          aria-valuenow={Math.round(remaining)}
        >
          <div
            className="header-progress-fill"
            style={{ width: `${frac * 100}%` }}
          />
        </div>
      )}

      {sheetOpen && (
        <PresetSheet
          mt={mt}
          presets={presets}
          onPresetsChange={onPresetsChange}
          onClose={() => setSheetOpen(false)}
        />
      )}
    </header>
  );
}
