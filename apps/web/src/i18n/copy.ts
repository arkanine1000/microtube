import type { EegBandId, SliderGroupId, SliderKey } from '../audio/params';

export const LOCALES = ['en', 'hr'] as const;

export type Locale = (typeof LOCALES)[number];
export type StudioTab = 'play' | 'shape';

export interface LanguageOption {
  locale: Locale;
  flag: string;
  name: string;
}

type Pair = readonly [string, string];
type Quad = readonly [string, string, string, string];
type Quint = readonly [string, string, string, string, string];
type PresetTuple = readonly [
  PresetText,
  PresetText,
  PresetText,
  PresetText,
  PresetText,
];
type JourneyStepTuple = readonly [
  string,
  string,
  string,
  string,
  string,
  string,
  string,
  string,
  string,
  string,
  string,
  string,
  string,
];

interface LabelCaption {
  label: string;
  caption: string;
}

interface SliderText {
  label: string;
  hint: string;
  decrease: string;
  increase: string;
}

interface BandText {
  name: string;
  blurb: string;
}

interface PresetText {
  name: string;
  description: string;
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
    headphones: string;
    errorPrefix: string;
  };
  topbar: {
    play: string;
    pause: string;
    signalActive: string;
    signalPaused: string;
    timeRemaining: string;
    left: string;
    backToStart: string;
  };
  tabs: Record<StudioTab, LabelCaption>;
  studioSectionsLabel: string;
  timer: {
    autoStop: string;
    off: string;
    stopped: string;
    minutesAbbrev: string;
    decrease: string;
    increase: string;
    minutes: string;
  };
  panels: {
    transport: string;
    presets: string;
    modes: string;
    journey: string;
  };
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
    spawnModes: Pair;
  };
  sliders: Record<SliderKey, SliderText>;
  sliderGroups: Record<SliderGroupId, LabelCaption>;
  bands: Record<EegBandId, BandText>;
  presets: PresetTuple;
  journey: {
    copy: string;
    idle: string;
    stepPrefix: string;
    begin: string;
    stop: string;
    steps: JourneyStepTuple;
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
      headphones:
        'Headphones recommended. The binaural effect lives in the gap between your ears.',
      errorPrefix: 'Engine failed to start:',
    },
    topbar: {
      play: 'play',
      pause: 'pause',
      signalActive: 'signal active',
      signalPaused: 'signal paused',
      timeRemaining: 'session time remaining',
      left: 'left',
      backToStart: 'return to start screen',
    },
    tabs: {
      play: { label: 'Play', caption: 'basic' },
      shape: { label: 'Shape', caption: 'advanced' },
    },
    studioSectionsLabel: 'Studio sections',
    timer: {
      autoStop: 'Auto-stop',
      off: 'off',
      stopped: 'stopped',
      minutesAbbrev: 'min',
      decrease: 'decrease auto-stop timer',
      increase: 'increase auto-stop timer',
      minutes: 'auto-stop minutes',
    },
    panels: {
      transport: 'Transport',
      presets: 'Presets',
      modes: 'Modes',
      journey: 'Journey Through the Cosmos',
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
      spawnModes: ['Canon', 'Penrose'],
    },
    sliders: {
      baseFreq: {
        label: 'Base frequency',
        hint: 'Carrier pitch of the binaural pair',
        decrease: 'decrease base frequency',
        increase: 'increase base frequency',
      },
      beatFreq: {
        label: 'Beat frequency',
        hint: 'L/R offset - sets the EEG band',
        decrease: 'decrease beat frequency',
        increase: 'increase beat frequency',
      },
      harmonics: {
        label: 'Warmth',
        hint: 'Harmonic partials mixed into the carrier',
        decrease: 'decrease warmth',
        increase: 'increase warmth',
      },
      emergence: {
        label: 'Emergence',
        hint: 'Generative canon / quasicrystal voices',
        decrease: 'decrease emergence',
        increase: 'increase emergence',
      },
      noiseLevel: {
        label: 'Mist',
        hint: 'Ambient coloured-noise mist layer',
        decrease: 'decrease mist',
        increase: 'increase mist',
      },
      shepard: {
        label: 'Drift gain',
        hint: 'Shepard-Risset endless-glissando level',
        decrease: 'decrease drift gain',
        increase: 'increase drift gain',
      },
      shepardBase: {
        label: 'Drift base',
        hint: 'Lowest oscillator in the Shepard stack',
        decrease: 'decrease drift base',
        increase: 'increase drift base',
      },
      volume: {
        label: 'Master volume',
        hint: 'Overall output level',
        decrease: 'decrease master volume',
        increase: 'increase master volume',
      },
    },
    sliderGroups: {
      carrier: { label: 'Carrier', caption: 'the binaural pair' },
      texture: { label: 'Texture', caption: 'tone & atmosphere' },
      motion: { label: 'Motion', caption: 'generative movement' },
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
    journey: {
      copy:
        'A guided descent and return. Every parameter automated, interpolating between thirteen named worlds. Hand the controls to the sequence and listen.',
      idle: 'Idle - press begin to set off',
      stepPrefix: 'Step',
      begin: 'Begin journey',
      stop: 'Stop journey',
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
      headphones:
        'Preporučene su slušalice. Binauralni učinak nastaje u razlici između lijevog i desnog uha.',
      errorPrefix: 'Zvučni pogon se nije pokrenuo:',
    },
    topbar: {
      play: 'pokreni',
      pause: 'pauziraj',
      signalActive: 'signal aktivan',
      signalPaused: 'signal pauziran',
      timeRemaining: 'preostalo vrijeme sesije',
      left: 'preostalo',
      backToStart: 'povratak na početni zaslon',
    },
    tabs: {
      play: { label: 'Slušaj', caption: 'osnovno' },
      shape: { label: 'Oblikuj', caption: 'napredno' },
    },
    studioSectionsLabel: 'Dijelovi studija',
    timer: {
      autoStop: 'Auto-stop',
      off: 'isključeno',
      stopped: 'zaustavljeno',
      minutesAbbrev: 'min',
      decrease: 'smanji auto-stop tajmer',
      increase: 'povećaj auto-stop tajmer',
      minutes: 'minute auto-stop tajmera',
    },
    panels: {
      transport: 'Kontrole',
      presets: 'Preseti',
      modes: 'Načini',
      journey: 'Putovanje kroz svemir',
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
      spawnModes: ['Kanon', 'Penrose'],
    },
    sliders: {
      baseFreq: {
        label: 'Osnovna frekvencija',
        hint: 'Nosiva visina binauralnog para',
        decrease: 'smanji osnovnu frekvenciju',
        increase: 'povećaj osnovnu frekvenciju',
      },
      beatFreq: {
        label: 'Beat frekvencija',
        hint: 'L/D pomak - određuje EEG pojas',
        decrease: 'smanji beat frekvenciju',
        increase: 'povećaj beat frekvenciju',
      },
      harmonics: {
        label: 'Toplina',
        hint: 'Harmonijski parcijali pomiješani s nosiocem',
        decrease: 'smanji toplinu',
        increase: 'povećaj toplinu',
      },
      emergence: {
        label: 'Emergencija',
        hint: 'Generativni kanon / kvazikristalni glasovi',
        decrease: 'smanji emergenciju',
        increase: 'povećaj emergenciju',
      },
      noiseLevel: {
        label: 'Maglica',
        hint: 'Sloj ambijentalnog obojenog šuma',
        decrease: 'smanji maglicu',
        increase: 'povećaj maglicu',
      },
      shepard: {
        label: 'Jačina klizanja',
        hint: 'Razina Shepard-Risset beskonačnog glissanda',
        decrease: 'smanji jačinu klizanja',
        increase: 'povećaj jačinu klizanja',
      },
      shepardBase: {
        label: 'Baza klizanja',
        hint: 'Najniži oscilator u Shepardovu sloju',
        decrease: 'smanji bazu klizanja',
        increase: 'povećaj bazu klizanja',
      },
      volume: {
        label: 'Glavna glasnoća',
        hint: 'Ukupna izlazna razina',
        decrease: 'smanji glavnu glasnoću',
        increase: 'povećaj glavnu glasnoću',
      },
    },
    sliderGroups: {
      carrier: { label: 'Nositelj', caption: 'binauralni par' },
      texture: { label: 'Tekstura', caption: 'ton i atmosfera' },
      motion: { label: 'Kretanje', caption: 'generativni pomak' },
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
    journey: {
      copy:
        'Vođeni silazak i povratak. Svaki parametar je automatiziran i prelazi između trinaest imenovanih svjetova. Prepusti kontrole sekvenci i slušaj.',
      idle: 'Miruje - pritisni početak za polazak',
      stepPrefix: 'Korak',
      begin: 'Počni putovanje',
      stop: 'Zaustavi putovanje',
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
    footer: {
      engine: 'pogon',
    },
  },
};
