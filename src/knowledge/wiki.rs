//! Wiki sub-pane: a list of articles + a scrollable reader for each.

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};

use crate::knowledge::content::ARTICLES;
use crate::knowledge::markdown::{self, MdStyle};

pub struct WikiState {
    pub selected: usize,
    pub scroll: u16,
    pub in_reader: bool,
    pub list_scroll: usize,
    /// Cached visible viewport height for the reader, used by `G` to jump
    /// to the bottom on demand.
    last_reader_height: u16,
    last_reader_lines: u16,
}

impl WikiState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            scroll: 0,
            in_reader: false,
            list_scroll: 0,
            last_reader_height: 0,
            last_reader_lines: 0,
        }
    }
}

impl Default for WikiState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn handle_key(state: &mut WikiState, code: KeyCode) -> bool {
    let len = ARTICLES.len();

    if state.in_reader {
        match code {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') | KeyCode::Char('h') | KeyCode::Left => {
                state.in_reader = false;
                state.scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                state.scroll = state.scroll.saturating_add(1);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                state.scroll = state.scroll.saturating_sub(1);
            }
            KeyCode::Char('d') | KeyCode::PageDown => {
                let step = state.last_reader_height.max(1).saturating_sub(1);
                state.scroll = state.scroll.saturating_add(step);
            }
            KeyCode::Char('u') | KeyCode::PageUp => {
                let step = state.last_reader_height.max(1).saturating_sub(1);
                state.scroll = state.scroll.saturating_sub(step);
            }
            KeyCode::Char('g') | KeyCode::Home => {
                state.scroll = 0;
            }
            KeyCode::Char('G') | KeyCode::End => {
                state.scroll = state
                    .last_reader_lines
                    .saturating_sub(state.last_reader_height);
            }
            _ => return true,
        }
        return true;
    }

    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            if state.selected + 1 < len {
                state.selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.selected > 0 {
                state.selected -= 1;
            }
        }
        KeyCode::Char('d') | KeyCode::PageDown => {
            state.selected = (state.selected + 5).min(len.saturating_sub(1));
        }
        KeyCode::Char('u') | KeyCode::PageUp => {
            state.selected = state.selected.saturating_sub(5);
        }
        KeyCode::Char('g') | KeyCode::Home => {
            state.selected = 0;
        }
        KeyCode::Char('G') | KeyCode::End => {
            if len > 0 {
                state.selected = len - 1;
            }
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if len > 0 {
                state.in_reader = true;
                state.scroll = 0;
            }
        }
        _ => return false,
    }
    true
}

pub fn draw(f: &mut Frame, area: Rect, state: &mut WikiState, accent: Color, border: Color) {
    if state.in_reader {
        draw_reader(f, area, state, accent, border);
    } else {
        draw_index(f, area, state, accent, border);
    }
}

fn draw_index(f: &mut Frame, area: Rect, state: &mut WikiState, accent: Color, border: Color) {
    let panel_bg = Color::Rgb(9, 12, 22);
    let bright = Color::Rgb(236, 240, 248);
    let soft = Color::Rgb(178, 184, 204);
    let dim = Color::Rgb(138, 146, 168);
    let select_fg = Color::Rgb(5, 7, 14);

    let title = format!(
        " WIKI  {}/{} ",
        if ARTICLES.is_empty() { 0 } else { state.selected + 1 },
        ARTICLES.len()
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(panel_bg))
        .title(Span::styled(
            title,
            Style::default().fg(bright).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Line::from(Span::styled(
            "  j/k move   d/u page   g/G top/bot   Enter open  ",
            Style::default().fg(dim),
        )));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    if state.selected < state.list_scroll {
        state.list_scroll = state.selected;
    } else if state.selected >= state.list_scroll + height && height > 0 {
        state.list_scroll = state.selected + 1 - height;
    }
    if state.list_scroll + height > ARTICLES.len() {
        state.list_scroll = ARTICLES.len().saturating_sub(height);
    }

    let total = inner.width as usize;
    let num_w = 4;
    let title_w = 28;
    let cat_w = 12;
    let summary_w = total
        .saturating_sub(num_w + title_w + cat_w + 6);

    let visible_end = (state.list_scroll + height).min(ARTICLES.len());
    let items: Vec<ListItem> = ARTICLES[state.list_scroll..visible_end]
        .iter()
        .enumerate()
        .map(|(local, article)| {
            let global = state.list_scroll + local;
            let is_selected = global == state.selected;
            let bg = if is_selected { accent } else { panel_bg };
            let title_fg = if is_selected { select_fg } else { bright };
            let num_fg = if is_selected { select_fg } else { dim };
            let cat_fg = if is_selected { select_fg } else { dim };
            let blurb_fg = if is_selected { select_fg } else { soft };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:>2}. ", global + 1),
                    Style::default().fg(num_fg).bg(bg),
                ),
                Span::styled(
                    format!("{:<width$}", truncate(article.title, title_w), width = title_w),
                    Style::default().fg(title_fg).bg(bg).add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
                Span::styled(
                    format!("  {:<width$}", truncate(article.category, cat_w), width = cat_w),
                    Style::default().fg(cat_fg).bg(bg),
                ),
                Span::styled(
                    format!("  {:<width$} ", truncate(article.summary, summary_w), width = summary_w),
                    Style::default().fg(blurb_fg).bg(bg),
                ),
            ]))
        })
        .collect();
    let list = List::new(items).style(Style::default().bg(panel_bg));
    f.render_widget(list, inner);
}

fn draw_reader(f: &mut Frame, area: Rect, state: &mut WikiState, accent: Color, border: Color) {
    let panel_bg = Color::Rgb(9, 12, 22);
    let bright = Color::Rgb(236, 240, 248);
    let dim = Color::Rgb(138, 146, 168);
    let code = Color::Rgb(255, 200, 80);
    let quote = Color::Rgb(180, 200, 220);

    let article = match ARTICLES.get(state.selected) {
        Some(a) => a,
        None => return,
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(panel_bg))
        .title(Line::from(vec![
            Span::styled(
                format!(" {} ", article.title),
                Style::default().fg(bright).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("[{}] ", article.category), Style::default().fg(dim)),
        ]))
        .title_bottom(Line::from(Span::styled(
            "  Esc back   j/k line   d/u page   g/G top/bot  ",
            Style::default().fg(dim),
        )));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let theme = MdStyle {
        accent,
        body: bright,
        dim,
        code,
        quote,
        bg: panel_bg,
    };
    let lines = markdown::render(article.body, theme);
    let line_count = lines.len() as u16;
    let max_scroll = line_count.saturating_sub(inner.height);
    if state.scroll > max_scroll {
        state.scroll = max_scroll;
    }
    state.last_reader_height = inner.height;
    state.last_reader_lines = line_count;

    let para = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((state.scroll, 0))
        .style(Style::default().bg(panel_bg));
    f.render_widget(para, inner);
}

fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_string();
    }
    if width <= 3 {
        return ".".repeat(width);
    }
    let take = width - 3;
    let mut out: String = s.chars().take(take).collect();
    out.push_str("...");
    out
}
