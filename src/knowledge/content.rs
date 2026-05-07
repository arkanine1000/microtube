//! Content manifest. Articles and the glossary file are baked into the
//! binary at compile time via `include_str!`. Adding a new article means
//! one entry here plus one new file under `knowledge_assets/wiki/`.

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

pub static GLOSSARY_TOML: &str = include_str!("../knowledge_assets/glossary.toml");
