import { Save, Trash2 } from 'lucide-react';
import { useEffect, useMemo, useState, type FormEvent } from 'react';
import {
  normalizePresetName,
  nextPresetName,
  persistLocalPresets,
  removeLocalPreset,
  snapshotFromState,
  upsertLocalPreset,
  type LocalPreset,
} from '../audio/localPresets';
import {
  EEG_BANDS,
  eegBandIndex,
  type PresetSnapshot,
} from '../audio/params';
import type { MicroTube } from '../audio/useMicroTube';
import { useLocale } from '../i18n/LocaleProvider';
import type { Copy } from '../i18n/copy';
import { Modal } from './Modal';

type DialogState =
  | { type: 'closed' }
  | { type: 'save' }
  | { type: 'delete'; preset: LocalPreset };

const PAGE_SIZE = 3;

export function LocalPresetsPanel({
  mt,
  presets,
  onPresetsChange,
}: {
  mt: MicroTube;
  presets: LocalPreset[];
  onPresetsChange: (presets: LocalPreset[]) => void;
}) {
  const { copy } = useLocale();
  const [dialog, setDialog] = useState<DialogState>({ type: 'closed' });
  const [presetName, setPresetName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [visibleCount, setVisibleCount] = useState(PAGE_SIZE);
  const accent = EEG_BANDS[eegBandIndex(mt.state.beatFreq)].color;

  const sortedPresets = useMemo(
    () => [...presets].sort((a, b) => b.updatedAt - a.updatedAt),
    [presets],
  );
  const visiblePresets = sortedPresets.slice(0, visibleCount);

  useEffect(() => {
    setVisibleCount((count) =>
      Math.min(Math.max(count, PAGE_SIZE), Math.max(sortedPresets.length, PAGE_SIZE)),
    );
  }, [sortedPresets.length]);

  const closeDialog = () => {
    setDialog({ type: 'closed' });
    setError(null);
  };

  const openSaveDialog = () => {
    setPresetName(nextPresetName(presets, copy.localPresets.defaultName));
    setError(null);
    setDialog({ type: 'save' });
  };

  const savePreset = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const trimmed = normalizePresetName(presetName);
    if (!trimmed) {
      setError(copy.localPresets.nameRequired);
      return;
    }

    const next = upsertLocalPreset(
      presets,
      trimmed,
      snapshotFromState(mt.state),
    );
    if (!persistLocalPresets(next)) {
      setError(copy.localPresets.storageError);
      return;
    }

    onPresetsChange(next);
    closeDialog();
  };

  const deletePreset = (preset: LocalPreset) => {
    const next = removeLocalPreset(presets, preset.id);
    if (!persistLocalPresets(next)) {
      setError(copy.localPresets.storageError);
      return;
    }

    onPresetsChange(next);
    closeDialog();
  };

  return (
    <div className="local-presets">
      <button
        className="btn btn-primary local-preset-save"
        type="button"
        onClick={openSaveDialog}
      >
        <Save size={16} strokeWidth={2.4} />
        {copy.localPresets.saveCurrent}
      </button>

      {sortedPresets.length === 0 ? (
        <p className="local-preset-empty">{copy.localPresets.empty}</p>
      ) : (
        <div className="local-preset-list" role="list">
          {visiblePresets.map((preset) => (
            <div className="local-preset-row" role="listitem" key={preset.id}>
              <button
                className="local-preset-load"
                type="button"
                onClick={() => mt.applySnapshot(preset.snapshot)}
                aria-label={`${copy.localPresets.loadPreset} ${preset.name}`}
              >
                <span className="local-preset-name">{preset.name}</span>
                <span className="local-preset-summary">
                  {formatPresetSummary(preset.snapshot, copy)}
                </span>
              </button>
              <button
                className="local-preset-delete"
                type="button"
                onClick={() => {
                  setError(null);
                  setDialog({ type: 'delete', preset });
                }}
                aria-label={`${copy.localPresets.deletePreset} ${preset.name}`}
                title={`${copy.localPresets.deletePreset} ${preset.name}`}
              >
                <Trash2 size={16} strokeWidth={2.35} />
              </button>
            </div>
          ))}
          {visibleCount < sortedPresets.length && (
            <button
              className="btn local-preset-more"
              type="button"
              onClick={() => setVisibleCount((count) => count + PAGE_SIZE)}
            >
              {copy.localPresets.showMore}
            </button>
          )}
        </div>
      )}

      {dialog.type === 'save' && (
        <Modal
          title={copy.localPresets.saveTitle}
          closeLabel={copy.localPresets.close}
          accent={accent}
          onClose={closeDialog}
        >
          <form className="preset-form" onSubmit={savePreset}>
            <label className="preset-field">
              <span>{copy.localPresets.nameLabel}</span>
              <input
                data-autofocus
                className="preset-name-input"
                type="text"
                maxLength={48}
                value={presetName}
                onChange={(event) => {
                  setPresetName(event.currentTarget.value);
                  if (error) setError(null);
                }}
              />
            </label>
            {error && (
              <p className="modal-error" role="alert">
                {error}
              </p>
            )}
            <div className="modal-actions">
              <button className="btn" type="button" onClick={closeDialog}>
                {copy.localPresets.cancel}
              </button>
              <button className="btn btn-primary" type="submit">
                {copy.localPresets.saveAction}
              </button>
            </div>
          </form>
        </Modal>
      )}

      {dialog.type === 'delete' && (
        <Modal
          title={copy.localPresets.deleteTitle}
          closeLabel={copy.localPresets.close}
          accent={accent}
          onClose={closeDialog}
        >
          <div className="preset-delete-confirm">
            <p className="modal-copy">
              {copy.localPresets.deletePromptPrefix}
              <strong>{dialog.preset.name}</strong>
              {copy.localPresets.deletePromptSuffix}
            </p>
            {error && (
              <p className="modal-error" role="alert">
                {error}
              </p>
            )}
            <div className="modal-actions">
              <button className="btn" type="button" onClick={closeDialog}>
                {copy.localPresets.cancel}
              </button>
              <button
                className="btn btn-danger"
                type="button"
                data-autofocus
                onClick={() => deletePreset(dialog.preset)}
              >
                {copy.localPresets.deleteAction}
              </button>
            </div>
          </div>
        </Modal>
      )}
    </div>
  );
}

function formatPresetSummary(snapshot: PresetSnapshot, copy: Copy): string {
  const parts = [
    `${snapshot.beatFreq.toFixed(1)} Hz ${copy.localPresets.beatLabel}`,
    `${snapshot.baseFreq.toFixed(0)} Hz ${copy.localPresets.baseLabel}`,
    copy.modes.timbres[snapshot.timbre],
  ];

  if (snapshot.noiseLevel > 0.01) {
    parts.push(
      `${copy.modes.mists[snapshot.mistType]} ${copy.localPresets.mistLabel}`,
    );
  }
  if (snapshot.emergence > 0.01) {
    parts.push(copy.modes.spawnModes[snapshot.spawnMode]);
  }
  if (snapshot.shepard > 0.01) {
    parts.push(
      `${copy.modes.directions[snapshot.shepardDirection]} ${copy.localPresets.driftLabel}`,
    );
  }

  return parts.join(' / ');
}
