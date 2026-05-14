use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap};

use crate::app::{ActiveParam, App, AppMode, Tab, VizMode};
use crate::knowledge;
use crate::presets::{PRESETS, SEQUENCES, freq_band_name, freq_color};
use crate::shepard::{MAX_BASE_FREQ_HZ, MIN_BASE_FREQ_HZ};
use crate::visualization::{
    HARMONIC_PARTIAL_LABELS, HarmonicLattice, harmonic_partial_levels, render_beat_envelope,
    render_braille_waveform, render_emergence, render_harmonic_lattice, render_penrose,
    render_spectrum_bars,
};

const BG_TOP: Color = Color::Rgb(18, 20, 23);
const BG_MID: Color = Color::Rgb(24, 27, 32);
const BG_BOTTOM: Color = Color::Rgb(16, 18, 21);
const PANEL_BG: Color = Color::Rgb(22, 24, 28);
const PANEL_ALT: Color = Color::Rgb(28, 31, 36);
const DIM: Color = Color::Rgb(138, 146, 168);
const SOFT: Color = Color::Rgb(178, 184, 204);
const BRIGHT: Color = Color::Rgb(236, 240, 248);
const BORDER: Color = Color::Rgb(88, 98, 128);
const SHADOW: Color = Color::Rgb(10, 11, 13);

struct MeterSpec {
    param: ActiveParam,
    label: &'static str,
    value: String,
    ratio: f32,
    color: Color,
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();
    app.frame_count += 1;

    if size.width < 76 || size.height < 30 {
        let msg = Paragraph::new("Terminal too small\nMinimum: 76x30")
            .style(Style::default().fg(Color::Red).bg(BG_TOP))
            .alignment(Alignment::Center);
        draw_backdrop(
            f.buffer_mut(),
            size,
            app.start_time.elapsed().as_secs_f64(),
            Color::Red,
        );
        f.render_widget(msg, size);
        return;
    }

    let elapsed = app.start_time.elapsed().as_secs_f64();
    let beat_freq = app.params.get_beat_freq();
    let accent = freq_color(beat_freq);
    let border_color = breathing_color(accent, elapsed);

    draw_backdrop(f.buffer_mut(), size, elapsed, accent);
    let shell = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG_TOP))
        .title(Line::from(Span::styled(
            " MICROTUBE ",
            Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD),
        )));
    let shell_inner = shell.inner(size);
    f.render_widget(shell, size);

    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(8)])
        .split(shell_inner);

    draw_tab_strip(f, outer_chunks[0], app.tab, accent);

    match app.tab {
        Tab::Studio => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Min(9),
                    Constraint::Length(10),
                    Constraint::Length(3),
                ])
                .split(outer_chunks[1]);

            draw_header(f, chunks[0], app, accent);
            draw_visualization(f, chunks[1], app, accent, border_color);
            draw_status(f, chunks[2], app, accent, border_color);
            draw_controls(f, chunks[3], accent, border_color);

            match app.mode {
                AppMode::PresetSelect => draw_preset_menu(f, size, app, accent),
                AppMode::SequenceSelect => draw_sequence_menu(f, size, app, accent),
                AppMode::Help => draw_help(f, size, accent),
                AppMode::Normal => {}
            }
        }
        Tab::Knowledge => {
            knowledge::draw(f, outer_chunks[1], app, accent, border_color);
        }
    }
}

fn draw_tab_strip(f: &mut Frame, area: Rect, active: Tab, accent: Color) {
    let tabs = [Tab::Studio, Tab::Knowledge];
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(tabs.len() * 3 + 2);
    spans.push(Span::styled("  ", Style::default().bg(BG_TOP)));
    for tab in tabs {
        let is_active = tab == active;
        let style = if is_active {
            Style::default()
                .fg(BG_TOP)
                .bg(accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM).bg(BG_TOP)
        };
        spans.push(Span::styled(format!("  {}  ", tab.label()), style));
        spans.push(Span::styled("  ", Style::default().bg(BG_TOP)));
    }
    spans.push(Span::styled(
        "Tab to switch    Q to quit",
        Style::default().fg(DIM).bg(BG_TOP),
    ));

    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(BG_TOP)),
        area,
    );
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    let playing = app
        .params
        .playing
        .load(std::sync::atomic::Ordering::Relaxed);
    let status = if playing { "PLAY" } else { "PAUSE" };
    let status_icon = if playing { "\u{25B6}" } else { "\u{23F8}" };
    let beat_freq = app.params.get_beat_freq();
    let base_freq = app.params.get_base_freq();
    let band = freq_band_name(beat_freq);
    let right_freq = base_freq + beat_freq;
    let secs = app.session_elapsed() as u32;
    let timer = format!("{:02}:{:02}", secs / 60, secs % 60);
    let preset = app
        .current_preset
        .map(|idx| PRESETS[idx].name)
        .unwrap_or("Custom");

    let lines = vec![
        Line::from(vec![
            Span::styled("  \u{25C8} ", Style::default().fg(accent)),
            Span::styled(
                "M I C R O T U B E",
                Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  \u{2500}\u{2500}  ", Style::default().fg(BORDER)),
            Span::styled(status_icon, Style::default().fg(accent)),
            Span::styled(
                format!(" {status} "),
                Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{band} "), Style::default().fg(accent)),
            Span::styled(format!("{beat_freq:.1}Hz"), Style::default().fg(BRIGHT)),
        ])
        .centered(),
        Line::from(vec![
            Span::styled(" L ", Style::default().fg(DIM)),
            Span::styled(format!("{base_freq:.1}Hz"), Style::default().fg(BRIGHT)),
            Span::styled("   R ", Style::default().fg(DIM)),
            Span::styled(format!("{right_freq:.1}Hz"), Style::default().fg(BRIGHT)),
            Span::styled("   preset ", Style::default().fg(DIM)),
            Span::styled(preset, Style::default().fg(accent)),
            Span::styled("   viz ", Style::default().fg(DIM)),
            Span::styled(app.viz_mode.label(), Style::default().fg(BRIGHT)),
            Span::styled("   time ", Style::default().fg(DIM)),
            Span::styled(timer, Style::default().fg(BRIGHT)),
        ])
        .centered(),
        spectral_ribbon(beat_freq),
        Line::from(Span::styled(
            make_phase_wave(app.session_elapsed(), area.width as usize / 3),
            Style::default().fg(dim_color(accent, 0.62)),
        ))
        .centered(),
    ];

    f.render_widget(
        Paragraph::new(lines).style(Style::default().bg(BG_TOP)),
        area,
    );
}

fn draw_visualization(f: &mut Frame, area: Rect, app: &mut App, accent: Color, border: Color) {
    let (samples_l, samples_r) = if let Ok(buf) = app.viz_buffer.try_lock() {
        buf.read_ordered()
    } else {
        return;
    };

    let title = Line::from(vec![
        Span::styled(" \u{25C7} ", Style::default().fg(accent)),
        Span::styled(
            format!("{} ", app.viz_mode.label()),
            Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD),
        ),
        Span::styled("audiovisual plane ", Style::default().fg(DIM)),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(PANEL_BG))
        .title(title)
        .title_bottom(mode_rail(app.viz_mode, accent));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let elapsed = app.start_time.elapsed().as_secs_f64();
    draw_visual_backdrop(f.buffer_mut(), inner, elapsed, accent);

    match app.viz_mode {
        VizMode::Waveform => {
            render_waveform_stage(f.buffer_mut(), inner, &samples_l, &samples_r, accent)
        }
        VizMode::Spectrum => {
            let combined: Vec<f32> = samples_l
                .iter()
                .zip(&samples_r)
                .map(|(left, right)| (left + right) * 0.5)
                .collect();
            render_spectrum_bars(
                f.buffer_mut(),
                inner,
                &combined,
                &mut app.spectrum_bars,
                accent,
            );
            draw_spectrum_floor(f.buffer_mut(), inner, accent);
        }
        VizMode::Harmonics => {
            let beat_freq = app.params.get_beat_freq() as f64;
            let harmonics = app.params.get_harmonics() as f64;
            let harmonic_weights = app.params.get_timbre().weights();
            render_harmonic_lattice(
                f.buffer_mut(),
                inner,
                HarmonicLattice {
                    samples_l: &samples_l,
                    samples_r: &samples_r,
                    elapsed,
                    beat_freq,
                    harmonics,
                    harmonic_weights,
                    color: accent,
                },
            );
        }
        VizMode::Envelope => {
            let beat_freq = app.params.get_beat_freq() as f64;
            render_beat_envelope(f.buffer_mut(), inner, elapsed, beat_freq, accent);
        }
        VizMode::Penrose => {
            let beat_freq = app.params.get_beat_freq() as f64;
            render_penrose(f.buffer_mut(), inner, elapsed, beat_freq, accent);
        }
        VizMode::Emergence => {
            let snapshot = if let Ok(snap) = app.emergence_snapshot.try_lock() {
                snap.clone()
            } else {
                return;
            };
            render_emergence(f.buffer_mut(), inner, &snapshot, elapsed, accent);
        }
    }
}

fn render_waveform_stage(
    buf: &mut Buffer,
    area: Rect,
    samples_l: &[f32],
    samples_r: &[f32],
    accent: Color,
) {
    let halves = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    let mid_y = halves[0].y + halves[0].height.saturating_sub(1);
    for x in area.x..area.x + area.width {
        let cell = &mut buf[(x, mid_y)];
        cell.set_char('\u{2500}');
        cell.set_fg(dim_color(accent, 0.34));
        cell.set_bg(PANEL_BG);
    }

    let left_area = Rect {
        y: halves[0].y + 1,
        height: halves[0].height.saturating_sub(1),
        ..halves[0]
    };
    let right_area = Rect {
        y: halves[1].y,
        height: halves[1].height.saturating_sub(1),
        ..halves[1]
    };
    render_braille_waveform(buf, left_area, samples_l, accent);
    render_braille_waveform(buf, right_area, samples_r, shift_color(accent, 34, 8, 44));

    write_str(
        buf,
        area,
        2,
        0,
        "LEFT EAR",
        Style::default().fg(dim_color(accent, 0.82)).bg(PANEL_BG),
    );
    write_str(
        buf,
        area,
        2,
        halves[0].height,
        "RIGHT EAR",
        Style::default()
            .fg(shift_color(accent, 34, 8, 44))
            .bg(PANEL_BG),
    );
}

fn draw_status(f: &mut Frame, area: Rect, app: &App, accent: Color, border: Color) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(47),
            Constraint::Percentage(33),
            Constraint::Percentage(20),
        ])
        .split(area);

    draw_parameter_panel(f, chunks[0], app, accent, border);
    draw_session_panel(f, chunks[1], app, accent, border);
    draw_harmonic_panel(f, chunks[2], app, accent, border);
}

fn draw_parameter_panel(f: &mut Frame, area: Rect, app: &App, accent: Color, border: Color) {
    let block = panel_block(" PARAMETERS ", border);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let base = app.params.get_base_freq();
    let beat = app.params.get_beat_freq();
    let shepard_base = app.params.get_shepard_base_freq();
    let specs = [
        MeterSpec {
            param: ActiveParam::BaseFreq,
            label: "base",
            value: format!("{base:>5.1} Hz"),
            ratio: (base - 50.0) / 450.0,
            color: Color::Rgb(80, 220, 245),
        },
        MeterSpec {
            param: ActiveParam::BeatFreq,
            label: "beat",
            value: format!("{beat:>5.1} Hz"),
            ratio: beat / 100.0,
            color: accent,
        },
        MeterSpec {
            param: ActiveParam::Volume,
            label: "gain",
            value: format!("{:>3.0}%", app.params.get_volume() * 100.0),
            ratio: app.params.get_volume(),
            color: Color::Rgb(84, 240, 150),
        },
        MeterSpec {
            param: ActiveParam::Harmonics,
            label: "warm",
            value: format!("{:>3.0}%", app.params.get_harmonics() * 100.0),
            ratio: app.params.get_harmonics(),
            color: Color::Rgb(250, 210, 92),
        },
        MeterSpec {
            param: ActiveParam::Emergence,
            label: "life",
            value: format!("{:>3.0}%", app.params.get_emergence() * 100.0),
            ratio: app.params.get_emergence(),
            color: Color::Rgb(210, 145, 255),
        },
        MeterSpec {
            param: ActiveParam::Shepard,
            label: "drift",
            value: format!(
                "{} {:>3.0}%",
                app.params.get_shepard_direction().glyph(),
                app.params.get_shepard() * 100.0
            ),
            ratio: app.params.get_shepard(),
            color: Color::Rgb(255, 170, 110),
        },
        MeterSpec {
            param: ActiveParam::ShepardBase,
            label: "dbase",
            value: format!("{shepard_base:>5.1} Hz"),
            ratio: shepard_base_ratio(shepard_base),
            color: Color::Rgb(255, 205, 135),
        },
        MeterSpec {
            param: ActiveParam::NoiseLevel,
            label: "mist",
            value: format!("{:>3.0}%", app.params.get_noise_level() * 100.0),
            ratio: app.params.get_noise_level(),
            color: Color::Rgb(120, 170, 255),
        },
    ];

    let meter_width = inner.width.saturating_sub(24).clamp(5, 18) as usize;
    let rows: Vec<Line> = specs
        .into_iter()
        .map(|spec| meter_line(spec, app.active_param, meter_width))
        .collect();
    f.render_widget(
        Paragraph::new(rows).style(Style::default().bg(PANEL_BG)),
        inner,
    );
}

fn draw_session_panel(f: &mut Frame, area: Rect, app: &App, accent: Color, border: Color) {
    let block = panel_block(" SESSION ", border);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let beat = app.params.get_beat_freq();
    let epoch_or_preset = if let Some(name) = app.current_step_name() {
        data_line("epoch", name, accent)
    } else {
        let preset = app
            .current_preset
            .map(|idx| PRESETS[idx].name)
            .unwrap_or("Custom");
        data_line("preset", preset, accent)
    };
    let band = freq_band_name(beat);
    let seconds = app.session_elapsed() as u32;
    let timer = format!("{:02}:{:02}", seconds / 60, seconds % 60);
    let sequence = sequence_status(app);
    let breath = breath_meter(app.session_elapsed() as f64);
    let emergence = app.params.get_emergence();
    let mode_label = app.params.get_spawn_mode().label();
    let em_status = if emergence > 0.01 {
        if let Ok(snap) = app.emergence_snapshot.try_lock() {
            format!(
                "{} voices / gen {} / {}",
                snap.voices.len(),
                snap.generation_count,
                mode_label
            )
        } else {
            format!("active / {mode_label}")
        }
    } else {
        format!("quiet / {mode_label}")
    };

    let mist = format!(
        "{} / {}",
        app.params.get_mist_type().label(),
        app.params.get_mist_type().texture()
    );
    let rows = vec![
        epoch_or_preset,
        data_line("band", band, accent),
        data_line("visual", app.viz_mode.label(), BRIGHT),
        data_line("time", &timer, BRIGHT),
        data_line("seq", &sequence, SOFT),
        data_line("mist", &mist, Color::Rgb(120, 170, 255)),
        Line::from(vec![
            Span::styled(" life   ", Style::default().fg(DIM).bg(PANEL_BG)),
            Span::styled(em_status, Style::default().fg(accent).bg(PANEL_BG)),
            Span::styled("  ", Style::default().bg(PANEL_BG)),
            Span::styled(breath, Style::default().fg(accent).bg(PANEL_BG)),
        ]),
    ];
    f.render_widget(
        Paragraph::new(rows).style(Style::default().bg(PANEL_BG)),
        inner,
    );
}

fn draw_harmonic_panel(f: &mut Frame, area: Rect, app: &App, accent: Color, border: Color) {
    let block = panel_block(" PARTIALS ", border);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let harmonics = app.params.get_harmonics();
    let timbre = app.params.get_timbre();
    let levels = harmonic_partial_levels(harmonics as f64, timbre.weights());
    let max_level = levels
        .iter()
        .copied()
        .fold(0.0_f64, f64::max)
        .max(f64::EPSILON);
    let bar_width = inner.width.saturating_sub(10).clamp(2, 8) as usize;

    let mut rows = Vec::with_capacity(7);
    for (idx, (id, label)) in HARMONIC_PARTIAL_LABELS.iter().enumerate() {
        let relative = (levels[idx] / max_level).clamp(0.0, 1.0) as f32;
        let glow = 0.28 + relative * 0.72;
        let filled = (relative * bar_width as f32).round() as usize;
        let bar = format!(
            "{}{}",
            "\u{25B0}".repeat(filled),
            "\u{25B1}".repeat(bar_width.saturating_sub(filled))
        );
        rows.push(Line::from(vec![
            Span::styled(
                format!(" {id:<2} "),
                Style::default()
                    .fg(dim_color(accent, glow + 0.14))
                    .bg(PANEL_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("{label:<5}"), Style::default().fg(DIM).bg(PANEL_BG)),
            Span::styled(
                bar,
                Style::default().fg(dim_color(accent, glow)).bg(PANEL_BG),
            ),
        ]));
    }
    rows.push(Line::from(vec![
        Span::styled(" timbre ", Style::default().fg(DIM).bg(PANEL_BG)),
        Span::styled(timbre.label(), Style::default().fg(accent).bg(PANEL_BG)),
    ]));
    f.render_widget(
        Paragraph::new(rows).style(Style::default().bg(PANEL_BG)),
        inner,
    );
}

fn draw_controls(f: &mut Frame, area: Rect, accent: Color, border: Color) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(PANEL_ALT));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let controls = if inner.width < 144 {
        Line::from(vec![
            key_chip("h/l", accent),
            key_chip("j/k", Color::Rgb(84, 240, 150)),
            key_chip("H/L", Color::Rgb(250, 210, 92)),
            key_chip("p", Color::Rgb(120, 170, 255)),
            key_chip("s", Color::Rgb(210, 145, 255)),
            key_chip("v/V", accent),
            key_chip("e", Color::Rgb(210, 145, 255)),
            key_chip("g", Color::Rgb(250, 210, 92)),
            key_chip("r/R", Color::Rgb(255, 170, 110)),
            key_chip("n", Color::Rgb(120, 170, 255)),
            key_chip("m", Color::Rgb(120, 170, 255)),
            key_chip("?", BRIGHT),
            key_chip("q", DIM),
        ])
    } else {
        Line::from(vec![
            command_chip("h/l", "tune", accent),
            command_chip("j/k", "select", Color::Rgb(84, 240, 150)),
            command_chip("H/L", "coarse", Color::Rgb(250, 210, 92)),
            command_chip("p", "preset", Color::Rgb(120, 170, 255)),
            command_chip("s", "sequence", Color::Rgb(210, 145, 255)),
            command_chip("v/V", "visual", accent),
            command_chip("e", "life", Color::Rgb(210, 145, 255)),
            command_chip("g", "geom", Color::Rgb(250, 210, 92)),
            command_chip("r/R", "drift", Color::Rgb(255, 170, 110)),
            command_chip("n", "noise", Color::Rgb(120, 170, 255)),
            command_chip("m", "mist", Color::Rgb(120, 170, 255)),
            command_chip("t", "timbre", Color::Rgb(170, 255, 120)),
            command_chip("?", "help", BRIGHT),
            command_chip("q", "quit", DIM),
        ])
    };
    f.render_widget(
        Paragraph::new(controls)
            .centered()
            .style(Style::default().bg(PANEL_ALT)),
        inner,
    );
}

fn draw_preset_menu(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    f.render_widget(Clear, area);
    draw_backdrop(f.buffer_mut(), area, app.session_elapsed() as f64, accent);
    let menu_area = centered_rect(68, 56, area);
    draw_modal_shadow(f.buffer_mut(), menu_area);

    let items: Vec<ListItem> = PRESETS
        .iter()
        .enumerate()
        .map(|(idx, preset)| {
            let selected = idx == app.menu_index;
            let fg = if selected { BG_TOP } else { preset.color };
            let bg = if selected { preset.color } else { PANEL_BG };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", idx + 1),
                    Style::default()
                        .fg(if selected { BG_TOP } else { DIM })
                        .bg(bg),
                ),
                Span::styled(
                    format!("{:<14}", preset.name),
                    Style::default().fg(fg).bg(bg).add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
                Span::styled(
                    format!("  {}", preset.description),
                    Style::default()
                        .fg(if selected { BG_TOP } else { SOFT })
                        .bg(bg),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(modal_block(" PRESETS  j/k Enter Esc ", accent));
    f.render_widget(list, menu_area);
}

fn draw_sequence_menu(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    f.render_widget(Clear, area);
    draw_backdrop(f.buffer_mut(), area, app.session_elapsed() as f64, accent);
    let menu_area = centered_rect(72, 56, area);
    draw_modal_shadow(f.buffer_mut(), menu_area);

    let items: Vec<ListItem> = SEQUENCES
        .iter()
        .enumerate()
        .map(|(idx, sequence)| {
            let selected = idx == app.menu_index;
            let fg = if selected { BG_TOP } else { BRIGHT };
            let bg = if selected {
                Color::Rgb(250, 210, 92)
            } else {
                PANEL_BG
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", idx + 1),
                    Style::default()
                        .fg(if selected { BG_TOP } else { DIM })
                        .bg(bg),
                ),
                Span::styled(
                    format!("{:<18}", sequence.name),
                    Style::default().fg(fg).bg(bg).add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
                ),
                Span::styled(
                    format!("  {}", sequence.description),
                    Style::default()
                        .fg(if selected { BG_TOP } else { SOFT })
                        .bg(bg),
                ),
            ]))
        })
        .collect();

    let list = List::new(items).block(modal_block(" SEQUENCES  j/k Enter Esc ", accent));
    f.render_widget(list, menu_area);
}

fn draw_help(f: &mut Frame, area: Rect, accent: Color) {
    f.render_widget(Clear, area);
    draw_backdrop(f.buffer_mut(), area, 0.0, accent);
    let help_area = centered_rect(72, 78, area);
    draw_modal_shadow(f.buffer_mut(), help_area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("   navigation   ", Style::default().fg(BG_TOP).bg(accent)),
            Span::styled(
                "  j/k choose parameter    h/l adjust    H/L coarse",
                Style::default().fg(BRIGHT),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "   playback     ",
                Style::default().fg(BG_TOP).bg(Color::Rgb(84, 240, 150)),
            ),
            Span::styled(
                "  Space play or pause    q/Esc leave",
                Style::default().fg(BRIGHT),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "   programs     ",
                Style::default().fg(BG_TOP).bg(Color::Rgb(250, 210, 92)),
            ),
            Span::styled(
                "  1-5 quick presets    p presets    s sequences",
                Style::default().fg(BRIGHT),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "   spectacle    ",
                Style::default().fg(BG_TOP).bg(Color::Rgb(210, 145, 255)),
            ),
            Span::styled(
                "  v/V visual    e emergence    g geom    r/R drift (toggle/reverse)    n noise    m mist",
                Style::default().fg(BRIGHT),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "   Emergence voices spawn by simple harmonic ratios, mutate gently, and fade by consonance.",
            Style::default().fg(SOFT),
        )),
        Line::from(Span::styled(
            "   Harmonics view maps the live stereo phase trace and the active H1-H6 timbre partials.",
            Style::default().fg(SOFT),
        )),
        Line::from(Span::styled(
            "   Penrose mode walks a Fibonacci-word Conway worm; tile pairs (LL/LS/SL) pick 3:2 / 5:4 / 4:3.",
            Style::default().fg(SOFT),
        )),
        Line::from(Span::styled(
            "   Drift adds a Shepard-Risset glissando: 7 octaves under a bell window, endlessly rising or falling.",
            Style::default().fg(SOFT),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "   Press any key to return.",
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        )),
    ];

    let block = modal_block(" HELP ", accent);
    f.render_widget(
        Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: false })
            .style(Style::default().bg(PANEL_BG)),
        help_area,
    );
}

fn draw_backdrop(buf: &mut Buffer, area: Rect, elapsed: f64, accent: Color) {
    let tick = (elapsed * 9.0) as u16;
    for y in 0..area.height {
        let t = y as f32 / area.height.max(1) as f32;
        let base = if t < 0.55 {
            blend_color(BG_TOP, BG_MID, t / 0.55)
        } else {
            blend_color(BG_MID, BG_BOTTOM, (t - 0.55) / 0.45)
        };
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, area.y + y)];
            cell.set_char(' ');
            cell.set_bg(base);
            let hash = x
                .wrapping_mul(37)
                .wrapping_add(y.wrapping_mul(17))
                .wrapping_add(tick);
            if hash % 41 == 0 {
                let drift = ((x as f64 * 0.17 + elapsed * 0.6).sin()
                    + (y as f64 * 0.31 + elapsed * 0.3).cos())
                    * 0.5;
                if drift > -0.2 {
                    cell.set_char(if hash % 82 == 0 {
                        '\u{2219}'
                    } else {
                        '\u{00B7}'
                    });
                    cell.set_fg(dim_color(accent, 0.20 + drift as f32 * 0.18));
                }
            }
        }
    }
}

fn draw_visual_backdrop(buf: &mut Buffer, area: Rect, elapsed: f64, accent: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    for y in 0..area.height {
        let t = y as f32 / area.height.max(1) as f32;
        let bg = blend_color(PANEL_BG, PANEL_ALT, t * 0.6);
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, area.y + y)];
            cell.set_char(' ');
            cell.set_bg(bg);
            let wave =
                (x as f64 * 0.12 + elapsed * 0.8).sin() + (y as f64 * 0.23 - elapsed * 0.4).cos();
            if (x + y * 3 + elapsed as u16).is_multiple_of(23) && wave > 0.4 {
                cell.set_char('\u{00B7}');
                cell.set_fg(dim_color(accent, 0.24));
            }
        }
    }
}

fn draw_spectrum_floor(buf: &mut Buffer, area: Rect, accent: Color) {
    if area.height < 2 {
        return;
    }
    let labels = [
        ("Delta", 1),
        ("Theta", 2),
        ("Alpha", 4),
        ("Beta", 7),
        ("Gamma", 11),
    ];
    let y = area.height.saturating_sub(1);
    for (label, position) in labels {
        let x = area.width.saturating_mul(position) / 12;
        write_str(
            buf,
            area,
            x,
            y,
            label,
            Style::default().fg(dim_color(accent, 0.56)).bg(PANEL_BG),
        );
    }
}

fn spectral_ribbon(beat_freq: f32) -> Line<'static> {
    let bands = [
        ("Delta", Color::Rgb(180, 120, 255)),
        ("Theta", Color::Rgb(140, 100, 255)),
        ("Alpha", Color::Rgb(80, 230, 230)),
        ("Beta", Color::Rgb(80, 255, 140)),
        ("Gamma", Color::Rgb(255, 220, 80)),
    ];
    let mut spans = Vec::with_capacity(bands.len() * 3);
    let active_band = spectral_band_index(beat_freq);
    for (idx, (name, color)) in bands.into_iter().enumerate() {
        let active = idx == active_band;
        let style = if active {
            Style::default()
                .fg(BG_TOP)
                .bg(color)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(dim_color(color, 0.65)).bg(BG_TOP)
        };
        spans.push(Span::styled(" ", Style::default().bg(BG_TOP)));
        spans.push(Span::styled(format!(" {name} "), style));
        spans.push(Span::styled(" ", Style::default().fg(DIM).bg(BG_TOP)));
    }
    Line::from(spans).centered()
}

fn spectral_band_index(beat_freq: f32) -> usize {
    if beat_freq < 4.0 {
        0
    } else if beat_freq < 8.0 {
        1
    } else if beat_freq < 13.0 {
        2
    } else if beat_freq < 30.0 {
        3
    } else {
        4
    }
}

fn mode_rail(active: VizMode, accent: Color) -> Line<'static> {
    let modes = [
        VizMode::Waveform,
        VizMode::Spectrum,
        VizMode::Harmonics,
        VizMode::Envelope,
        VizMode::Penrose,
        VizMode::Emergence,
    ];
    let mut spans = Vec::with_capacity(modes.len() * 2);
    for mode in modes {
        let style = if mode == active {
            Style::default()
                .fg(BG_TOP)
                .bg(accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(DIM).bg(PANEL_BG)
        };
        spans.push(Span::styled(format!(" {} ", mode.label()), style));
        spans.push(Span::styled(" ", Style::default().fg(BORDER).bg(PANEL_BG)));
    }
    Line::from(spans).centered()
}

fn meter_line(spec: MeterSpec, active: ActiveParam, width: usize) -> Line<'static> {
    let is_active = spec.param == active;
    let mut spans = Vec::with_capacity(8);
    spans.push(Span::styled(
        if is_active { " \u{25B8}" } else { "  " },
        Style::default().fg(spec.color).bg(PANEL_BG),
    ));
    spans.push(Span::styled(
        format!(" {:<5}", spec.label),
        Style::default()
            .fg(if is_active { BRIGHT } else { SOFT })
            .bg(PANEL_BG)
            .add_modifier(if is_active {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    ));
    spans.push(Span::styled(
        format!(" {:>8} ", spec.value),
        Style::default().fg(DIM).bg(PANEL_BG),
    ));
    spans.extend(meter_bar(spec.ratio, width, spec.color));
    Line::from(spans)
}

fn shepard_base_ratio(freq: f32) -> f32 {
    let min = MIN_BASE_FREQ_HZ as f32;
    let max = MAX_BASE_FREQ_HZ as f32;
    ((freq / min).log2() / (max / min).log2()).clamp(0.0, 1.0)
}

fn meter_bar(value: f32, width: usize, color: Color) -> Vec<Span<'static>> {
    let filled = (value.clamp(0.0, 1.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    vec![
        Span::styled(
            "\u{258C}",
            Style::default().fg(dim_color(color, 0.45)).bg(PANEL_BG),
        ),
        Span::styled(
            "\u{2588}".repeat(filled),
            Style::default().fg(color).bg(PANEL_BG),
        ),
        Span::styled(
            "\u{2591}".repeat(empty),
            Style::default().fg(dim_color(color, 0.25)).bg(PANEL_BG),
        ),
        Span::styled(
            "\u{2590}",
            Style::default().fg(dim_color(color, 0.45)).bg(PANEL_BG),
        ),
    ]
}

fn data_line(label: &'static str, value: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!(" {label:<7}"),
            Style::default().fg(DIM).bg(PANEL_BG),
        ),
        Span::styled(value.to_string(), Style::default().fg(color).bg(PANEL_BG)),
    ])
}

fn sequence_status(app: &App) -> String {
    if let Some(idx) = app.current_sequence {
        let elapsed = app.sequence_elapsed().unwrap_or(0.0);
        let total = SEQUENCES[idx].total_duration_secs;
        let progress = elapsed / total;
        let mins = elapsed as u32 / 60;
        let secs = elapsed as u32 % 60;
        format!(
            "{} {} {mins}:{secs:02}",
            SEQUENCES[idx].name,
            make_progress_bar(progress, 8)
        )
    } else {
        "free running".to_string()
    }
}

fn breath_meter(elapsed: f64) -> String {
    let cycle = 7.5;
    let phase = (elapsed % cycle) / cycle;
    let width = 10usize;
    let fill = if phase < 0.4 {
        (phase / 0.4 * width as f64).round() as usize
    } else if phase < 0.5 {
        width
    } else if phase < 0.9 {
        ((0.9 - phase) / 0.4 * width as f64).round() as usize
    } else {
        1
    };
    format!(
        "{}{}",
        "\u{2588}".repeat(fill.min(width)),
        "\u{2591}".repeat(width.saturating_sub(fill.min(width)))
    )
}

fn command_chip(key: &'static str, label: &'static str, color: Color) -> Span<'static> {
    Span::styled(
        format!(" {key} {label} "),
        Style::default()
            .fg(if color == DIM { SOFT } else { BG_TOP })
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

fn key_chip(key: &'static str, color: Color) -> Span<'static> {
    Span::styled(
        format!(" {key} "),
        Style::default()
            .fg(if color == DIM { SOFT } else { BG_TOP })
            .bg(color)
            .add_modifier(Modifier::BOLD),
    )
}

fn panel_block(title: &'static str, border: Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(PANEL_BG))
        .title(Span::styled(title, Style::default().fg(DIM)))
}

fn modal_block(title: &'static str, accent: Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(PANEL_BG))
        .title(Span::styled(
            title,
            Style::default().fg(BRIGHT).add_modifier(Modifier::BOLD),
        ))
}

fn draw_modal_shadow(buf: &mut Buffer, area: Rect) {
    let shadow_x = area.x.saturating_add(2);
    let shadow_y = area.y.saturating_add(1);
    let shadow = Rect {
        x: shadow_x,
        y: shadow_y,
        width: area.width,
        height: area.height,
    };
    for y in shadow.y..shadow.y.saturating_add(shadow.height).min(buf.area.height) {
        for x in shadow.x..shadow.x.saturating_add(shadow.width).min(buf.area.width) {
            let cell = &mut buf[(x, y)];
            cell.set_bg(SHADOW);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

fn make_phase_wave(elapsed: f32, width: usize) -> String {
    let width = width.clamp(18, 44);
    let glyphs = [
        '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    (0..width)
        .map(|idx| {
            let phase = elapsed as f64 * 2.0 + idx as f64 * 0.42;
            let value = ((phase.sin() + 1.0) * 0.5 * (glyphs.len() - 1) as f64).round() as usize;
            glyphs[value.min(glyphs.len() - 1)]
        })
        .collect()
}

fn make_progress_bar(value: f32, width: usize) -> String {
    let filled = (value.clamp(0.0, 1.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "\u{25AC}".repeat(filled), "\u{25AD}".repeat(empty))
}

fn write_str(buf: &mut Buffer, area: Rect, x: u16, y: u16, text: &str, style: Style) {
    if y >= area.height {
        return;
    }
    for (offset, ch) in text.chars().enumerate() {
        let cell_x = x.saturating_add(offset as u16);
        if cell_x >= area.width {
            break;
        }
        let cell = &mut buf[(area.x + cell_x, area.y + y)];
        cell.set_char(ch);
        cell.set_style(style);
    }
}

fn breathing_color(color: Color, elapsed: f64) -> Color {
    let pulse = 0.62 + 0.22 * (elapsed * 0.8).sin().abs() as f32;
    dim_color(color, pulse)
}

fn dim_color(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(red, green, blue) => Color::Rgb(
            (red as f32 * factor).clamp(0.0, 255.0) as u8,
            (green as f32 * factor).clamp(0.0, 255.0) as u8,
            (blue as f32 * factor).clamp(0.0, 255.0) as u8,
        ),
        other => other,
    }
}

fn shift_color(color: Color, red_delta: u8, green_delta: u8, blue_delta: u8) -> Color {
    match color {
        Color::Rgb(red, green, blue) => Color::Rgb(
            red.saturating_add(red_delta),
            green.saturating_add(green_delta),
            blue.saturating_add(blue_delta),
        ),
        other => other,
    }
}

fn blend_color(start: Color, end: Color, amount: f32) -> Color {
    let (sr, sg, sb) = rgb_from_color(start);
    let (er, eg, eb) = rgb_from_color(end);
    let t = amount.clamp(0.0, 1.0);
    Color::Rgb(
        (sr as f32 + (er as f32 - sr as f32) * t) as u8,
        (sg as f32 + (eg as f32 - sg as f32) * t) as u8,
        (sb as f32 + (eb as f32 - sb as f32) * t) as u8,
    )
}

fn rgb_from_color(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(red, green, blue) => (red, green, blue),
        _ => (128, 128, 128),
    }
}

#[cfg(test)]
mod tests {
    use super::spectral_band_index;

    #[test]
    fn spectral_band_index_matches_frequency_thresholds() {
        let cases = [
            (2.0, 0),
            (3.99, 0),
            (4.0, 1),
            (7.83, 1),
            (8.0, 2),
            (12.99, 2),
            (13.0, 3),
            (29.99, 3),
            (30.0, 4),
            (40.0, 4),
        ];

        for (frequency, expected) in cases {
            assert_eq!(spectral_band_index(frequency), expected);
        }
    }
}
