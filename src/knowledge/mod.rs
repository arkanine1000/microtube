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

    use crate::theme::{INK_0, INK_3, INK_4};

    let panes = [
        ('1', KnowledgePane::MicroTube),
        ('2', KnowledgePane::Wiki),
        ('3', KnowledgePane::Glossary),
    ];

    // --- Row 1: tab labels (browser-style — no chip background) ----------
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(panes.len() * 3 + 1);
    spans.push(Span::raw("  "));
    let mut tab_positions: Vec<(u16, u16, bool)> = Vec::with_capacity(panes.len());
    let mut x_cursor: u16 = 2;
    for (key, pane) in panes {
        let is_active = pane == active;
        let label = format!("{key} {}", pane.label());
        let label_str = format!("  {label}  ");
        let len = label_str.chars().count() as u16;
        let style = if is_active {
            Style::default().fg(INK_0).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(INK_3)
        };
        spans.push(Span::styled(label_str, style));
        tab_positions.push((x_cursor, len, is_active));
        x_cursor = x_cursor.saturating_add(len);
    }
    let line1 = Line::from(spans);
    f.render_widget(Paragraph::new(line1), area);

    // --- Row 2: hairline rule + active underline -------------------------
    if area.height >= 2 {
        let buf = f.buffer_mut();
        let y1 = area.y + 1;
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, y1)];
            cell.set_char('\u{2500}');
            cell.set_fg(INK_4);
        }
        for (tx, len, is_active) in tab_positions {
            for i in 0..len {
                let xx = area.x + tx + i;
                if xx >= area.x + area.width {
                    break;
                }
                let cell = &mut buf[(xx, y1)];
                if is_active {
                    cell.set_char('\u{2501}');
                    cell.set_fg(accent);
                } else {
                    cell.set_char('\u{2500}');
                    cell.set_fg(INK_4);
                }
            }
        }
    }
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
