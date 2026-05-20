//! Hand-rolled flat-key TOML parser sized for the glossary file.
//!
//! Supports exactly:
//!   `[entries.slug]` table headers
//!   `key = "string"`             (with `\"`, `\n`, `\\` escapes)
//!   `key = ["a", "b", "c"]`      (list of strings, single-line)
//!   `# comment`                  (line ignored)
//!
//! Anything fancier panics at parse time. Because the source is
//! `include_str!`'d, malformed input is a build-time-grade bug.
//!
//! Also owns `GlossaryEntry`, the rendered list / detail UI, the sort
//! state, and the cross-reference handler that jumps to a wiki article.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap};

use crate::app::App;
use crate::knowledge::content::GLOSSARY_TOML;

#[derive(Debug, Clone)]
pub struct GlossaryEntry {
    pub slug: String,
    pub term: String,
    pub category: String,
    pub brief: String,
    pub expanded: String,
    pub related: Vec<String>,
    pub wiki_ref: Option<String>,
}

#[derive(Debug)]
pub struct Glossary {
    pub entries: Vec<GlossaryEntry>,
    pub categories: Vec<String>,
}

pub fn glossary() -> &'static Glossary {
    static G: OnceLock<Glossary> = OnceLock::new();
    G.get_or_init(|| parse(GLOSSARY_TOML))
}

fn parse(input: &str) -> Glossary {
    let mut current_table: Option<String> = None;
    let mut tables: BTreeMap<String, BTreeMap<String, Value>> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();

    for (lineno, raw) in input.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix('[') {
            let header = rest.strip_suffix(']').unwrap_or_else(|| {
                panic!("glossary line {}: unterminated table header", lineno + 1)
            });
            let header = header.trim();
            let slug = header.strip_prefix("entries.").unwrap_or_else(|| {
                panic!(
                    "glossary line {}: expected [entries.slug], got [{header}]",
                    lineno + 1
                )
            });
            current_table = Some(slug.to_string());
            tables.entry(slug.to_string()).or_default();
            if !order.contains(&slug.to_string()) {
                order.push(slug.to_string());
            }
            continue;
        }

        let table_slug = current_table.as_ref().unwrap_or_else(|| {
            panic!(
                "glossary line {}: key outside any [entries.*] table",
                lineno + 1
            )
        });

        let eq = line
            .find('=')
            .unwrap_or_else(|| panic!("glossary line {}: missing `=`", lineno + 1));
        let key = line[..eq].trim().to_string();
        let value_text = line[eq + 1..].trim();
        let value = parse_value(value_text, lineno + 1);
        tables.get_mut(table_slug).unwrap().insert(key, value);
    }

    let mut entries: Vec<GlossaryEntry> = Vec::new();
    let mut categories: Vec<String> = Vec::new();

    for slug in &order {
        let fields = &tables[slug];
        let term = string_field(fields, "term", slug);
        let category = string_field(fields, "category", slug);
        let brief = string_field(fields, "brief", slug);
        let expanded = optional_string(fields, "expanded").unwrap_or_default();
        let related = optional_list(fields, "related").unwrap_or_default();
        let wiki_ref = optional_string(fields, "wiki_ref");

        if !categories.iter().any(|c| c == &category) {
            categories.push(category.clone());
        }

        entries.push(GlossaryEntry {
            slug: slug.clone(),
            term,
            category,
            brief,
            expanded,
            related,
            wiki_ref,
        });
    }

    Glossary {
        entries,
        categories,
    }
}

#[derive(Debug)]
enum Value {
    Str(String),
    List(Vec<String>),
}

fn parse_value(raw: &str, lineno: usize) -> Value {
    let raw = raw.trim();
    if let Some(inside) = raw.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let mut items: Vec<String> = Vec::new();
        let bytes = inside.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b',') {
                i += 1;
            }
            if i >= bytes.len() {
                break;
            }
            if bytes[i] != b'"' {
                panic!("glossary line {lineno}: list element not a string: '{raw}'");
            }
            i += 1;
            let mut buf = String::new();
            while i < bytes.len() && bytes[i] != b'"' {
                consume_char(bytes, &mut i, &mut buf, lineno);
            }
            if i >= bytes.len() {
                panic!("glossary line {lineno}: unterminated string in list");
            }
            i += 1;
            items.push(buf);
        }
        Value::List(items)
    } else {
        let inside = raw
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or_else(|| {
                panic!("glossary line {lineno}: expected quoted string, got '{raw}'")
            });
        let mut buf = String::new();
        let bytes = inside.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            consume_char(bytes, &mut i, &mut buf, lineno);
        }
        Value::Str(buf)
    }
}

/// Decode one escape sequence or one UTF-8 codepoint into `buf`. Advances `i`.
fn consume_char(bytes: &[u8], i: &mut usize, buf: &mut String, lineno: usize) {
    if bytes[*i] == b'\\' && *i + 1 < bytes.len() {
        match bytes[*i + 1] {
            b'n' => {
                buf.push('\n');
                *i += 2;
                return;
            }
            b'"' => {
                buf.push('"');
                *i += 2;
                return;
            }
            b'\\' => {
                buf.push('\\');
                *i += 2;
                return;
            }
            b'u' => {
                // Accept either `\u{XXXX}` (Rust style, what we author) or
                // `\uXXXX` (TOML 1.0 style). Both decode to a Unicode scalar.
                let mut j = *i + 2;
                let (start, end);
                if j < bytes.len() && bytes[j] == b'{' {
                    j += 1;
                    start = j;
                    while j < bytes.len() && bytes[j] != b'}' {
                        j += 1;
                    }
                    if j >= bytes.len() {
                        panic!("glossary line {lineno}: unterminated \\u{{...}}");
                    }
                    end = j;
                    j += 1;
                } else {
                    start = j;
                    end = (j + 4).min(bytes.len());
                    j = end;
                }
                let hex = std::str::from_utf8(&bytes[start..end])
                    .unwrap_or_else(|_| panic!("glossary line {lineno}: bad \\u escape"));
                let code = u32::from_str_radix(hex, 16)
                    .unwrap_or_else(|_| panic!("glossary line {lineno}: invalid \\u hex '{hex}'"));
                let ch = char::from_u32(code).unwrap_or_else(|| {
                    panic!("glossary line {lineno}: invalid Unicode codepoint U+{hex}")
                });
                buf.push(ch);
                *i = j;
                return;
            }
            _ => {}
        }
    }
    let ch_len = utf8_char_len(bytes[*i]);
    if let Ok(s) = std::str::from_utf8(&bytes[*i..(*i + ch_len).min(bytes.len())]) {
        buf.push_str(s);
    }
    *i += ch_len;
}

fn utf8_char_len(byte: u8) -> usize {
    if byte < 0xC0 {
        1
    } else if byte < 0xE0 {
        2
    } else if byte < 0xF0 {
        3
    } else {
        4
    }
}

fn string_field(fields: &BTreeMap<String, Value>, name: &str, slug: &str) -> String {
    match fields.get(name) {
        Some(Value::Str(s)) => s.clone(),
        Some(Value::List(_)) => panic!("glossary entry {slug}: '{name}' must be a string"),
        None => panic!("glossary entry {slug}: missing required field '{name}'"),
    }
}

fn optional_string(fields: &BTreeMap<String, Value>, name: &str) -> Option<String> {
    match fields.get(name) {
        Some(Value::Str(s)) => Some(s.clone()),
        _ => None,
    }
}

fn optional_list(fields: &BTreeMap<String, Value>, name: &str) -> Option<Vec<String>> {
    match fields.get(name) {
        Some(Value::List(v)) => Some(v.clone()),
        _ => None,
    }
}

// ─── UI state & rendering ───────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    /// Group by category (insertion order within each category).
    Category,
    /// Sort alphabetically by term, ignoring category.
    Alpha,
}

impl SortMode {
    fn label(self) -> &'static str {
        match self {
            Self::Category => "by category",
            Self::Alpha => "A-Z",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Category => Self::Alpha,
            Self::Alpha => Self::Category,
        }
    }
}

pub struct GlossaryState {
    /// Index into the *visible* (filtered + sorted) entry list.
    pub selected: usize,
    /// `None` shows all categories. `Some(i)` filters to the i-th category.
    pub category_filter: Option<usize>,
    /// `false` shows the list view; `true` shows the detail pane.
    pub expanded: bool,
    pub sort: SortMode,
    /// First row currently scrolled into view in the list mode.
    pub list_scroll: usize,
}

impl GlossaryState {
    pub fn new() -> Self {
        Self {
            selected: 0,
            category_filter: None,
            expanded: false,
            sort: SortMode::Category,
            list_scroll: 0,
        }
    }

    /// Returns the indices into `glossary().entries` that should be shown,
    /// in display order, after applying the category filter and sort mode.
    pub fn visible_indices(&self) -> Vec<usize> {
        let g = glossary();
        let mut idxs: Vec<usize> = match self.category_filter {
            None => (0..g.entries.len()).collect(),
            Some(cat_idx) => {
                let cat = g.categories.get(cat_idx).cloned().unwrap_or_default();
                g.entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.category == cat)
                    .map(|(i, _)| i)
                    .collect()
            }
        };
        match self.sort {
            SortMode::Category => {
                idxs.sort_by(|&a, &b| {
                    let ea = &g.entries[a];
                    let eb = &g.entries[b];
                    let cat_a = g
                        .categories
                        .iter()
                        .position(|c| c == &ea.category)
                        .unwrap_or(usize::MAX);
                    let cat_b = g
                        .categories
                        .iter()
                        .position(|c| c == &eb.category)
                        .unwrap_or(usize::MAX);
                    cat_a
                        .cmp(&cat_b)
                        .then_with(|| ea.term.to_lowercase().cmp(&eb.term.to_lowercase()))
                });
            }
            SortMode::Alpha => {
                idxs.sort_by(|&a, &b| {
                    g.entries[a]
                        .term
                        .to_lowercase()
                        .cmp(&g.entries[b].term.to_lowercase())
                });
            }
        }
        idxs
    }

    pub fn current_entry_index(&self) -> Option<usize> {
        self.visible_indices().get(self.selected).copied()
    }

    fn select_entry_by_slug(&mut self, slug: &str) -> bool {
        let g = glossary();
        let Some(entry_idx) = g.entries.iter().position(|entry| entry.slug == slug) else {
            return false;
        };

        self.category_filter = None;
        let idxs = self.visible_indices();
        let Some(visible_idx) = idxs.iter().position(|&idx| idx == entry_idx) else {
            return false;
        };

        self.selected = visible_idx;
        self.list_scroll = self.list_scroll.min(self.selected);
        true
    }
}

impl Default for GlossaryState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn handle_key(app: &mut App, code: KeyCode) -> bool {
    let g = glossary();
    let idxs = app.knowledge.glossary.visible_indices();
    let len = idxs.len();
    let state = &mut app.knowledge.glossary;

    if state.expanded {
        match code {
            KeyCode::Esc
            | KeyCode::Char('q')
            | KeyCode::Backspace
            | KeyCode::Char('h')
            | KeyCode::Left => {
                state.expanded = false;
            }
            KeyCode::Char('w') | KeyCode::Enter => {
                if let Some(idx) = idxs.get(state.selected).copied()
                    && let Some(slug) = g.entries[idx].wiki_ref.clone()
                {
                    app.knowledge.open_wiki_article(&slug);
                }
            }
            KeyCode::Char(ch) if ('1'..='9').contains(&ch) => {
                let related_idx = ch as usize - '1' as usize;
                if let Some(idx) = idxs.get(state.selected).copied()
                    && let Some(slug) = g.entries[idx].related.get(related_idx).cloned()
                {
                    state.select_entry_by_slug(&slug);
                }
            }
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
            _ => {}
        }
        return true;
    }

    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            if len > 0 && state.selected + 1 < len {
                state.selected += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if state.selected > 0 {
                state.selected -= 1;
            }
        }
        KeyCode::Char('d') | KeyCode::PageDown => {
            state.selected = (state.selected + 10).min(len.saturating_sub(1));
        }
        KeyCode::Char('u') | KeyCode::PageUp => {
            state.selected = state.selected.saturating_sub(10);
        }
        KeyCode::Char('g') | KeyCode::Home => {
            state.selected = 0;
        }
        KeyCode::Char('G') | KeyCode::End => {
            if len > 0 {
                state.selected = len - 1;
            }
        }
        KeyCode::Char('c') => {
            // Cycle category filter: All -> cat0 -> cat1 -> ... -> All.
            let n = g.categories.len();
            state.category_filter = match state.category_filter {
                None if n > 0 => Some(0),
                Some(i) if i + 1 < n => Some(i + 1),
                _ => None,
            };
            state.selected = 0;
        }
        KeyCode::Char('s') => {
            state.sort = state.sort.next();
            state.selected = 0;
        }
        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => {
            if len > 0 {
                state.expanded = true;
            }
        }
        _ => return false,
    }
    true
}

pub fn draw(f: &mut Frame, area: Rect, state: &mut GlossaryState, accent: Color, border: Color) {
    let panel_bg = Color::Rgb(9, 12, 22);
    let dim = Color::Rgb(138, 146, 168);
    let bright = Color::Rgb(236, 240, 248);

    let g = glossary();
    let idxs = state.visible_indices();
    if state.selected >= idxs.len() && !idxs.is_empty() {
        state.selected = idxs.len() - 1;
    }

    let cat_label = match state.category_filter {
        None => "all".to_string(),
        Some(i) => g.categories.get(i).cloned().unwrap_or_else(|| "?".into()),
    };
    let title = format!(
        " GLOSSARY  [c] {cat_label}  [s] {}  {}/{} ",
        state.sort.label(),
        if idxs.is_empty() {
            0
        } else {
            state.selected + 1
        },
        idxs.len()
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
            "  j/k move   d/u page   g/G top/bot   c category   s sort   Enter open  ",
            Style::default().fg(dim),
        )));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if state.expanded {
        draw_detail(f, inner, state, accent);
    } else {
        draw_list(f, inner, state, &idxs, accent);
    }
}

fn draw_list(f: &mut Frame, area: Rect, state: &mut GlossaryState, idxs: &[usize], accent: Color) {
    let panel_bg = Color::Rgb(9, 12, 22);
    let bright = Color::Rgb(236, 240, 248);
    let soft = Color::Rgb(178, 184, 204);
    let dim = Color::Rgb(138, 146, 168);
    let select_fg = Color::Rgb(5, 7, 14);

    let g = glossary();
    let height = area.height as usize;

    // Keep the selected row in view by sliding the scroll window.
    if state.selected < state.list_scroll {
        state.list_scroll = state.selected;
    } else if state.selected >= state.list_scroll + height && height > 0 {
        state.list_scroll = state.selected + 1 - height;
    }
    if state.list_scroll + height > idxs.len() {
        state.list_scroll = idxs.len().saturating_sub(height);
    }

    // Column widths: term takes a fixed slot, category a fixed slot, brief
    // gets whatever is left. Truncate with an ellipsis if longer.
    let total = area.width as usize;
    let term_w = 24;
    let cat_w = 14;
    let gutter = 2;
    let brief_w = total.saturating_sub(2 + term_w + cat_w + gutter * 2);

    let visible_end = (state.list_scroll + height).min(idxs.len());
    let items: Vec<ListItem> = idxs[state.list_scroll..visible_end]
        .iter()
        .enumerate()
        .map(|(local, &full_idx)| {
            let global = state.list_scroll + local;
            let is_selected = global == state.selected;
            let bg = if is_selected { accent } else { panel_bg };
            let term_fg = if is_selected { select_fg } else { bright };
            let cat_fg = if is_selected { select_fg } else { dim };
            let brief_fg = if is_selected { select_fg } else { soft };
            let entry = &g.entries[full_idx];
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {:<width$}", truncate(&entry.term, term_w), width = term_w),
                    Style::default()
                        .fg(term_fg)
                        .bg(bg)
                        .add_modifier(if is_selected {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
                Span::styled(
                    format!(
                        "  {:<width$}",
                        truncate(&entry.category, cat_w),
                        width = cat_w
                    ),
                    Style::default().fg(cat_fg).bg(bg),
                ),
                Span::styled(
                    format!(
                        "  {:<width$} ",
                        truncate(&entry.brief, brief_w),
                        width = brief_w
                    ),
                    Style::default().fg(brief_fg).bg(bg),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(panel_bg));
    f.render_widget(list, area);
}

fn draw_detail(f: &mut Frame, area: Rect, state: &GlossaryState, accent: Color) {
    let panel_bg = Color::Rgb(9, 12, 22);
    let dim = Color::Rgb(138, 146, 168);
    let bright = Color::Rgb(236, 240, 248);
    let soft = Color::Rgb(178, 184, 204);

    let g = glossary();
    let entry_idx = match state.current_entry_index() {
        Some(i) => i,
        None => return,
    };
    let entry = &g.entries[entry_idx];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area);

    let header = vec![
        Line::from(vec![
            Span::styled(
                format!("  {}  ", entry.term),
                Style::default()
                    .fg(accent)
                    .bg(panel_bg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("[{}]", entry.category),
                Style::default().fg(dim).bg(panel_bg),
            ),
        ]),
        Line::from(""),
    ];
    f.render_widget(
        Paragraph::new(header).style(Style::default().bg(panel_bg)),
        chunks[0],
    );

    let mut body: Vec<Line<'static>> = Vec::new();
    body.push(Line::from(vec![Span::styled(
        entry.brief.clone(),
        Style::default().fg(bright).bg(panel_bg),
    )]));
    body.push(Line::from(""));
    if !entry.expanded.is_empty() {
        for paragraph in entry.expanded.split("\n\n") {
            body.push(Line::from(vec![Span::styled(
                paragraph.replace('\n', " "),
                Style::default().fg(soft).bg(panel_bg),
            )]));
            body.push(Line::from(""));
        }
    }
    if !entry.related.is_empty() {
        body.push(Line::from(vec![Span::styled(
            "Related:",
            Style::default().fg(dim).bg(panel_bg),
        )]));
        for (idx, r) in entry.related.iter().enumerate() {
            let label = g
                .entries
                .iter()
                .find(|candidate| candidate.slug == *r)
                .map(|candidate| candidate.term.as_str())
                .unwrap_or(r);
            body.push(Line::from(vec![
                Span::styled(
                    format!("  [{}] ", idx + 1),
                    Style::default()
                        .fg(accent)
                        .bg(panel_bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(label.to_string(), Style::default().fg(soft).bg(panel_bg)),
            ]));
        }
    }
    f.render_widget(
        Paragraph::new(body)
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(panel_bg)),
        chunks[1],
    );

    let mut hints: Vec<Span<'static>> = Vec::new();
    hints.push(Span::styled(
        "  Esc back   ",
        Style::default().fg(dim).bg(panel_bg),
    ));
    if entry.wiki_ref.is_some() {
        hints.push(Span::styled(
            "[w] read article   ",
            Style::default()
                .fg(accent)
                .bg(panel_bg)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if !entry.related.is_empty() {
        hints.push(Span::styled(
            "[1-9] related term",
            Style::default()
                .fg(accent)
                .bg(panel_bg)
                .add_modifier(Modifier::BOLD),
        ));
    }
    f.render_widget(
        Paragraph::new(Line::from(hints)).style(Style::default().bg(panel_bg)),
        chunks[2],
    );
}

/// Truncate `s` to fit within `width` columns, with a `...` suffix when
/// it has to be shortened. ASCII ellipsis avoids the U+2026 codepoint,
/// which is missing or width-ambiguous in some terminal fonts.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_glossary_parses() {
        let g = glossary();
        assert!(!g.entries.is_empty(), "glossary has no entries");
        assert!(!g.categories.is_empty(), "glossary has no categories");
        assert!(g.entries.iter().any(|e| e.slug == "binaural_beat"));
        assert!(g.entries.iter().any(|e| e.slug == "fibonacci_word"));
        for entry in &g.entries {
            if let Some(slug) = &entry.wiki_ref {
                assert!(
                    crate::knowledge::content::ARTICLES
                        .iter()
                        .any(|a| a.slug == slug),
                    "glossary entry '{}' wiki_ref '{}' points at no article",
                    entry.slug,
                    slug,
                );
            }
            for related in &entry.related {
                assert!(
                    g.entries.iter().any(|candidate| candidate.slug == *related),
                    "glossary entry '{}' related term '{}' points at no entry",
                    entry.slug,
                    related,
                );
            }
        }
    }

    #[test]
    fn can_select_related_entry_by_slug() {
        let mut state = GlossaryState::new();
        assert!(state.select_entry_by_slug("alpha"));

        let idx = state.current_entry_index().expect("selected entry");
        assert_eq!(glossary().entries[idx].slug, "alpha");
        assert_eq!(state.category_filter, None);
    }

    #[test]
    fn shipped_glossary_has_no_unicode_escape_leakage() {
        // The hand-rolled parser must turn `\u{XXXX}` and `\uXXXX` into
        // real codepoints. If escapes survive into rendered fields the
        // user sees "u{2013}" in the UI.
        let g = glossary();
        for entry in &g.entries {
            for (label, field) in [
                ("term", &entry.term),
                ("brief", &entry.brief),
                ("expanded", &entry.expanded),
            ] {
                assert!(
                    !field.contains("\\u"),
                    "entry '{}' field '{label}' still contains \\u: {field:?}",
                    entry.slug
                );
                assert!(
                    !field.contains("u{"),
                    "entry '{}' field '{label}' contains literal 'u{{...}}': {field:?}",
                    entry.slug
                );
            }
        }
    }

    #[test]
    fn parses_string_with_escape() {
        let toml = r#"
[entries.x]
term = "Hello \"World\""
category = "Audio"
brief = "Line A\nLine B"
"#;
        let g = parse(toml);
        assert_eq!(g.entries[0].term, r#"Hello "World""#);
        assert_eq!(g.entries[0].brief, "Line A\nLine B");
    }

    #[test]
    fn parses_unicode_escape_braced() {
        let toml = r#"
[entries.x]
term = "phi \u{03C6}"
category = "Math"
brief = "test"
"#;
        let g = parse(toml);
        assert_eq!(g.entries[0].term, "phi \u{03C6}");
    }

    #[test]
    fn parses_unicode_escape_toml_form() {
        let toml = "
[entries.x]
term = \"em-dash \\u2014\"
category = \"Math\"
brief = \"test\"
";
        let g = parse(toml);
        assert_eq!(g.entries[0].term, "em-dash \u{2014}");
    }

    #[test]
    fn parses_list_value() {
        let toml = r#"
[entries.x]
term = "X"
category = "Audio"
brief = "x"
related = ["a", "b", "c"]
"#;
        let g = parse(toml);
        assert_eq!(g.entries[0].related, vec!["a", "b", "c"]);
    }

    #[test]
    fn truncate_shortens_long_strings() {
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("short", 10), "short");
        // Total width preserved when truncated.
        assert_eq!(truncate("abcdefghij", 6).chars().count(), 6);
    }
}
