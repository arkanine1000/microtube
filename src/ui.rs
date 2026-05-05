use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

use crate::app::{ActiveParam, App, AppMode, VizMode};
use crate::presets::{PRESETS, SEQUENCES, freq_band_name, freq_color};
use crate::visualization::{
    HarmonicLattice, render_beat_envelope, render_braille_waveform, render_emergence,
    render_harmonic_lattice, render_penrose, render_spectrum_bars,
};

// Color palette tuned for dark backgrounds (~#383c4a)
const DIM: ratatui::style::Color = ratatui::style::Color::Rgb(150, 150, 170);
const BRIGHT: ratatui::style::Color = ratatui::style::Color::Rgb(230, 230, 240);
const BORDER: ratatui::style::Color = ratatui::style::Color::Rgb(100, 105, 130);

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();
    app.frame_count += 1;

    if size.width < 64 || size.height < 22 {
        let msg = Paragraph::new("Terminal too small\nMinimum: 64x22")
            .style(Style::default().fg(ratatui::style::Color::Red));
        f.render_widget(msg, size);
        return;
    }

    let beat_freq = app.params.get_beat_freq();
    let accent = freq_color(beat_freq);

    // Breathing pulse on border
    let elapsed = app.start_time.elapsed().as_secs_f64();
    let breath = (elapsed * 0.4 * std::f64::consts::PI).sin();
    let border_color = dim_color(accent, 0.5 + 0.15 * breath as f32);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(8),    // Visualization
            Constraint::Length(8), // Status panel
            Constraint::Length(3), // Controls
        ])
        .split(size);

    draw_title(f, chunks[0], accent, border_color, app);
    draw_visualization(f, chunks[1], app, accent);
    draw_status(f, chunks[2], app, accent, border_color);
    draw_controls(f, chunks[3], accent, border_color);

    match app.mode {
        AppMode::PresetSelect => draw_preset_menu(f, size, app, accent),
        AppMode::SequenceSelect => draw_sequence_menu(f, size, app),
        AppMode::Help => draw_help(f, size, accent),
        AppMode::Normal => {}
    }
}

fn draw_title(
    f: &mut Frame,
    area: Rect,
    accent: ratatui::style::Color,
    border: ratatui::style::Color,
    app: &App,
) {
    let playing = app
        .params
        .playing
        .load(std::sync::atomic::Ordering::Relaxed);
    let status_icon = if playing { "\u{25B6}" } else { "\u{23F8}" };
    let beat_freq = app.params.get_beat_freq();
    let band = freq_band_name(beat_freq);

    let secs = app.session_elapsed() as u32;
    let timer = format!("{}:{:02}", secs / 60, secs % 60);

    let emergence = app.params.get_emergence();
    let em_indicator = if emergence > 0.01 {
        format!(" \u{2234}{:.0}%", emergence * 100.0)
    } else {
        String::new()
    };

    let title = Line::from(vec![
        Span::styled(" \u{2B22} ", Style::default().fg(accent)),
        Span::styled(
            "MICROTUBE",
            Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" \u{2500}\u{2500}\u{2500} ", Style::default().fg(BORDER)),
        Span::styled(status_icon, Style::default().fg(accent)),
        Span::styled(
            format!(" {band}"),
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("  {beat_freq:.1} Hz"), Style::default().fg(BRIGHT)),
        Span::styled(em_indicator, Style::default().fg(accent)),
        Span::styled(format!("  \u{23F1} {timer}"), Style::default().fg(DIM)),
    ]);

    let viz_label = app.viz_mode.label();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title_bottom(
            Line::from(Span::styled(
                format!(" {viz_label} "),
                Style::default().fg(DIM),
            ))
            .centered(),
        );

    f.render_widget(Paragraph::new(title).block(block), area);
}

fn draw_visualization(f: &mut Frame, area: Rect, app: &mut App, accent: ratatui::style::Color) {
    let (samples_l, samples_r) = if let Ok(buf) = app.viz_buffer.try_lock() {
        buf.read_ordered()
    } else {
        return;
    };

    match app.viz_mode {
        VizMode::Waveform => {
            let viz_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);

            let block_l = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(Span::styled(" L ", Style::default().fg(accent)));
            let inner_l = block_l.inner(viz_chunks[0]);
            f.render_widget(block_l, viz_chunks[0]);

            let block_r = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(Span::styled(" R ", Style::default().fg(accent)));
            let inner_r = block_r.inner(viz_chunks[1]);
            f.render_widget(block_r, viz_chunks[1]);

            let buf = f.buffer_mut();
            render_braille_waveform(buf, inner_l, &samples_l, accent);
            let r_color = shift_color(accent, 30, 0, 30);
            render_braille_waveform(buf, inner_r, &samples_r, r_color);
        }
        VizMode::Spectrum => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(Span::styled(
                    " \u{2261} Spectrum ",
                    Style::default().fg(accent),
                ));
            let inner = block.inner(area);
            f.render_widget(block, area);

            let combined: Vec<f32> = samples_l
                .iter()
                .zip(&samples_r)
                .map(|(l, r)| (l + r) * 0.5)
                .collect();
            let buf = f.buffer_mut();
            render_spectrum_bars(buf, inner, &combined, &mut app.spectrum_bars, accent);
        }
        VizMode::Harmonics => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(Span::styled(
                    " \u{223F} Harmonics ",
                    Style::default().fg(accent),
                ));
            let inner = block.inner(area);
            f.render_widget(block, area);

            let elapsed = app.start_time.elapsed().as_secs_f64();
            let beat_freq = app.params.get_beat_freq() as f64;
            let harmonics = app.params.get_harmonics() as f64;
            let buf = f.buffer_mut();
            render_harmonic_lattice(
                buf,
                inner,
                HarmonicLattice {
                    samples_l: &samples_l,
                    samples_r: &samples_r,
                    elapsed,
                    beat_freq,
                    harmonics,
                    color: accent,
                },
            );
        }
        VizMode::Envelope => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(Span::styled(
                    " \u{223F} Beat Envelope ",
                    Style::default().fg(accent),
                ));
            let inner = block.inner(area);
            f.render_widget(block, area);

            let elapsed = app.start_time.elapsed().as_secs_f64();
            let beat_freq = app.params.get_beat_freq() as f64;
            let buf = f.buffer_mut();
            render_beat_envelope(buf, inner, elapsed, beat_freq, accent);
        }
        VizMode::Penrose => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(Span::styled(
                    " \u{2B22} Penrose ",
                    Style::default().fg(accent),
                ));
            let inner = block.inner(area);
            f.render_widget(block, area);

            let elapsed = app.start_time.elapsed().as_secs_f64();
            let beat_freq = app.params.get_beat_freq() as f64;
            let buf = f.buffer_mut();
            render_penrose(buf, inner, elapsed, beat_freq, accent);
        }
        VizMode::Emergence => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .title(Span::styled(
                    " \u{2234} Emergence ",
                    Style::default().fg(accent),
                ));
            let inner = block.inner(area);
            f.render_widget(block, area);

            let snapshot = if let Ok(snap) = app.emergence_snapshot.try_lock() {
                snap.clone()
            } else {
                return;
            };
            let elapsed = app.start_time.elapsed().as_secs_f64();
            let buf = f.buffer_mut();
            render_emergence(buf, inner, &snapshot, elapsed, accent);
        }
    }
}

fn draw_status(
    f: &mut Frame,
    area: Rect,
    app: &App,
    accent: ratatui::style::Color,
    border: ratatui::style::Color,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let base_freq = app.params.get_base_freq();
    let beat_freq = app.params.get_beat_freq();
    let volume = app.params.get_volume();
    let noise = app.params.get_noise_level();
    let harmonics = app.params.get_harmonics();
    let emergence = app.params.get_emergence();

    let param_style = |p: ActiveParam| -> Style {
        if p == app.active_param {
            Style::default().fg(accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(BRIGHT)
        }
    };

    let arrow = |p: ActiveParam| -> &'static str {
        if p == app.active_param {
            "\u{25B8} "
        } else {
            "  "
        }
    };

    let params_text = vec![
        Line::from(vec![
            Span::raw(arrow(ActiveParam::BaseFreq)),
            Span::styled(
                format!("Base:   {:>6.1} Hz", base_freq),
                param_style(ActiveParam::BaseFreq),
            ),
        ]),
        Line::from(vec![
            Span::raw(arrow(ActiveParam::BeatFreq)),
            Span::styled(
                format!("Beat:   {:>6.1} Hz", beat_freq),
                param_style(ActiveParam::BeatFreq),
            ),
        ]),
        Line::from(vec![
            Span::raw(arrow(ActiveParam::Volume)),
            Span::styled(
                format!("Vol:    {} {:>3.0}%", make_bar(volume, 8), volume * 100.0),
                param_style(ActiveParam::Volume),
            ),
        ]),
        Line::from(vec![
            Span::raw(arrow(ActiveParam::Harmonics)),
            Span::styled(
                format!(
                    "Warmth: {} {:>3.0}%",
                    make_bar(harmonics, 8),
                    harmonics * 100.0
                ),
                param_style(ActiveParam::Harmonics),
            ),
        ]),
        Line::from(vec![
            Span::raw(arrow(ActiveParam::Emergence)),
            Span::styled(
                format!(
                    "Emerge: {} {:>3.0}%",
                    make_bar(emergence, 8),
                    emergence * 100.0
                ),
                param_style(ActiveParam::Emergence),
            ),
        ]),
        Line::from(vec![
            Span::raw(arrow(ActiveParam::NoiseLevel)),
            Span::styled(
                format!("Noise:  {} {:>3.0}%", make_bar(noise, 8), noise * 100.0),
                param_style(ActiveParam::NoiseLevel),
            ),
        ]),
    ];

    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(" Parameters ", Style::default().fg(DIM)));
    f.render_widget(Paragraph::new(params_text).block(left_block), chunks[0]);

    // Right panel
    let preset_name = app
        .current_preset
        .map(|i| PRESETS[i].name)
        .unwrap_or("Custom");

    let (sequence_name, sequence_progress) = if let Some(idx) = app.current_sequence {
        let elapsed = app.sequence_elapsed().unwrap_or(0.0);
        let total = SEQUENCES[idx].total_duration_secs;
        let mins = elapsed as u32 / 60;
        let secs = elapsed as u32 % 60;
        let total_mins = total as u32 / 60;
        let progress = elapsed / total;
        (
            SEQUENCES[idx].name,
            format!(
                "{} {mins}:{secs:02}/{total_mins}:00",
                make_progress_bar(progress, 10),
            ),
        )
    } else {
        ("None", String::new())
    };

    let band = freq_band_name(beat_freq);

    // Breathing indicator
    let breath_elapsed = app.start_time.elapsed().as_secs_f64();
    let breath_cycle = 7.5;
    let breath_phase = (breath_elapsed % breath_cycle) / breath_cycle;
    let breath_char = if breath_phase < 0.4 {
        "\u{2581}\u{2582}\u{2583}\u{2584}\u{2585}\u{2586}\u{2587}"
    } else if breath_phase < 0.5 {
        "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}"
    } else if breath_phase < 0.9 {
        "\u{2587}\u{2586}\u{2585}\u{2584}\u{2583}\u{2582}\u{2581}"
    } else {
        "\u{2581}\u{2581}\u{2581}\u{2581}\u{2581}\u{2581}\u{2581}"
    };

    // Emergence status
    let em_status = if emergence > 0.01 {
        if let Ok(snap) = app.emergence_snapshot.try_lock() {
            format!(
                "{} voices, gen {}",
                snap.voices.len(),
                snap.generation_count
            )
        } else {
            "active".to_string()
        }
    } else {
        "off".to_string()
    };

    let right_text = vec![
        Line::from(vec![
            Span::styled("  Preset:  ", Style::default().fg(DIM)),
            Span::styled(preset_name, Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("  Band:    ", Style::default().fg(DIM)),
            Span::styled(band, Style::default().fg(accent)),
        ]),
        Line::from(vec![
            Span::styled("  Seq:     ", Style::default().fg(DIM)),
            Span::styled(sequence_name, Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("  Prog:    ", Style::default().fg(DIM)),
            Span::styled(sequence_progress, Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("  Emerge:  ", Style::default().fg(DIM)),
            Span::styled(em_status, Style::default().fg(accent)),
        ]),
        Line::from(vec![
            Span::styled("  Breath:  ", Style::default().fg(DIM)),
            Span::styled(breath_char, Style::default().fg(accent)),
        ]),
    ];

    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .title(Span::styled(" Session ", Style::default().fg(DIM)));
    f.render_widget(Paragraph::new(right_text).block(right_block), chunks[1]);
}

fn draw_controls(
    f: &mut Frame,
    area: Rect,
    accent: ratatui::style::Color,
    border: ratatui::style::Color,
) {
    let controls = Line::from(vec![
        Span::styled(" h/l", Style::default().fg(accent)),
        Span::styled(" adj ", Style::default().fg(DIM)),
        Span::styled("j/k", Style::default().fg(accent)),
        Span::styled(" sel ", Style::default().fg(DIM)),
        Span::styled("H/L", Style::default().fg(accent)),
        Span::styled(" big ", Style::default().fg(DIM)),
        Span::styled("p", Style::default().fg(accent)),
        Span::styled("re ", Style::default().fg(DIM)),
        Span::styled("s", Style::default().fg(accent)),
        Span::styled("eq ", Style::default().fg(DIM)),
        Span::styled("v/V", Style::default().fg(accent)),
        Span::styled("iz ", Style::default().fg(DIM)),
        Span::styled("e", Style::default().fg(accent)),
        Span::styled("merge ", Style::default().fg(DIM)),
        Span::styled("n", Style::default().fg(accent)),
        Span::styled("oise ", Style::default().fg(DIM)),
        Span::styled("?", Style::default().fg(accent)),
        Span::styled("help ", Style::default().fg(DIM)),
        Span::styled("q", Style::default().fg(accent)),
        Span::styled("uit", Style::default().fg(DIM)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border));
    f.render_widget(Paragraph::new(controls).centered().block(block), area);
}

fn draw_preset_menu(f: &mut Frame, area: Rect, app: &App, accent: ratatui::style::Color) {
    let menu_area = centered_rect(55, 55, area);
    f.render_widget(Clear, menu_area);

    let items: Vec<ListItem> = PRESETS
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let style = if i == app.menu_index {
                Style::default()
                    .fg(ratatui::style::Color::Rgb(30, 30, 40))
                    .bg(p.color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(p.color)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", i + 1), Style::default().fg(DIM)),
                Span::styled(format!("{:<14}", p.name), style),
                Span::styled(format!(" {}", p.description), Style::default().fg(DIM)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(accent))
            .title(Span::styled(
                " Presets (j/k, Enter, Esc) ",
                Style::default().fg(accent).add_modifier(Modifier::BOLD),
            )),
    );
    f.render_widget(list, menu_area);
}

fn draw_sequence_menu(f: &mut Frame, area: Rect, app: &App) {
    let menu_area = centered_rect(58, 55, area);
    f.render_widget(Clear, menu_area);

    let items: Vec<ListItem> = SEQUENCES
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.menu_index {
                Style::default()
                    .fg(ratatui::style::Color::Rgb(30, 30, 40))
                    .bg(ratatui::style::Color::Rgb(255, 220, 80))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(BRIGHT)
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", i + 1), Style::default().fg(DIM)),
                Span::styled(format!("{:<18}", s.name), style),
                Span::styled(format!(" {}", s.description), Style::default().fg(DIM)),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(ratatui::style::Color::Rgb(255, 220, 80)))
            .title(Span::styled(
                " Sequences (j/k, Enter, Esc) ",
                Style::default()
                    .fg(ratatui::style::Color::Rgb(255, 220, 80))
                    .add_modifier(Modifier::BOLD),
            )),
    );
    f.render_widget(list, menu_area);
}

fn draw_help(f: &mut Frame, area: Rect, accent: ratatui::style::Color) {
    let help_area = centered_rect(62, 80, area);
    f.render_widget(Clear, help_area);

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Navigation",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    j / \u{2193}     ", Style::default().fg(accent)),
            Span::styled("Next parameter", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    k / \u{2191}     ", Style::default().fg(accent)),
            Span::styled("Previous parameter", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    h / \u{2190}     ", Style::default().fg(accent)),
            Span::styled("Decrease value", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    l / \u{2192}     ", Style::default().fg(accent)),
            Span::styled("Increase value", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    H / L       ", Style::default().fg(accent)),
            Span::styled("Big decrease / increase", Style::default().fg(BRIGHT)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Controls",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("    Space       ", Style::default().fg(accent)),
            Span::styled("Play / Pause", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    p           ", Style::default().fg(accent)),
            Span::styled("Preset menu", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    s           ", Style::default().fg(accent)),
            Span::styled("Sequence menu", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    v / V       ", Style::default().fg(accent)),
            Span::styled("Next / previous visualization", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    e           ", Style::default().fg(accent)),
            Span::styled("Toggle emergence mode", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    n           ", Style::default().fg(accent)),
            Span::styled("Toggle noise", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    1-5         ", Style::default().fg(accent)),
            Span::styled("Quick preset select", Style::default().fg(BRIGHT)),
        ]),
        Line::from(vec![
            Span::styled("    q / Esc     ", Style::default().fg(accent)),
            Span::styled("Quit", Style::default().fg(BRIGHT)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Emergence",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "    Voices spawn at harmonic intervals, interact via",
            Style::default().fg(DIM),
        )),
        Line::from(Span::styled(
            "    consonance rules, and decay. A Bach-like canon of",
            Style::default().fg(DIM),
        )),
        Line::from(Span::styled(
            "    living sound. Adjust intensity with the Emerge param.",
            Style::default().fg(DIM),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent))
        .title(Span::styled(
            " Help (press any key) ",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ));
    f.render_widget(Paragraph::new(help_text).block(block), help_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(v[1])[1]
}

fn make_bar(value: f32, width: usize) -> String {
    let filled = (value * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty))
}

fn make_progress_bar(value: f32, width: usize) -> String {
    let filled = (value.clamp(0.0, 1.0) * width as f32) as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "\u{2523}{}\u{2501}{}\u{252B}",
        "\u{2501}".repeat(filled),
        "\u{2500}".repeat(empty)
    )
}

fn dim_color(color: ratatui::style::Color, factor: f32) -> ratatui::style::Color {
    match color {
        ratatui::style::Color::Rgb(r, g, b) => ratatui::style::Color::Rgb(
            (r as f32 * factor) as u8,
            (g as f32 * factor) as u8,
            (b as f32 * factor) as u8,
        ),
        c => c,
    }
}

fn shift_color(color: ratatui::style::Color, dr: u8, dg: u8, db: u8) -> ratatui::style::Color {
    match color {
        ratatui::style::Color::Rgb(r, g, b) => ratatui::style::Color::Rgb(
            r.saturating_add(dr),
            g.saturating_add(dg),
            b.saturating_add(db),
        ),
        c => c,
    }
}
