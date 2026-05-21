import type {
  Direction,
  MicroTubeState,
  MistType,
  PresetSnapshot,
  SpawnMode,
  Timbre,
} from './params';

export const LOCAL_PRESETS_STORAGE_KEY = 'microtube.localPresets.v1';

const STORAGE_VERSION = 1;

export interface LocalPreset {
  id: string;
  name: string;
  createdAt: number;
  updatedAt: number;
  snapshot: PresetSnapshot;
}

interface StoredPresetFile {
  version: number;
  presets: LocalPreset[];
}

type UnknownRecord = Record<string, unknown>;

const TIMBRES = [0, 1, 2, 3] as const;
const MIST_TYPES = [0, 1, 2, 3, 4] as const;
const SPAWN_MODES = [0, 1] as const;
const DIRECTIONS = [0, 1] as const;

export function snapshotFromState(state: MicroTubeState): PresetSnapshot {
  const { playing: _playing, ...snapshot } = state;
  return { ...snapshot };
}

export function normalizePresetName(name: string): string {
  return name.trim();
}

export function nextPresetName(presets: LocalPreset[], baseName: string): string {
  const base = normalizePresetName(baseName) || 'Custom Preset';
  if (!presets.some((preset) => preset.name === base)) return base;

  for (let index = 2; ; index += 1) {
    const candidate = `${base} ${index}`;
    if (!presets.some((preset) => preset.name === candidate)) {
      return candidate;
    }
  }
}

export function upsertLocalPreset(
  presets: LocalPreset[],
  name: string,
  snapshot: PresetSnapshot,
): LocalPreset[] {
  const trimmed = normalizePresetName(name);
  if (!trimmed) return presets;

  const now = Date.now();
  const existing = presets.find((preset) => preset.name === trimmed);
  if (existing) {
    return presets.map((preset) =>
      preset.id === existing.id
        ? {
            ...preset,
            name: trimmed,
            updatedAt: now,
            snapshot,
          }
        : preset,
    );
  }

  return [
    ...presets,
    {
      id: createPresetId(),
      name: trimmed,
      createdAt: now,
      updatedAt: now,
      snapshot,
    },
  ];
}

export function removeLocalPreset(
  presets: LocalPreset[],
  id: string,
): LocalPreset[] {
  return presets.filter((preset) => preset.id !== id);
}

export function loadLocalPresets(): LocalPreset[] {
  if (typeof window === 'undefined') return [];

  try {
    const raw = window.localStorage.getItem(LOCAL_PRESETS_STORAGE_KEY);
    if (!raw) return [];

    const parsed = JSON.parse(raw);
    if (!isRecord(parsed) || parsed.version !== STORAGE_VERSION) return [];
    if (!Array.isArray(parsed.presets)) return [];

    return parsed.presets.flatMap((preset) => {
      const parsedPreset = parseLocalPreset(preset);
      return parsedPreset ? [parsedPreset] : [];
    });
  } catch {
    return [];
  }
}

export function persistLocalPresets(presets: LocalPreset[]): boolean {
  if (typeof window === 'undefined') return false;

  const file: StoredPresetFile = {
    version: STORAGE_VERSION,
    presets,
  };

  try {
    window.localStorage.setItem(
      LOCAL_PRESETS_STORAGE_KEY,
      JSON.stringify(file),
    );
    return true;
  } catch {
    return false;
  }
}

function createPresetId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID();
  }

  return `preset-${Date.now().toString(36)}-${Math.random()
    .toString(36)
    .slice(2, 9)}`;
}

function parseLocalPreset(value: unknown): LocalPreset | null {
  if (!isRecord(value)) return null;

  const id = stringField(value.id);
  const name = stringField(value.name);
  const createdAt = numberField(value.createdAt);
  const updatedAt = numberField(value.updatedAt);
  const snapshot = parseSnapshot(value.snapshot);

  if (!id || !name || createdAt === null || updatedAt === null || !snapshot) {
    return null;
  }

  const trimmedName = normalizePresetName(name);
  if (!trimmedName) return null;

  return {
    id,
    name: trimmedName,
    createdAt,
    updatedAt,
    snapshot,
  };
}

function parseSnapshot(value: unknown): PresetSnapshot | null {
  if (!isRecord(value)) return null;

  const baseFreq = numberField(value.baseFreq);
  const beatFreq = numberField(value.beatFreq);
  const volume = numberField(value.volume);
  const noiseLevel = numberField(value.noiseLevel);
  const mistType = enumField<MistType>(value.mistType, MIST_TYPES);
  const harmonics = numberField(value.harmonics);
  const emergence = numberField(value.emergence);
  const spawnMode = enumField<SpawnMode>(value.spawnMode, SPAWN_MODES);
  const shepard = numberField(value.shepard);
  const shepardBase = numberField(value.shepardBase);
  const shepardDirection = enumField<Direction>(
    value.shepardDirection,
    DIRECTIONS,
  );
  const timbre = enumField<Timbre>(value.timbre, TIMBRES);

  if (
    baseFreq === null ||
    beatFreq === null ||
    volume === null ||
    noiseLevel === null ||
    mistType === null ||
    harmonics === null ||
    emergence === null ||
    spawnMode === null ||
    shepard === null ||
    shepardBase === null ||
    shepardDirection === null ||
    timbre === null
  ) {
    return null;
  }

  return {
    baseFreq,
    beatFreq,
    volume,
    noiseLevel,
    mistType,
    harmonics,
    emergence,
    spawnMode,
    shepard,
    shepardBase,
    shepardDirection,
    timbre,
  };
}

function isRecord(value: unknown): value is UnknownRecord {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function stringField(value: unknown): string | null {
  return typeof value === 'string' && value.trim() ? value : null;
}

function numberField(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function enumField<T extends number>(
  value: unknown,
  allowed: readonly T[],
): T | null {
  return typeof value === 'number' &&
    Number.isInteger(value) &&
    allowed.includes(value as T)
    ? (value as T)
    : null;
}
