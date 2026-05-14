//! Knowledge tab — a wiki and glossary that ship inside the binary so
//! the program teaches itself.
//!
//! The tab is structured as three sub-panes (MicroTube / Wiki / Glossary),
//! each with its own state. All UI is driven by the same 30 Hz tick that drives Studio.
//! The audio thread is never touched from this module.

pub mod content;
pub mod glossary;
pub mod markdown;
pub mod wiki;

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::app::App;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KnowledgePane {
    MicroTube,
    Wiki,
    Glossary,
}

impl KnowledgePane {
    pub fn label(self) -> &'static str {
        match self {
            Self::MicroTube => "MicroTube",
            Self::Wiki => "Wiki",
            Self::Glossary => "Glossary",
        }
    }
}

pub struct KnowledgeState {
    pub pane: KnowledgePane,
    pub microtube: wiki::WikiState,
    pub wiki: wiki::WikiState,
    pub glossary: glossary::GlossaryState,
}

impl KnowledgeState {
    pub fn new() -> Self {
        Self {
            pane: KnowledgePane::MicroTube,
            microtube: wiki::WikiState::new(),
            wiki: wiki::WikiState::new(),
            glossary: glossary::GlossaryState::new(),
        }
    }

    /// True when the active sub-view should consume number keys etc. that
    /// would otherwise switch panes.
    pub fn is_capturing_input(&self) -> bool {
        match self.pane {
            KnowledgePane::MicroTube => self.microtube.in_reader,
            KnowledgePane::Wiki => self.wiki.in_reader,
            KnowledgePane::Glossary => self.glossary.expanded,
        }
    }

    /// Jump to a wiki article by slug. Used by glossary cross-references.
    pub fn open_wiki_article(&mut self, slug: &str) {
        if let Some(idx) = content::ARTICLES.iter().position(|a| a.slug == slug) {
            self.pane = KnowledgePane::Wiki;
            self.wiki.selected = idx;
            self.wiki.scroll = 0;
            self.wiki.in_reader = true;
        }
    }
}

impl Default for KnowledgeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the Knowledge body into `area`.
pub fn draw(f: &mut Frame, area: Rect, app: &mut App, accent: Color, border: Color) {
    use ratatui::layout::{Constraint, Direction, Layout};

    if area.height < 4 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(3)])
        .split(area);

    draw_pane_strip(f, chunks[0], app.knowledge.pane, accent);

    match app.knowledge.pane {
        KnowledgePane::MicroTube => wiki::draw_articles(
            f,
            chunks[1],
            &mut app.knowledge.microtube,
            accent,
            border,
            content::MICROTUBE_ARTICLES,
            "MICROTUBE",
        ),
        KnowledgePane::Wiki => wiki::draw(f, chunks[1], &mut app.knowledge.wiki, accent, border),
        KnowledgePane::Glossary => {
            glossary::draw(f, chunks[1], &mut app.knowledge.glossary, accent, border)
        }
    }
}

fn draw_pane_strip(f: &mut Frame, area: Rect, active: KnowledgePane, accent: Color) {
    use ratatui::style::{Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    let panes = [
        ('1', KnowledgePane::MicroTube),
        ('2', KnowledgePane::Wiki),
        ('3', KnowledgePane::Glossary),
    ];
    let dim = Color::Rgb(138, 146, 168);
    let bg_top = Color::Rgb(5, 7, 14);
    let panel_bg = Color::Rgb(9, 12, 22);

    let mut spans: Vec<Span<'static>> = Vec::with_capacity(panes.len() * 3);
    spans.push(Span::styled(" ", Style::default().bg(bg_top)));
    for (key, pane) in panes {
        let is_active = pane == active;
        let label_style = if is_active {
            Style::default()
                .fg(bg_top)
                .bg(accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(dim).bg(panel_bg)
        };
        spans.push(Span::styled(format!(" [{key}] "), label_style));
        spans.push(Span::styled(format!(" {} ", pane.label()), label_style));
        spans.push(Span::styled("  ", Style::default().bg(bg_top)));
    }
    spans.push(Span::styled(
        " Tab: Studio  ",
        Style::default().fg(dim).bg(bg_top),
    ));

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(bg_top)),
        area,
    );
}

/// Returns true if the key was handled by the Knowledge tab.
pub fn handle_key(app: &mut App, code: crossterm::event::KeyCode) -> bool {
    use crossterm::event::KeyCode;

    if !app.knowledge.is_capturing_input() {
        match code {
            KeyCode::Char('1') => {
                app.knowledge.pane = KnowledgePane::MicroTube;
                return true;
            }
            KeyCode::Char('2') => {
                app.knowledge.pane = KnowledgePane::Wiki;
                return true;
            }
            KeyCode::Char('3') => {
                app.knowledge.pane = KnowledgePane::Glossary;
                return true;
            }
            _ => {}
        }
    }

    match app.knowledge.pane {
        KnowledgePane::MicroTube => wiki::handle_key_for(
            &mut app.knowledge.microtube,
            code,
            content::MICROTUBE_ARTICLES.len(),
        ),
        KnowledgePane::Wiki => wiki::handle_key(&mut app.knowledge.wiki, code),
        KnowledgePane::Glossary => glossary::handle_key(app, code),
    }
}
