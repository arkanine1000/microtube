import {
  AudioWaveform,
  CloudFog,
  Orbit,
  Sparkles,
  Waves,
  type LucideIcon,
} from 'lucide-react';
import type { MicroTubeState } from './audio/params';

export type SectionId =
  | 'signal'
  | 'mist'
  | 'emergence'
  | 'drift'
  | 'sequences';

export interface SectionDef {
  id: SectionId;
  icon: LucideIcon;
  /**
   * Whether the section's function is audibly on right now — drives the
   * status dot on its dock tab. Absent for always-on sections.
   */
  isOn?: (state: MicroTubeState, sequenceActive: boolean) => boolean;
}

/** Deck sections in dock order. */
export const SECTIONS: SectionDef[] = [
  { id: 'signal', icon: AudioWaveform },
  { id: 'mist', icon: CloudFog, isOn: (s) => s.noiseLevel > 0 },
  { id: 'emergence', icon: Sparkles, isOn: (s) => s.emergence > 0 },
  { id: 'drift', icon: Waves, isOn: (s) => s.shepard > 0 },
  { id: 'sequences', icon: Orbit, isOn: (_s, seq) => seq },
];
