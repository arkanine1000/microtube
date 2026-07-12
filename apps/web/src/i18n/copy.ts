import type { EegBandId, SliderKey } from '../audio/params';
import type { SequenceId } from '../audio/sequences';
import type { SectionId } from '../sections';

export const LOCALES = ['en', 'hr'] as const;

export type Locale = (typeof LOCALES)[number];

export interface LanguageOption {
  locale: Locale;
  flag: string;
  name: string;
}

type Pair = readonly [string, string];
type Trio = readonly [string, string, string];
type Quad = readonly [string, string, string, string];
type Quint = readonly [string, string, string, string, string];
type PresetTuple = readonly [
  PresetText,
  PresetText,
  PresetText,
  PresetText,
  PresetText,
];
interface SectionText {
  label: string;
  /** Compact form for the bottom dock's tab label. */
  short: string;
}

interface SliderText {
  label: string;
  hint: string;
}

interface BandText {
  name: string;
  blurb: string;
}

interface PresetText {
  name: string;
  description: string;
}

interface SequenceText {
  name: string;
  description: string;
  steps: readonly string[];
}

export interface Copy {
  language: {
    selectorLabel: string;
    toggleLabel: string;
    menuLabel: string;
  };
  start: {
    tagline: string;
    enter: string;
    loading: string;
    errorPrefix: string;
  };
  header: {
    play: string;
    pause: string;
    timeRemaining: string;
    left: string;
    backToStart: string;
    timer: string;
    presets: string;
  };
  studioSectionsLabel: string;
  timer: {
    autoStop: string;
    off: string;
    stopped: string;
    minutesAbbrev: string;
    minutes: string;
  };
  sections: Record<SectionId, SectionText>;
  localPresets: {
    saveCurrent: string;
    empty: string;
    saveTitle: string;
    nameLabel: string;
    saveAction: string;
    cancel: string;
    close: string;
    nameRequired: string;
    storageError: string;
    loadPreset: string;
    deletePreset: string;
    deleteTitle: string;
    deleteAction: string;
    deletePromptPrefix: string;
    deletePromptSuffix: string;
    defaultName: string;
    beatLabel: string;
    baseLabel: string;
    mistLabel: string;
    driftLabel: string;
    showMore: string;
  };
  modes: {
    captions: {
      timbre: string;
      mist: string;
      direction: string;
      spawn: string;
    };
    status: {
      on: string;
      off: string;
    };
    timbres: Quad;
    mists: Quint;
    directions: Pair;
    spawnModes: Trio;
  };
  sliders: Record<SliderKey, SliderText>;
  slider: {
    fine: string;
    editValue: string;
    apply: string;
    cancel: string;
  };
  bands: Record<EegBandId, BandText>;
  presets: PresetTuple;
  sequences: {
    intro: string;
    idle: string;
    running: string;
    stepPrefix: string;
    start: string;
    stop: string;
    cards: Record<SequenceId, SequenceText>;
  };
  footer: {
    engine: string;
  };
}

export const LANGUAGE_OPTIONS: readonly LanguageOption[] = [
  { locale: 'en', flag: '🇺🇸', name: 'English' },
  { locale: 'hr', flag: '🇭🇷', name: 'Hrvatski' },
];

export const COPY: Record<Locale, Copy> = {
  en: {
    language: {
      selectorLabel: 'Language',
      toggleLabel: 'Change language',
      menuLabel: 'Choose language',
    },
    start: {
      tagline: 'Tune your mind to a frequency.',
      enter: 'Enter studio',
      loading: 'Spinning up engine...',
      errorPrefix: 'Engine failed to start:',
    },
    header: {
      play: 'play',
      pause: 'pause',
      timeRemaining: 'session time remaining',
      left: 'left',
      backToStart: 'return to start screen',
      timer: 'Auto-stop timer',
      presets: 'Presets',
    },
    studioSectionsLabel: 'Studio sections',
    timer: {
      autoStop: 'Auto-stop',
      off: 'off',
      stopped: 'stopped',
      minutesAbbrev: 'min',
      minutes: 'auto-stop minutes',
    },
    sections: {
      signal: { label: 'Signal', short: 'Signal' },
      mist: { label: 'Mist', short: 'Mist' },
      emergence: { label: 'Emergence', short: 'Emerge' },
      drift: { label: 'Drift', short: 'Drift' },
      sequences: { label: 'Sequences', short: 'Seqs' },
    },
    localPresets: {
      saveCurrent: 'Save current sound',
      empty: 'No saved presets yet.',
      saveTitle: 'Save preset',
      nameLabel: 'Preset name',
      saveAction: 'Save preset',
      cancel: 'Cancel',
      close: 'Close modal',
      nameRequired: 'Enter a preset name.',
      storageError: 'Preset storage is unavailable in this browser.',
      loadPreset: 'Load preset',
      deletePreset: 'Delete preset',
      deleteTitle: 'Delete preset',
      deleteAction: 'Delete',
      deletePromptPrefix: 'Delete ',
      deletePromptSuffix: '? This cannot be undone.',
      defaultName: 'Custom Preset',
      beatLabel: 'beat',
      baseLabel: 'base',
      mistLabel: 'mist',
      driftLabel: 'drift',
      showMore: 'Show more',
    },
    modes: {
      captions: {
        timbre: 'Timbre',
        mist: 'Mist colour',
        direction: 'Drift direction',
        spawn: 'Emergence spawn',
      },
      status: {
        on: 'on',
        off: 'off',
      },
      timbres: ['Organ', 'Flute', 'Bell', 'Saw'],
      mists: ['Pink', 'White', 'Brown', 'Blue', 'Velvet'],
      directions: ['Rising', 'Falling'],
      spawnModes: ['Canon', 'Penrose', 'Fuxian'],
    },
    sliders: {
      baseFreq: {
        label: 'Base frequency',
        hint: 'Carrier pitch of the binaural pair',
      },
      beatFreq: {
        label: 'Beat frequency',
        hint: 'L/R offset - sets the EEG band',
      },
      harmonics: {
        label: 'Warmth',
        hint: 'Harmonic partials mixed into the carrier',
      },
      emergence: {
        label: 'Emergence',
        hint: 'Generative canon / quasicrystal voices',
      },
      gravity: {
        label: 'Gravity',
        hint: 'Fuxian pull toward the root',
      },
      noiseLevel: {
        label: 'Mist',
        hint: 'Ambient coloured-noise mist layer',
      },
      shepard: {
        label: 'Drift gain',
        hint: 'Shepard-Risset endless-glissando level',
      },
      shepardBase: {
        label: 'Drift base',
        hint: 'Lowest oscillator in the Shepard stack',
      },
      volume: {
        label: 'Master volume',
        hint: 'Overall output level',
      },
    },
    slider: {
      fine: 'fine',
      editValue: 'enter exact value',
      apply: 'OK',
      cancel: 'Cancel',
    },
    bands: {
      delta: { name: 'Delta', blurb: 'sleep' },
      theta: { name: 'Theta', blurb: 'meditation' },
      alpha: { name: 'Alpha', blurb: 'calm focus' },
      beta: { name: 'Beta', blurb: 'alertness' },
      gamma: { name: 'Gamma', blurb: 'peak insight' },
    },
    presets: [
      {
        name: 'Sleep',
        description: 'Delta 2 Hz - deep dreamless sleep',
      },
      {
        name: 'Meditation',
        description: 'Theta 6 Hz - meditation, creativity',
      },
      {
        name: 'Relaxation',
        description: 'Alpha 10 Hz - calm, relaxed awareness',
      },
      {
        name: 'Focus',
        description: 'Beta 18 Hz - concentration, alertness',
      },
      {
        name: 'Flow State',
        description: 'Gamma 40 Hz - peak performance, insight',
      },
    ],
    sequences: {
      intro:
        'Timed programs automate the binaural pair, with Journey also shaping tone, mist, emergence, and drift.',
      idle: 'Ready',
      running: 'Running',
      stepPrefix: 'Step',
      start: 'Start',
      stop: 'Stop',
      cards: {
        'deep-focus': {
          name: 'Deep Focus',
          description: '25 min: Beta -> Alpha -> Theta',
          steps: ['Beta focus', 'Alpha settle', 'Theta landing'],
        },
        'wake-up': {
          name: 'Wake Up',
          description: '10 min: Delta -> Theta -> Alpha -> Beta',
          steps: ['Delta', 'Theta', 'Alpha', 'Beta'],
        },
        'power-nap': {
          name: 'Power Nap',
          description: '20 min: Alpha -> Theta -> Alpha -> Beta',
          steps: ['Alpha descent', 'Theta nap', 'Alpha return', 'Beta lift'],
        },
        'deep-meditation': {
          name: 'Deep Meditation',
          description: '30 min: Alpha -> Theta -> Deep -> Alpha',
          steps: ['Alpha gate', 'Theta field', 'Deep theta', 'Alpha return'],
        },
        'orch-or': {
          name: 'Orch-OR',
          description: '25 min: Gamma -> Schumann -> Gamma -> Theta',
          steps: ['Gamma', 'Schumann', 'Gamma return', 'Theta'],
        },
        'journey-through-cosmos': {
          name: 'Journey Through the Cosmos',
          description: '25 min: Microtubule -> Cosmos -> Strange Loop',
          steps: [
            'Microtubule',
            'Synapse',
            'Neural Awareness',
            'Body',
            'Earth · Schumann',
            'Lunar Tide',
            'Solar Wind',
            'Stellar Bells',
            'Galactic',
            'Cosmic Web',
            'Background Radiation',
            'Singularity',
            'Strange Loop',
          ],
        },
      },
    },
    footer: {
      engine: 'engine',
    },
  },
  hr: {
    language: {
      selectorLabel: 'Jezik',
      toggleLabel: 'Promijeni jezik',
      menuLabel: 'Odaberi jezik',
    },
    start: {
      tagline: 'Uskladi um s frekvencijom.',
      enter: 'Uđi u studio',
      loading: 'Pokrećem studio...',
      errorPrefix: 'Zvučni pogon se nije pokrenuo:',
    },
    header: {
      play: 'pokreni',
      pause: 'pauziraj',
      timeRemaining: 'preostalo vrijeme sesije',
      left: 'preostalo',
      backToStart: 'povratak na početni zaslon',
      timer: 'Auto-stop tajmer',
      presets: 'Preseti',
    },
    studioSectionsLabel: 'Dijelovi studija',
    timer: {
      autoStop: 'Auto-stop',
      off: 'isključeno',
      stopped: 'zaustavljeno',
      minutesAbbrev: 'min',
      minutes: 'minute auto-stop tajmera',
    },
    sections: {
      signal: { label: 'Signal', short: 'Signal' },
      mist: { label: 'Maglica', short: 'Maglica' },
      emergence: { label: 'Emergencija', short: 'Emerg.' },
      drift: { label: 'Klizanje', short: 'Kliz.' },
      sequences: { label: 'Sekvence', short: 'Sekv.' },
    },
    localPresets: {
      saveCurrent: 'Spremi trenutni zvuk',
      empty: 'Još nema spremljenih preseta.',
      saveTitle: 'Spremi preset',
      nameLabel: 'Naziv preseta',
      saveAction: 'Spremi preset',
      cancel: 'Odustani',
      close: 'Zatvori modal',
      nameRequired: 'Unesi naziv preseta.',
      storageError: 'Pohrana preseta nije dostupna u ovom pregledniku.',
      loadPreset: 'Učitaj preset',
      deletePreset: 'Izbriši preset',
      deleteTitle: 'Izbriši preset',
      deleteAction: 'Izbriši',
      deletePromptPrefix: 'Izbrisati ',
      deletePromptSuffix: '? To se ne može poništiti.',
      defaultName: 'Moj preset',
      beatLabel: 'beat',
      baseLabel: 'baza',
      mistLabel: 'maglica',
      driftLabel: 'klizanje',
      showMore: 'Prikaži još',
    },
    modes: {
      captions: {
        timbre: 'Boja tona',
        mist: 'Boja maglice',
        direction: 'Smjer klizanja',
        spawn: 'Način emergencije',
      },
      status: {
        on: 'uklj.',
        off: 'isklj.',
      },
      timbres: ['Orgulje', 'Flauta', 'Zvono', 'Pila'],
      mists: ['Ružičasta', 'Bijela', 'Smeđa', 'Plava', 'Baršun'],
      directions: ['Uzlazno', 'Silazno'],
      spawnModes: ['Kanon', 'Penrose', 'Fuxian'],
    },
    sliders: {
      baseFreq: {
        label: 'Osnovna frekvencija',
        hint: 'Nosiva visina binauralnog para',
      },
      beatFreq: {
        label: 'Beat frekvencija',
        hint: 'L/D pomak - određuje EEG pojas',
      },
      harmonics: {
        label: 'Toplina',
        hint: 'Harmonijski parcijali pomiješani s nosiocem',
      },
      emergence: {
        label: 'Emergencija',
        hint: 'Generativni kanon / kvazikristalni glasovi',
      },
      gravity: {
        label: 'Gravitacija',
        hint: 'Fuxian privlačenje prema korijenu',
      },
      noiseLevel: {
        label: 'Maglica',
        hint: 'Sloj ambijentalnog obojenog šuma',
      },
      shepard: {
        label: 'Jačina klizanja',
        hint: 'Razina Shepard-Risset beskonačnog glissanda',
      },
      shepardBase: {
        label: 'Baza klizanja',
        hint: 'Najniži oscilator u Shepardovu sloju',
      },
      volume: {
        label: 'Glavna glasnoća',
        hint: 'Ukupna izlazna razina',
      },
    },
    slider: {
      fine: 'fino',
      editValue: 'unesi točnu vrijednost',
      apply: 'U redu',
      cancel: 'Odustani',
    },
    bands: {
      delta: { name: 'Delta', blurb: 'san' },
      theta: { name: 'Theta', blurb: 'meditacija' },
      alpha: { name: 'Alpha', blurb: 'smiren fokus' },
      beta: { name: 'Beta', blurb: 'budnost' },
      gamma: { name: 'Gamma', blurb: 'vrhunski uvid' },
    },
    presets: [
      {
        name: 'San',
        description: 'Delta 2 Hz - duboki san bez snova',
      },
      {
        name: 'Meditacija',
        description: 'Theta 6 Hz - meditacija, kreativnost',
      },
      {
        name: 'Opuštanje',
        description: 'Alpha 10 Hz - mirna, opuštena svjesnost',
      },
      {
        name: 'Fokus',
        description: 'Beta 18 Hz - koncentracija, budnost',
      },
      {
        name: 'Stanje toka',
        description: 'Gamma 40 Hz - vrhunska izvedba, uvid',
      },
    ],
    sequences: {
      intro:
        'Vremenski programi automatiziraju binauralni par, a Putovanje oblikuje i ton, maglicu, emergenciju i klizanje.',
      idle: 'Spremno',
      running: 'U tijeku',
      stepPrefix: 'Korak',
      start: 'Pokreni',
      stop: 'Zaustavi',
      cards: {
        'deep-focus': {
          name: 'Duboki fokus',
          description: '25 min: Beta -> Alpha -> Theta',
          steps: ['Beta fokus', 'Alpha smirenje', 'Theta spuštanje'],
        },
        'wake-up': {
          name: 'Buđenje',
          description: '10 min: Delta -> Theta -> Alpha -> Beta',
          steps: ['Delta', 'Theta', 'Alpha', 'Beta'],
        },
        'power-nap': {
          name: 'Kratki san',
          description: '20 min: Alpha -> Theta -> Alpha -> Beta',
          steps: ['Alpha spuštanje', 'Theta san', 'Alpha povratak', 'Beta dizanje'],
        },
        'deep-meditation': {
          name: 'Duboka meditacija',
          description: '30 min: Alpha -> Theta -> Duboko -> Alpha',
          steps: ['Alpha ulaz', 'Theta polje', 'Duboka theta', 'Alpha povratak'],
        },
        'orch-or': {
          name: 'Orch-OR',
          description: '25 min: Gamma -> Schumann -> Gamma -> Theta',
          steps: ['Gamma', 'Schumann', 'Gamma povratak', 'Theta'],
        },
        'journey-through-cosmos': {
          name: 'Putovanje kroz svemir',
          description: '25 min: Mikrotubul -> svemir -> čudna petlja',
          steps: [
            'Mikrotubul',
            'Sinapsa',
            'Neuralna svjesnost',
            'Tijelo',
            'Zemlja · Schumann',
            'Mjesečeva plima',
            'Sunčev vjetar',
            'Zvjezdana zvona',
            'Galaktika',
            'Kozmička mreža',
            'Pozadinsko zračenje',
            'Singularnost',
            'Čudna petlja',
          ],
        },
      },
    },
    footer: {
      engine: 'pogon',
    },
  },
};
