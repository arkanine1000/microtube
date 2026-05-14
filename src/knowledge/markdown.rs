//! Minimal markdown subset → ratatui `Line`s.
//!
//! Supported:
//! - ATX headers `#`, `##`, `###`
//! - Bold `**x**`, italic `*x*`, inline code `` `x` ``
//! - Bullet lists with `- ` (single level)
//! - Numbered lists like `1. item`
//! - Blockquotes `> `
//! - Fenced code blocks ` ``` `
//! - Horizontal rules `---`
//! - Inline links `[label](target)` rendered as emphasized labels
//! - Blank-line paragraphs
//!
//! Anything fancier than that simply renders as plain text. We control all
//! the source files, so the constraint is easy to honor.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Theme colors used by the markdown renderer. The Knowledge tab passes its
/// accent color in; the rest are tuned to match the rest of the UI.
#[derive(Clone, Copy)]
pub struct MdStyle {
    pub accent: Color,
    pub body: Color,
    pub dim: Color,
    pub code: Color,
    pub quote: Color,
    pub bg: Color,
}

#[derive(Debug, PartialEq, Eq)]
enum Block<'a> {
    H1(&'a str),
    H2(&'a str),
    H3(&'a str),
    Paragraph(Vec<&'a str>),
    Bullet(&'a str),
    Numbered(usize, &'a str),
    Quote(&'a str),
    Code(Vec<&'a str>),
    Rule,
    Blank,
}

fn block_lines(input: &str) -> Vec<Block<'_>> {
    let mut blocks: Vec<Block<'_>> = Vec::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut in_code = false;
    let mut code_buffer: Vec<&str> = Vec::new();

    for raw in input.lines() {
        let line = raw.trim_end_matches('\r');

        if in_code {
            if line.trim_start().starts_with("```") {
                blocks.push(Block::Code(std::mem::take(&mut code_buffer)));
                in_code = false;
            } else {
                code_buffer.push(line);
            }
            continue;
        }

        if line.trim_start().starts_with("```") {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            in_code = true;
            continue;
        }

        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            blocks.push(Block::Blank);
            continue;
        }

        if trimmed == "---" || trimmed == "***" {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            blocks.push(Block::Rule);
        } else if let Some(rest) = trimmed.strip_prefix("### ") {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            blocks.push(Block::H3(rest));
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            blocks.push(Block::H2(rest));
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            blocks.push(Block::H1(rest));
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            blocks.push(Block::Bullet(rest));
        } else if let Some((number, rest)) = numbered_item(trimmed) {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            blocks.push(Block::Numbered(number, rest));
        } else if let Some(rest) = trimmed.strip_prefix("> ") {
            if !paragraph.is_empty() {
                blocks.push(Block::Paragraph(std::mem::take(&mut paragraph)));
            }
            blocks.push(Block::Quote(rest));
        } else {
            paragraph.push(line);
        }
    }

    if !paragraph.is_empty() {
        blocks.push(Block::Paragraph(paragraph));
    }
    if in_code && !code_buffer.is_empty() {
        blocks.push(Block::Code(code_buffer));
    }

    blocks
}

fn numbered_item(line: &str) -> Option<(usize, &str)> {
    let dot = line.find(". ")?;
    if dot == 0 || !line[..dot].bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let number = line[..dot].parse().ok()?;
    Some((number, &line[dot + 2..]))
}

/// Parse and render `input` into ratatui Lines. Each Line carries a `bg`
/// matching the surrounding panel, so `Paragraph` rendering looks seamless.
pub fn render(input: &str, theme: MdStyle) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let blocks = block_lines(input);
    let mut prev_blank = true;

    for block in blocks {
        match block {
            Block::H1(text) => {
                if !prev_blank {
                    lines.push(blank(theme.bg));
                }
                lines.push(Line::from(vec![Span::styled(
                    text.to_string(),
                    Style::default()
                        .fg(theme.accent)
                        .bg(theme.bg)
                        .add_modifier(Modifier::BOLD),
                )]));
                lines.push(Line::from(vec![Span::styled(
                    "\u{2500}".repeat(text.chars().count().min(60)),
                    Style::default().fg(theme.accent).bg(theme.bg),
                )]));
                prev_blank = false;
            }
            Block::H2(text) => {
                if !prev_blank {
                    lines.push(blank(theme.bg));
                }
                lines.push(Line::from(vec![Span::styled(
                    format!("\u{25C7} {text}"),
                    Style::default()
                        .fg(theme.accent)
                        .bg(theme.bg)
                        .add_modifier(Modifier::BOLD),
                )]));
                prev_blank = false;
            }
            Block::H3(text) => {
                lines.push(Line::from(vec![Span::styled(
                    text.to_string(),
                    Style::default()
                        .fg(theme.body)
                        .bg(theme.bg)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )]));
                prev_blank = false;
            }
            Block::Paragraph(rows) => {
                for row in rows {
                    lines.push(inline_line(row, &theme, theme.body, &[]));
                }
                prev_blank = false;
            }
            Block::Bullet(text) => {
                lines.push(inline_line(
                    text,
                    &theme,
                    theme.body,
                    &[Span::styled(
                        "  \u{2022} ",
                        Style::default().fg(theme.accent).bg(theme.bg),
                    )],
                ));
                prev_blank = false;
            }
            Block::Numbered(number, text) => {
                lines.push(inline_line(
                    text,
                    &theme,
                    theme.body,
                    &[Span::styled(
                        format!("  {number}. "),
                        Style::default().fg(theme.accent).bg(theme.bg),
                    )],
                ));
                prev_blank = false;
            }
            Block::Quote(text) => {
                lines.push(inline_line(
                    text,
                    &theme,
                    theme.quote,
                    &[Span::styled(
                        "  \u{2502} ",
                        Style::default().fg(theme.accent).bg(theme.bg),
                    )],
                ));
                prev_blank = false;
            }
            Block::Code(rows) => {
                for row in rows {
                    lines.push(Line::from(vec![
                        Span::styled("    ", Style::default().bg(theme.bg)),
                        Span::styled(
                            row.to_string(),
                            Style::default().fg(theme.code).bg(theme.bg),
                        ),
                    ]));
                }
                prev_blank = false;
            }
            Block::Rule => {
                lines.push(Line::from(vec![Span::styled(
                    "\u{2500}".repeat(60),
                    Style::default().fg(theme.dim).bg(theme.bg),
                )]));
                prev_blank = false;
            }
            Block::Blank => {
                lines.push(blank(theme.bg));
                prev_blank = true;
            }
        }
    }

    lines
}

fn blank(bg: Color) -> Line<'static> {
    Line::from(vec![Span::styled("", Style::default().bg(bg))])
}

fn inline_line(
    text: &str,
    theme: &MdStyle,
    body: Color,
    prefix: &[Span<'static>],
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = prefix.to_vec();
    let body_style = Style::default().fg(body).bg(theme.bg);
    let bold_style = Style::default()
        .fg(theme.accent)
        .bg(theme.bg)
        .add_modifier(Modifier::BOLD);
    let italic_style = Style::default()
        .fg(body)
        .bg(theme.bg)
        .add_modifier(Modifier::ITALIC);
    let code_style = Style::default()
        .fg(theme.code)
        .bg(theme.bg)
        .add_modifier(Modifier::BOLD);
    let link_style = Style::default()
        .fg(theme.accent)
        .bg(theme.bg)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let bytes = text.as_bytes();
    let mut i = 0;
    let mut buf = String::new();

    while i < bytes.len() {
        // Two-char marker: `**`
        if i + 1 < bytes.len() && bytes[i] == b'*' && bytes[i + 1] == b'*' {
            if !buf.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut buf), body_style));
            }
            // Find closing `**`.
            let mut j = i + 2;
            while j + 1 < bytes.len() && !(bytes[j] == b'*' && bytes[j + 1] == b'*') {
                j += 1;
            }
            if j + 1 < bytes.len() {
                let inner = std::str::from_utf8(&bytes[i + 2..j]).unwrap_or("");
                spans.push(Span::styled(inner.to_string(), bold_style));
                i = j + 2;
                continue;
            } else {
                buf.push_str(std::str::from_utf8(&bytes[i..]).unwrap_or(""));
                break;
            }
        }
        // Single-char marker: `*` (italic) or `_`
        if bytes[i] == b'*' || bytes[i] == b'_' {
            let marker = bytes[i];
            if !buf.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut buf), body_style));
            }
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != marker {
                j += 1;
            }
            if j < bytes.len() {
                let inner = std::str::from_utf8(&bytes[i + 1..j]).unwrap_or("");
                spans.push(Span::styled(inner.to_string(), italic_style));
                i = j + 1;
                continue;
            } else {
                buf.push_str(std::str::from_utf8(&bytes[i..]).unwrap_or(""));
                break;
            }
        }
        // Inline code: ` ... `
        if bytes[i] == b'`' {
            if !buf.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut buf), body_style));
            }
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] != b'`' {
                j += 1;
            }
            if j < bytes.len() {
                let inner = std::str::from_utf8(&bytes[i + 1..j]).unwrap_or("");
                spans.push(Span::styled(inner.to_string(), code_style));
                i = j + 1;
                continue;
            } else {
                buf.push_str(std::str::from_utf8(&bytes[i..]).unwrap_or(""));
                break;
            }
        }
        // Inline link: [label](target). The terminal reader does not navigate
        // article links yet, but labels should still read as references.
        if bytes[i] == b'[' {
            if let Some((label, next)) = parse_link(text, i) {
                if !buf.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut buf), body_style));
                }
                spans.push(Span::styled(label, link_style));
                i = next;
                continue;
            }
        }
        // Take a codepoint at a time so we don't slice across a multi-byte char.
        let ch_len = utf8_char_len(bytes[i]);
        if let Ok(s) = std::str::from_utf8(&bytes[i..i + ch_len]) {
            buf.push_str(s);
        }
        i += ch_len;
    }

    if !buf.is_empty() {
        spans.push(Span::styled(buf, body_style));
    }

    Line::from(spans)
}

fn parse_link(text: &str, start: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    let mut close_label = start + 1;
    while close_label < bytes.len() && bytes[close_label] != b']' {
        close_label += 1;
    }
    if close_label + 1 >= bytes.len() || bytes[close_label + 1] != b'(' {
        return None;
    }

    let mut close_target = close_label + 2;
    while close_target < bytes.len() && bytes[close_target] != b')' {
        close_target += 1;
    }
    if close_target >= bytes.len() {
        return None;
    }

    let label = std::str::from_utf8(&bytes[start + 1..close_label])
        .ok()?
        .to_string();
    Some((label, close_target + 1))
}

fn utf8_char_len(byte: u8) -> usize {
    if byte < 0xC0 {
        1 // ASCII or continuation byte (fail safe)
    } else if byte < 0xE0 {
        2
    } else if byte < 0xF0 {
        3
    } else {
        4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme() -> MdStyle {
        MdStyle {
            accent: Color::Rgb(0, 255, 255),
            body: Color::Rgb(200, 200, 200),
            dim: Color::Rgb(120, 120, 120),
            code: Color::Rgb(255, 200, 80),
            quote: Color::Rgb(180, 200, 220),
            bg: Color::Rgb(0, 0, 0),
        }
    }

    #[test]
    fn parses_h1_h2_h3() {
        let blocks = block_lines("# title\n## sub\n### deep\n");
        assert_eq!(
            blocks,
            vec![Block::H1("title"), Block::H2("sub"), Block::H3("deep")]
        );
    }

    #[test]
    fn parses_bullet_list() {
        let blocks = block_lines("- one\n- two\n");
        assert_eq!(blocks, vec![Block::Bullet("one"), Block::Bullet("two")]);
    }

    #[test]
    fn parses_numbered_list_and_rule() {
        let blocks = block_lines("1. one\n2. two\n---\n");
        assert_eq!(
            blocks,
            vec![
                Block::Numbered(1, "one"),
                Block::Numbered(2, "two"),
                Block::Rule
            ]
        );
    }

    #[test]
    fn parses_blockquote() {
        let blocks = block_lines("> quote\n");
        assert_eq!(blocks, vec![Block::Quote("quote")]);
    }

    #[test]
    fn parses_fenced_code() {
        let blocks = block_lines("```\nlet x = 1;\nlet y = 2;\n```\n");
        assert_eq!(blocks, vec![Block::Code(vec!["let x = 1;", "let y = 2;"])]);
    }

    #[test]
    fn collapses_paragraph_lines() {
        let blocks = block_lines("hello\nworld\n\ntwo\n");
        assert_eq!(
            blocks,
            vec![
                Block::Paragraph(vec!["hello", "world"]),
                Block::Blank,
                Block::Paragraph(vec!["two"]),
            ]
        );
    }

    #[test]
    fn renders_inline_bold_italic_code() {
        let lines = render("hello **bold** *ital* `code` [link](target) end", theme());
        assert_eq!(lines.len(), 1);
        // The line is a sequence of styled spans; just check that each piece survived.
        let joined: String = lines[0]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect::<Vec<_>>()
            .concat();
        assert!(joined.contains("hello"));
        assert!(joined.contains("bold"));
        assert!(joined.contains("ital"));
        assert!(joined.contains("code"));
        assert!(joined.contains("link"));
        assert!(!joined.contains("target"));
        assert!(joined.contains("end"));
    }

    #[test]
    fn shipped_articles_render_without_panicking() {
        for article in crate::knowledge::content::ARTICLES {
            let lines = render(article.body, theme());
            assert!(
                !lines.is_empty(),
                "article '{}' produced no lines",
                article.slug
            );
        }
        for article in crate::knowledge::content::MICROTUBE_ARTICLES {
            let lines = render(article.body, theme());
            assert!(
                !lines.is_empty(),
                "microtube article '{}' produced no lines",
                article.slug
            );
        }
    }

    #[test]
    fn unterminated_marker_falls_back_to_plain() {
        // Should not panic, and the content should still appear.
        let lines = render("oops *no close", theme());
        let joined: String = lines[0]
            .spans
            .iter()
            .map(|s| s.content.as_ref())
            .collect::<Vec<_>>()
            .concat();
        assert!(joined.contains("oops"));
        assert!(joined.contains("no close"));
    }
}
