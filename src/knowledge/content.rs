//! Content manifests. Articles and the glossary file are baked into the
//! binary at compile time via `include_str!`. Adding a new article means
//! one entry here plus one new file under `knowledge_assets/`.

pub struct ArticleManifest {
    pub slug: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub summary: &'static str,
    pub body: &'static str,
}

pub static ARTICLES: &[ArticleManifest] = &[
    ArticleManifest {
        slug: "welcome",
        title: "Welcome to MicroTube",
        category: "Overview",
        summary: "Start here. Tour of the Knowledge tab and what the program is for.",
        body: include_str!("../knowledge_assets/wiki/welcome.md"),
    },
    ArticleManifest {
        slug: "binaural-beats",
        title: "Binaural Beats",
        category: "Audio",
        summary: "Two ears, two tones, one phantom beat. The math, the brain, and the evidence.",
        body: include_str!("../knowledge_assets/wiki/binaural-beats.md"),
    },
    ArticleManifest {
        slug: "consonance",
        title: "The Consonance Score",
        category: "Algorithm",
        summary: "How the emergence engine decides which voices live longer.",
        body: include_str!("../knowledge_assets/wiki/consonance.md"),
    },
    ArticleManifest {
        slug: "raised-cosine",
        title: "The Raised-Cosine Bell",
        category: "Audio",
        summary: "Why the Shepard\u{2013}Risset window is sin^4, not sin or sin^2.",
        body: include_str!("../knowledge_assets/wiki/raised-cosine.md"),
    },
    ArticleManifest {
        slug: "mist-textures",
        title: "Mist Textures",
        category: "Audio",
        summary: "Pink, white, brown, blue, velvet \u{2014} five noise colours and what they're for.",
        body: include_str!("../knowledge_assets/wiki/mist-textures.md"),
    },
    ArticleManifest {
        slug: "timbre-design",
        title: "Timbre Design",
        category: "Audio",
        summary: "Organ, flute, bell, saw \u{2014} the four harmonic profiles and the tradeoffs.",
        body: include_str!("../knowledge_assets/wiki/timbre-design.md"),
    },
    ArticleManifest {
        slug: "phase-accumulator",
        title: "Phase Accumulators",
        category: "Algorithm",
        summary: "Why MicroTube's oscillators don't drift over hours.",
        body: include_str!("../knowledge_assets/wiki/phase-accumulator.md"),
    },
    ArticleManifest {
        slug: "fibonacci-quasicrystal",
        title: "The Fibonacci Word",
        category: "Math",
        summary: "Penrose tilings, Conway worms, and the canonical 1D quasicrystal.",
        body: include_str!("../knowledge_assets/wiki/fibonacci-quasicrystal.md"),
    },
];

pub static MICROTUBE_ARTICLES: &[ArticleManifest] = &[
    ArticleManifest {
        slug: "microtube-first-listen",
        title: "First Listen",
        category: "Start",
        summary: "What you are hearing, what you are seeing, and what to try first.",
        body: include_str!("../knowledge_assets/microtube/first-listen.md"),
    },
    ArticleManifest {
        slug: "microtube-signal-flow",
        title: "The Signal Path",
        category: "Engine",
        summary: "The whole engine as a simple path from settings to sound.",
        body: include_str!("../knowledge_assets/microtube/signal-flow.md"),
    },
    ArticleManifest {
        slug: "microtube-controls",
        title: "Studio Controls",
        category: "Manual",
        summary: "How to steer MicroTube without needing to know the theory first.",
        body: include_str!("../knowledge_assets/microtube/controls.md"),
    },
    ArticleManifest {
        slug: "microtube-bands",
        title: "Beat Bands",
        category: "Listening",
        summary: "Delta, theta, alpha, beta, gamma: useful labels, not magic switches.",
        body: include_str!("../knowledge_assets/microtube/beat-bands.md"),
    },
    ArticleManifest {
        slug: "microtube-timbre",
        title: "Tone And Warmth",
        category: "Engine",
        summary: "Why the same beat can sound pure, hollow, glassy, or bright.",
        body: include_str!("../knowledge_assets/microtube/tone-and-warmth.md"),
    },
    ArticleManifest {
        slug: "microtube-mist",
        title: "Mist Layer",
        category: "Engine",
        summary: "The colored-noise bed that gives the carrier a room to live in.",
        body: include_str!("../knowledge_assets/microtube/mist-layer.md"),
    },
    ArticleManifest {
        slug: "microtube-shepard",
        title: "Endless Drift",
        category: "Engine",
        summary: "The rising or falling Shepard-Risset layer in plain language.",
        body: include_str!("../knowledge_assets/microtube/endless-drift.md"),
    },
    ArticleManifest {
        slug: "microtube-emergence",
        title: "Emergence",
        category: "Engine",
        summary: "The small voice ecosystem that grows around the carrier.",
        body: include_str!("../knowledge_assets/microtube/emergence.md"),
    },
    ArticleManifest {
        slug: "microtube-penrose",
        title: "Penrose Mode",
        category: "Engine",
        summary: "How the Fibonacci worm chooses musical moves without repeating.",
        body: include_str!("../knowledge_assets/microtube/penrose-mode.md"),
    },
    ArticleManifest {
        slug: "microtube-partials",
        title: "Partials And The Harmonics View",
        category: "Engine",
        summary: "What H1-H6 mean and how the Partials panel and Harmonics view fit together.",
        body: include_str!("../knowledge_assets/microtube/partials-and-harmonics.md"),
    },
    ArticleManifest {
        slug: "microtube-visuals",
        title: "Reading The Visuals",
        category: "Manual",
        summary: "What each visualization is trying to reveal.",
        body: include_str!("../knowledge_assets/microtube/visuals.md"),
    },
    ArticleManifest {
        slug: "microtube-sequences",
        title: "Presets And Sequences",
        category: "Manual",
        summary: "Quick moods, long journeys, and how automation moves the controls.",
        body: include_str!("../knowledge_assets/microtube/presets-and-sequences.md"),
    },
    ArticleManifest {
        slug: "microtube-listening",
        title: "Listening Notes",
        category: "Care",
        summary: "Headphones, volume, expectations, and practical caution.",
        body: include_str!("../knowledge_assets/microtube/listening-notes.md"),
    },
];

pub static GLOSSARY_TOML: &str = include_str!("../knowledge_assets/glossary.toml");
