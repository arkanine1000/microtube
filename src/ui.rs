use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::app::{ActiveParam, App, AppMode, Tab, VizMode};
use crate::knowledge;
use crate::presets::{PRESETS, SEQUENCES, freq_band_name, freq_color};
use crate::shepard::{MAX_BASE_FREQ_HZ, MIN_BASE_FREQ_HZ};
use crate::theme::{
    self, HORIZON, INK_0, INK_1, INK_2, INK_3, INK_4, PANEL, PANEL_RAISED, SEMANTIC, SHADOW,
    SPACE_DEEP, SPACE_MID,
};
use crate::visualization::{
    HARMONIC_PARTIAL_LABELS, HarmonicLattice, harmonic_partial_levels, render_beat_envelope,
    render_braille_waveform, render_emergence, render_harmonic_lattice, render_penrose,
    render_spectrum_bars,
};

// Phase A aliases — keep the old names alive across the file so this is a
// pure refactor. The semantic remapping happens here in one place.
const BG_TOP: Color = SPACE_DEEP;
const BG_MID: Color = SPACE_MID;
const BG_BOTTOM: Color = HORIZON;
const PANEL_BG: Color = PANEL;
const PANEL_ALT: Color = PANEL_RAISED;
const DIM: Color = INK_3;
const SOFT: Color = INK_1;
const BRIGHT: Color = INK_0;
const BORDER: Color = INK_4;
// Selection marker (2) + label (6) + value (10) = 18 cells reserved for
// the meter's non-bar chrome. Anything beyond becomes the gradient bar.
const METER_LINE_FIXED_WIDTH: usize = 18;

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
        let elapsed = app.start_time.elapsed().as_secs_f64();
        let beat_freq = app.params.get_beat_freq();
        let ctx = BackdropCtx {
            elapsed,
            accent: Color::Red,
            rms: 0.0,
            beat_freq,
        };
        draw_backdrop(f.buffer_mut(), size, ctx);
        let msg = Paragraph::new("Terminal too small\nMinimum: 76x30")
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center);
        f.render_widget(msg, size);
        return;
    }

    let elapsed = app.start_time.elapsed().as_secs_f64();
    let beat_freq = app.params.get_beat_freq();
    let accent = freq_color(beat_freq);
    let border_color = breathing_color(accent, elapsed);
    let backdrop = backdrop_ctx(app, accent);

    draw_backdrop(f.buffer_mut(), size, backdrop);

    // Shell border with NO bg fill — the backdrop shows through between panels.
    let shell = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color));
    let shell_inner = shell.inner(size);
    f.render_widget(shell, size);

    match app.tab {
        Tab::Studio => {
            draw_studio(f, shell_inner, app, accent, border_color);
            // Modal overlays paint last so they sit above all other chrome.
            match app.mode {
                AppMode::PresetSelect => draw_preset_menu(f, size, app, accent),
                AppMode::SequenceSelect => draw_sequence_menu(f, size, app, accent),
                AppMode::PresetName => draw_save_preset_prompt(f, size, app, accent),
                AppMode::Help => draw_help(f, size, accent),
                AppMode::Normal => {}
            }
        }
        Tab::Knowledge => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(2), Constraint::Min(8)])
                .split(shell_inner);
            draw_header_bar(f, chunks[0], app, accent);
            knowledge::draw(f, chunks[1], app, accent, border_color);
        }
    }
}

/// Studio tab — the main instrument layout.
///
/// Vertical: header (2) / content (Min) / film-strip (0 or 1) / footer (1).
/// Content splits horizontally into left rail (params), center stage
/// (visualization), and right rail (session + harmonics).
fn draw_studio(f: &mut Frame, area: Rect, app: &mut App, accent: Color, border: Color) {
    let sequence_active = app.current_sequence.is_some();
    let film_height = if sequence_active { 1 } else { 0 };

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // header bar (wordmark + tabs + indicators + rule)
            Constraint::Min(8),    // content
            Constraint::Length(film_height),
            Constraint::Length(1), // contextual footer
        ])
        .split(area);

    draw_header_bar(f, rows[0], app, accent);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Min(20)])
        .split(rows[1]);

    draw_info_column(f, cols[0], app, accent, border);
    draw_viz_stage(f, cols[1], app, accent);

    if sequence_active {
        draw_film_strip(f, rows[2], app, accent);
    }

    draw_footer_hints(f, rows[3], app, accent);
}

/// Thin two-row header that runs across the top of the shell.
///
/// Row 1: `◈ microtube` wordmark · tab strip · band readout · timer · play dot.
/// Row 2: a hairline rule that visually separates the chrome from the stage.
fn draw_header_bar(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let playing = app.params.playing.load(std::sync::atomic::Ordering::Relaxed);
    let beat_freq = app.params.get_beat_freq();
    let base_freq = app.params.get_base_freq();
    let band = freq_band_name(beat_freq);
    let secs = app.session_elapsed() as u32;
    let timer = format!("{:02}:{:02}", secs / 60, secs % 60);
    let elapsed = app.start_time.elapsed().as_secs_f64();
    let sigil = viz_sigil(app.viz_mode);

    // --- Row 1 -----------------------------------------------------------
    let buf = f.buffer_mut();
    let y0 = area.y;

    // Wordmark + sigil at the left.
    let crest_breath = 0.65 + 0.30 * (elapsed * 1.1).sin().abs() as f32;
    let crest_color = theme::dim(accent, crest_breath);
    let crest = format!(" {sigil} microtube ");
    write_str(
        buf,
        area,
        0,
        0,
        &crest,
        Style::default().fg(crest_color).add_modifier(Modifier::BOLD),
    );

    // Tab strip — browser-style underline rendered in row 2.
    let tab_origin_x = (crest.chars().count() as u16 + 2).min(area.width);
    let tabs = [Tab::Studio, Tab::Knowledge];
    let mut tab_x = tab_origin_x;
    let mut tab_runs: Vec<(u16, u16, bool)> = Vec::with_capacity(tabs.len());
    for tab in tabs {
        let label = tab.label();
        let active = tab == app.tab;
        let label_str = format!("  {label}  ");
        let len = label_str.chars().count() as u16;
        let style = if active {
            Style::default().fg(INK_0).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(INK_3)
        };
        write_str(buf, area, tab_x, 0, &label_str, style);
        tab_runs.push((tab_x, len, active));
        tab_x = tab_x.saturating_add(len);
    }

    // Right cluster: 5-dot band indicator · band · beat · timer · play dot.
    // Built progressively from most-essential to most-optional, dropping
    // segments from the head of the optional list when the cluster would
    // collide with the tab strip.
    let beat_str = format!("{band} · {beat_freq:.1} Hz");
    let base_str = format!("L {base_freq:.0}  R {:.0}", base_freq + beat_freq);
    let play_glyph = if playing { '\u{25CF}' } else { '\u{25CB}' };
    let play_breath = if playing {
        0.55 + 0.35 * (elapsed * 1.8).sin().abs() as f32
    } else {
        0.45
    };
    let active_band_idx = spectral_band_index(beat_freq);
    let band_dots = band_dot_indicator(active_band_idx, accent);

    // Essentials always render: beat readout, timer, play dot, with margins.
    let essential: Vec<(String, Style)> = vec![
        (
            beat_str,
            Style::default().fg(accent).add_modifier(Modifier::BOLD),
        ),
        ("   ".to_string(), Style::default()),
        (timer, Style::default().fg(INK_0)),
        ("  ".to_string(), Style::default()),
        (
            play_glyph.to_string(),
            Style::default().fg(theme::dim(accent, play_breath)),
        ),
        (" ".to_string(), Style::default()),
    ];
    // Optional prefix segments, ordered by drop-priority (drop the first
    // one first when the cluster won't fit).
    let optional_prefixes: [Vec<(String, Style)>; 2] = [
        // Lowest priority — drop first on narrow terminals.
        vec![
            (base_str, Style::default().fg(INK_3)),
            ("   ".to_string(), Style::default()),
        ],
        // Higher priority — drop only if even essentials need more room.
        vec![
            (band_dots, Style::default().fg(accent)),
            ("  ".to_string(), Style::default()),
        ],
    ];

    let min_gap_after_tabs: u16 = 2;
    let available = area
        .width
        .saturating_sub(tab_x)
        .saturating_sub(min_gap_after_tabs);

    let mut right_segments: Vec<(String, Style)> = Vec::new();
    let mut drop_count = 0usize;
    loop {
        right_segments.clear();
        for prefix in optional_prefixes.iter().skip(drop_count) {
            right_segments.extend(prefix.iter().cloned());
        }
        right_segments.extend(essential.iter().cloned());
        let w: usize = right_segments.iter().map(|(s, _)| s.chars().count()).sum();
        if (w as u16) <= available || drop_count >= optional_prefixes.len() {
            break;
        }
        drop_count += 1;
    }
    let right_width: usize = right_segments.iter().map(|(s, _)| s.chars().count()).sum();
    let right_origin = area.width.saturating_sub(right_width as u16 + 1);
    let mut rx = right_origin;
    for (text, style) in &right_segments {
        write_str(buf, area, rx, 0, text, *style);
        rx = rx.saturating_add(text.chars().count() as u16);
    }

    // --- Row 2 -----------------------------------------------------------
    if area.height >= 2 {
        let y1 = y0 + 1;
        // Hairline rule across the whole width.
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, y1)];
            cell.set_char('\u{2500}');
            cell.set_fg(INK_4);
        }
        // Active-tab underline highlight — overrides the hairline in
        // accent color for the tab's label width, with a soft veil under
        // the inactive tabs.
        for (tx, len, active) in tab_runs {
            for i in 0..len {
                let xx = area.x + tx + i;
                if xx >= area.x + area.width {
                    break;
                }
                let cell = &mut buf[(xx, y1)];
                if active {
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

/// Compact 5-glyph band indicator. The active band is rendered as `●`;
/// the others as `·`. Whole row is colored accent so the listener can
/// tell at a glance which band they're in by both color and shape.
fn band_dot_indicator(active: usize, _accent: Color) -> String {
    let mut s = String::with_capacity(9);
    for i in 0..5 {
        if i > 0 {
            s.push(' ');
        }
        if i == active {
            s.push('\u{25CF}');
        } else {
            s.push('\u{00B7}');
        }
    }
    s
}

/// One-character glyph that morphs with the visualization mode. Used in
/// the wordmark crest so the title carries a hint of the active stage.
fn viz_sigil(mode: VizMode) -> char {
    match mode {
        VizMode::Waveform => '\u{25C8}',
        VizMode::Spectrum => '\u{25A4}',
        VizMode::Harmonics => '\u{2737}',
        VizMode::Envelope => '\u{25C9}',
        VizMode::Penrose => '\u{273B}',
        VizMode::Emergence => '\u{273A}',
    }
}

/// Borderless visualization stage. The active mode renders directly into
/// the area; a floating sigil/name sits in the top-left, the mode rail
/// runs along the bottom edge, and a 1-cell accent glow softens the
/// rectangle so the stage feels framed without literal lines.
fn draw_viz_stage(f: &mut Frame, area: Rect, app: &mut App, accent: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let (samples_l, samples_r) = if let Ok(buf) = app.viz_buffer.try_lock() {
        buf.read_ordered()
    } else {
        return;
    };

    let elapsed = app.start_time.elapsed().as_secs_f64();
    let backdrop = backdrop_ctx(app, accent);
    draw_visual_backdrop(f.buffer_mut(), area, backdrop);

    // Render the active visualization across the entire area. Each renderer
    // paints into the cells; the inner-glow we apply afterwards is layered
    // on the outermost ring only and so does not overdraw the imagery.
    match app.viz_mode {
        VizMode::Waveform => {
            render_waveform_stage(f.buffer_mut(), area, &samples_l, &samples_r, accent)
        }
        VizMode::Spectrum => {
            let combined: Vec<f32> = samples_l
                .iter()
                .zip(&samples_r)
                .map(|(left, right)| (left + right) * 0.5)
                .collect();
            render_spectrum_bars(
                f.buffer_mut(),
                area,
                &combined,
                &mut app.spectrum_bars,
                accent,
            );
            draw_spectrum_floor(f.buffer_mut(), area, accent);
        }
        VizMode::Harmonics => {
            let beat_freq = app.params.get_beat_freq() as f64;
            let harmonics = app.params.get_harmonics() as f64;
            let harmonic_weights = app.params.get_timbre().weights();
            render_harmonic_lattice(
                f.buffer_mut(),
                area,
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
            render_beat_envelope(f.buffer_mut(), area, elapsed, beat_freq, accent);
        }
        VizMode::Penrose => {
            let beat_freq = app.params.get_beat_freq() as f64;
            render_penrose(f.buffer_mut(), area, elapsed, beat_freq, accent);
        }
        VizMode::Emergence => {
            let snapshot = if let Ok(snap) = app.emergence_snapshot.try_lock() {
                snap.clone()
            } else {
                return;
            };
            render_emergence(f.buffer_mut(), area, &snapshot, elapsed, accent);
        }
    }

    // Inner-glow vignette — 1-cell accent tint on every side, then a
    // brighter accent on the four corners. Gives the borderless stage a
    // soft frame that respects the active band color.
    apply_inner_glow(f.buffer_mut(), area, accent, app.signals.rms_l.max(app.signals.rms_r));

    // Floating sigil + mode name at the top-left, in the empty cell row
    // above the visualization content (we reserved no rows; we just draw
    // on top of the leftmost few cells of the visualization).
    let sigil = viz_sigil(app.viz_mode);
    let label = format!(" {sigil} {} ", app.viz_mode.label());
    write_str(
        f.buffer_mut(),
        area,
        1,
        0,
        &label,
        Style::default().fg(accent).add_modifier(Modifier::BOLD),
    );

    // Mode rail along the bottom edge.
    if area.height >= 2 {
        draw_mode_rail(f.buffer_mut(), area, app.viz_mode, accent);
    }
}

/// Paint a soft 1-cell accent glow on the rectangle's perimeter. Edge
/// cells are mixed (not overwritten) so any imagery rendered first still
/// reads through. `rms` brightens the glow on louder passages.
fn apply_inner_glow(buf: &mut Buffer, area: Rect, accent: Color, rms: f32) {
    if area.width < 2 || area.height < 2 {
        return;
    }
    let intensity = (0.18 + rms.clamp(0.0, 1.0) * 0.22).min(0.45);
    let tint = theme::dim(accent, 0.55);
    let corner_tint = theme::dim(accent, 0.85);
    let x0 = area.x;
    let x1 = area.x + area.width - 1;
    let y0 = area.y;
    let y1 = area.y + area.height - 1;

    let edge = |existing: Color| theme::mix(existing, tint, intensity);
    for x in x0..=x1 {
        // top
        let cell = &mut buf[(x, y0)];
        let bg = cell.bg;
        cell.set_bg(edge(bg));
        // bottom
        let cell = &mut buf[(x, y1)];
        let bg = cell.bg;
        cell.set_bg(edge(bg));
    }
    for y in y0..=y1 {
        let cell = &mut buf[(x0, y)];
        let bg = cell.bg;
        cell.set_bg(edge(bg));
        let cell = &mut buf[(x1, y)];
        let bg = cell.bg;
        cell.set_bg(edge(bg));
    }
    // Corner accents — brighter, draw a small glyph instead of just bg.
    for (cx, cy, glyph) in [
        (x0, y0, '\u{256D}'),
        (x1, y0, '\u{256E}'),
        (x0, y1, '\u{2570}'),
        (x1, y1, '\u{256F}'),
    ] {
        let cell = &mut buf[(cx, cy)];
        cell.set_char(glyph);
        cell.set_fg(corner_tint);
    }
}

/// A row of viz-mode names rendered along the bottom edge of the stage.
/// The active mode is bright + a leading dot; the others are dim. No
/// pill chips — quieter than the old `title_bottom` rail.
fn draw_mode_rail(buf: &mut Buffer, area: Rect, active: VizMode, accent: Color) {
    let modes = [
        VizMode::Waveform,
        VizMode::Spectrum,
        VizMode::Harmonics,
        VizMode::Envelope,
        VizMode::Penrose,
        VizMode::Emergence,
    ];
    let y_rel = area.height.saturating_sub(1);
    let inactive_glyph = '\u{00B7}';
    let active_glyph = '\u{25C9}';

    // Three densities, from richest to most compact. Pick the first that
    // fits the available width.
    let candidates: [(bool, u16); 3] = [
        (true, 2),  // glyph + full label, 2-space gap
        (true, 1),  // glyph + abbrev label, 1-space gap
        (false, 1), // glyph only, 1-space gap
    ];

    let mut chosen: Option<(Vec<(String, Style)>, u16)> = None;
    for &(with_label, gap) in &candidates {
        let segments = build_mode_rail_segments(
            &modes,
            active,
            accent,
            active_glyph,
            inactive_glyph,
            with_label,
            gap as usize,
            with_label && gap == 2, // first variant uses full labels
        );
        let total: u16 = segments.iter().map(|(s, _)| s.chars().count() as u16).sum();
        if total <= area.width {
            chosen = Some((segments, total));
            break;
        }
    }

    let Some((segments, total)) = chosen else {
        // Even the most compact form overflows — render nothing rather than
        // clip mid-glyph. Caller's other adornments still appear.
        return;
    };
    let start_x = area.width.saturating_sub(total) / 2;
    let mut x = start_x;
    for (text, style) in &segments {
        write_str(buf, area, x, y_rel, text, *style);
        x = x.saturating_add(text.chars().count() as u16);
    }
}

fn build_mode_rail_segments(
    modes: &[VizMode],
    active: VizMode,
    accent: Color,
    active_glyph: char,
    inactive_glyph: char,
    with_label: bool,
    gap_cols: usize,
    full_label: bool,
) -> Vec<(String, Style)> {
    let mut segments: Vec<(String, Style)> = Vec::with_capacity(modes.len() * 2);
    for (i, mode) in modes.iter().enumerate() {
        let is_active = *mode == active;
        let glyph = if is_active { active_glyph } else { inactive_glyph };
        let style = if is_active {
            Style::default().fg(accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(INK_3)
        };
        let text = if with_label {
            let label = if full_label {
                mode.label().to_string()
            } else {
                mode_short_label(*mode).to_string()
            };
            format!("{glyph} {label}")
        } else {
            glyph.to_string()
        };
        segments.push((text, style));
        if i + 1 < modes.len() {
            segments.push((" ".repeat(gap_cols), Style::default().fg(INK_4)));
        }
    }
    segments
}

fn mode_short_label(mode: VizMode) -> &'static str {
    match mode {
        VizMode::Waveform => "Wav",
        VizMode::Spectrum => "Spc",
        VizMode::Harmonics => "Hrm",
        VizMode::Envelope => "Env",
        VizMode::Penrose => "Pen",
        VizMode::Emergence => "Emg",
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
        Style::default().fg(dim_color(accent, 0.82)),
    );
    write_str(
        buf,
        area,
        2,
        halves[0].height,
        "RIGHT EAR",
        Style::default().fg(shift_color(accent, 34, 8, 44)),
    );
}

/// Left rail — the parameter meters. A panel card holding the 8
/// adjustable params; the active one carries an accent-colored arrow.
/// Info column — parameters / session / partials stacked vertically in one
/// panel card, separated by hairline rules. The visualization gets all
/// remaining horizontal space; this column gets the meters' width back so
/// the gradient bars have room to breathe.
fn draw_info_column(f: &mut Frame, area: Rect, app: &App, accent: Color, border: Color) {
    let block = panel_block(" instrument ", border);
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Section heights (excluding the two hairline rules between them).
    let param_h: u16 = 8;
    let session_h: u16 = 5 + app.status_message.as_ref().map(|_| 1).unwrap_or(0);
    let partials_h: u16 = HARMONIC_PARTIAL_LABELS.len() as u16 + 1;

    let parts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(param_h),
            Constraint::Length(1),
            Constraint::Length(session_h),
            Constraint::Length(1),
            Constraint::Min(partials_h),
        ])
        .split(inner);

    draw_param_section(f, parts[0], app, accent);
    draw_hairline(f.buffer_mut(), parts[1], inner.width, " session ");
    draw_session_section(f, parts[2], app, accent);
    draw_hairline(f.buffer_mut(), parts[3], inner.width, " partials ");
    draw_partials_section(f, parts[4], app, accent);
}

fn draw_param_section(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    let base = app.params.get_base_freq();
    let beat = app.params.get_beat_freq();
    let shepard_base = app.params.get_shepard_base_freq();
    let specs = [
        MeterSpec {
            param: ActiveParam::BaseFreq,
            label: "base",
            value: format!("{base:>5.1} Hz"),
            ratio: (base - 50.0) / 450.0,
            color: SEMANTIC.base,
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
            color: SEMANTIC.gain,
        },
        MeterSpec {
            param: ActiveParam::Harmonics,
            label: "warm",
            value: format!("{:>3.0}%", app.params.get_harmonics() * 100.0),
            ratio: app.params.get_harmonics(),
            color: SEMANTIC.warm,
        },
        MeterSpec {
            param: ActiveParam::Emergence,
            label: "life",
            value: format!("{:>3.0}%", app.params.get_emergence() * 100.0),
            ratio: app.params.get_emergence(),
            color: SEMANTIC.life,
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
            color: SEMANTIC.drift,
        },
        MeterSpec {
            param: ActiveParam::ShepardBase,
            label: "dbase",
            value: format!("{shepard_base:>5.1} Hz"),
            ratio: shepard_base_ratio(shepard_base),
            color: SEMANTIC.d_base,
        },
        MeterSpec {
            param: ActiveParam::NoiseLevel,
            label: "mist",
            value: format!("{:>3.0}%", app.params.get_noise_level() * 100.0),
            ratio: app.params.get_noise_level(),
            color: SEMANTIC.mist,
        },
    ];

    let meter_width = parameter_meter_width(area.width);
    let rows: Vec<Line> = specs
        .into_iter()
        .map(|spec| {
            let since = app.signals.since_adjust(spec.param);
            meter_line(spec, app.active_param, meter_width, since)
        })
        .collect();
    f.render_widget(
        Paragraph::new(rows).style(Style::default().bg(PANEL_BG)),
        area,
    );
}

fn draw_session_section(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    let beat = app.params.get_beat_freq();
    let epoch_or_preset = if let Some(name) = app.current_step_name() {
        data_line("epoch", name, accent)
    } else {
        let preset = app.current_preset_name().unwrap_or("Custom");
        data_line("preset", preset, accent)
    };
    let band = freq_band_name(beat);
    let breath = breath_meter(app.session_elapsed() as f64);
    let emergence = app.params.get_emergence();
    let mode_label = app.params.get_spawn_mode().label();
    let em_status = if emergence > 0.01 {
        if let Ok(snap) = app.emergence_snapshot.try_lock() {
            format!(
                "{} voices · gen {} · {}",
                snap.voices.len(),
                snap.generation_count,
                mode_label
            )
        } else {
            format!("active · {mode_label}")
        }
    } else {
        format!("quiet · {mode_label}")
    };
    let mist = format!(
        "{} · {}",
        app.params.get_mist_type().label(),
        app.params.get_mist_type().texture()
    );
    let mut rows = vec![
        epoch_or_preset,
        data_line("band", band, accent),
        data_line("mist", &mist, SEMANTIC.mist),
        Line::from(vec![
            Span::styled(" life   ", Style::default().fg(DIM).bg(PANEL_BG)),
            Span::styled(em_status, Style::default().fg(accent).bg(PANEL_BG)),
        ]),
        Line::from(vec![
            Span::styled(" breath ", Style::default().fg(DIM).bg(PANEL_BG)),
            Span::styled(breath, Style::default().fg(accent).bg(PANEL_BG)),
        ]),
    ];
    if let Some(message) = &app.status_message {
        rows.push(data_line("note", message, SEMANTIC.warm));
    }
    f.render_widget(
        Paragraph::new(rows).style(Style::default().bg(PANEL_BG)),
        area,
    );
}

fn draw_partials_section(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    let harmonics = app.params.get_harmonics();
    let timbre = app.params.get_timbre();
    let levels = harmonic_partial_levels(harmonics as f64, timbre.weights());
    let max_level = levels
        .iter()
        .copied()
        .fold(0.0_f64, f64::max)
        .max(f64::EPSILON);
    // The bar gets a generous slice of the wider column.
    let bar_width = area.width.saturating_sub(10).clamp(4, 24) as usize;

    let mut rows = Vec::with_capacity(HARMONIC_PARTIAL_LABELS.len() + 1);
    for (idx, (id, label)) in HARMONIC_PARTIAL_LABELS.iter().enumerate() {
        let relative = (levels[idx] / max_level).clamp(0.0, 1.0) as f32;
        let glow = 0.28 + relative * 0.72;
        let mut row_spans: Vec<Span<'static>> = Vec::with_capacity(2 + bar_width);
        row_spans.push(Span::styled(
            format!(" {id:<2} "),
            Style::default()
                .fg(dim_color(accent, (glow + 0.14).min(1.0)))
                .bg(PANEL_BG)
                .add_modifier(Modifier::BOLD),
        ));
        row_spans.push(Span::styled(
            format!("{label:<5}"),
            Style::default().fg(DIM).bg(PANEL_BG),
        ));
        row_spans.extend(meter_bar(relative, bar_width, accent, 0.0, PANEL_BG));
        rows.push(Line::from(row_spans));
    }
    rows.push(Line::from(vec![
        Span::styled(" timbre ", Style::default().fg(DIM).bg(PANEL_BG)),
        Span::styled(timbre.label(), Style::default().fg(accent).bg(PANEL_BG)),
    ]));
    f.render_widget(
        Paragraph::new(rows).style(Style::default().bg(PANEL_BG)),
        area,
    );
}

/// Hairline section divider — a thin rule running across the panel's
/// interior with a centered label sitting on top of it.
fn draw_hairline(buf: &mut Buffer, area: Rect, panel_width: u16, label: &str) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let y = area.y;
    for x in 0..area.width {
        let cell = &mut buf[(area.x + x, y)];
        cell.set_char('\u{2500}');
        cell.set_fg(INK_4);
        cell.set_bg(PANEL_BG);
    }
    let lx = panel_width.saturating_sub(label.chars().count() as u16) / 2;
    for (i, ch) in label.chars().enumerate() {
        let xx = area.x + lx + i as u16;
        if xx >= area.x + area.width {
            break;
        }
        let cell = &mut buf[(xx, y)];
        cell.set_char(ch);
        cell.set_fg(INK_2);
        cell.set_bg(PANEL_BG);
    }
}

/// Sequence film-strip — a 1-row visualization of the active sequence's
/// steps, with the current step luminous and the others dim. Renders
/// nothing if no sequence is active (the caller gates this).
fn draw_film_strip(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    let Some(seq_idx) = app.current_sequence else {
        return;
    };
    let sequence = &SEQUENCES[seq_idx];
    let elapsed = app.sequence_elapsed().unwrap_or(0.0);
    if area.width == 0 || area.height == 0 {
        return;
    }

    // Find active step.
    let mut acc = 0.0_f32;
    let mut active = sequence.steps.len();
    let mut step_progress = 0.0_f32;
    for (i, step) in sequence.steps.iter().enumerate() {
        if elapsed < acc + step.duration_secs {
            active = i;
            step_progress = (elapsed - acc) / step.duration_secs;
            break;
        }
        acc += step.duration_secs;
    }

    let buf = f.buffer_mut();
    let y = area.y;
    let total = sequence.total_duration_secs.max(1.0);
    let bar_width = area.width;

    for x in 0..bar_width {
        let t_pos = (x as f32 / bar_width as f32) * total;
        let mut walking = 0.0_f32;
        let mut step_idx = sequence.steps.len() - 1;
        for (i, step) in sequence.steps.iter().enumerate() {
            if t_pos < walking + step.duration_secs {
                step_idx = i;
                break;
            }
            walking += step.duration_secs;
        }
        let is_active = step_idx == active;
        let is_past = step_idx < active;

        // Cell-by-cell rendering. Boundaries between steps are marked with
        // a thin glyph; otherwise we draw a continuous line that shifts
        // luminance: past steps very dim, future steps medium, active step
        // bright with a sliding "head" at the current progress within it.
        let (glyph, fg) = if is_active {
            // Progress head — bright at current point, fading to either side.
            let cell_within_step = ((x as f32 / bar_width as f32) * total - acc).max(0.0)
                / sequence.steps[active].duration_secs.max(0.001);
            let dist = (cell_within_step - step_progress).abs();
            let head = (1.0 - (dist * 6.0).min(1.0)).max(0.0);
            let lum = 0.50 + head * 0.50;
            ('\u{2501}', theme::dim(accent, lum))
        } else if is_past {
            ('\u{2500}', INK_4)
        } else {
            ('\u{2500}', theme::dim(accent, 0.30))
        };
        let cell = &mut buf[(area.x + x, y)];
        cell.set_char(glyph);
        cell.set_fg(fg);
    }

    // Step boundary ticks.
    let mut acc2 = 0.0_f32;
    for step in sequence.steps.iter().take(sequence.steps.len() - 1) {
        acc2 += step.duration_secs;
        let xpos = ((acc2 / total) * bar_width as f32).round() as u16;
        if xpos >= bar_width {
            continue;
        }
        let cell = &mut buf[(area.x + xpos, y)];
        cell.set_char('\u{2502}');
        cell.set_fg(INK_3);
    }
}

/// Contextual key-hint footer. Shows only the keys relevant to the
/// current AppMode. A single quiet line, anchored at the bottom of the
/// shell, ink color throughout — no chip backgrounds.
fn draw_footer_hints(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let hints: Vec<(&str, &str)> = match app.mode {
        AppMode::Normal => vec![
            ("h/l", "tune"),
            ("j/k", "select"),
            ("space", "play"),
            ("p", "preset"),
            ("s", "seq"),
            ("v", "viz"),
            ("e", "life"),
            ("?", "help"),
            ("q", "quit"),
        ],
        AppMode::PresetSelect => vec![
            ("j/k", "navigate"),
            ("enter", "apply"),
            ("d", "delete"),
            ("esc", "back"),
        ],
        AppMode::SequenceSelect => vec![
            ("j/k", "navigate"),
            ("enter", "start"),
            ("esc", "back"),
        ],
        AppMode::PresetName => vec![
            ("type", "name"),
            ("enter", "save"),
            ("esc", "cancel"),
        ],
        AppMode::Help => vec![("any", "back")],
    };

    // Try densities from richest to most compact until one fits.
    // 0 = key + label + wide gap; 1 = key + label + narrow gap;
    // 2 = key only (chips); 3 = key only with single-space gap.
    let densities = [
        (true, 3u16),  // key + label, 3-space gap
        (true, 2u16),  // key + label, 2-space gap
        (true, 1u16),  // key + label, 1-space gap
        (false, 2u16), // key only, 2-space gap
        (false, 1u16), // key only, 1-space gap
    ];
    let line = densities
        .iter()
        .find_map(|&(with_label, gap)| {
            let line = footer_hint_line(&hints, accent, with_label, gap);
            (line.width() as u16 <= area.width).then_some(line)
        })
        .unwrap_or_else(|| footer_hint_line(&hints, accent, false, 1));
    f.render_widget(Paragraph::new(line).centered(), area);
}

fn footer_hint_line(
    hints: &[(&str, &str)],
    accent: Color,
    with_label: bool,
    gap: u16,
) -> Line<'static> {
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(hints.len() * 4);
    for (i, (key, label)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" ".repeat(gap as usize)));
        }
        spans.push(Span::styled(
            format!(" {key} "),
            Style::default()
                .fg(accent)
                .add_modifier(Modifier::BOLD),
        ));
        if with_label {
            spans.push(Span::styled(
                format!(" {}", *label),
                Style::default().fg(INK_3),
            ));
        }
    }
    Line::from(spans)
}

/// Wash the entire shell area with a low-opacity black, then carve out
/// the modal card on top. Replaces the old offset-shadow trick — pulls
/// the eye fully onto the modal without the underlying UI competing.
fn draw_modal_dim(buf: &mut Buffer, area: Rect) {
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, area.y + y)];
            // Mix the existing background toward SHADOW by 65%.
            let bg = cell.bg;
            cell.set_bg(theme::mix(bg, SHADOW, 0.65));
            // Also dim foreground content uniformly so text behind the
            // wash doesn't compete with the modal's lit type.
            let fg = cell.fg;
            cell.set_fg(theme::mix(fg, SHADOW, 0.55));
        }
    }
}

/// Render an elevated modal card.
///
/// Lays a 1-cell accent top rule, a thin double border around the body,
/// and a 1-row footer area with contextual key hints. Returns the inner
/// rect available for the modal's content.
fn draw_modal_card(
    f: &mut Frame,
    area: Rect,
    title: &str,
    hints: &[(&str, &str)],
    accent: Color,
) -> Rect {
    let buf = f.buffer_mut();
    // Solid panel fill behind the card.
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, area.y + y)];
            cell.set_char(' ');
            cell.set_bg(PANEL);
        }
    }
    // Top accent rule.
    if area.height >= 1 {
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, area.y)];
            cell.set_char('\u{2581}');
            cell.set_fg(accent);
        }
        // Title left-anchored on top of the rule.
        let title_text = format!(" {title} ");
        for (i, ch) in title_text.chars().enumerate() {
            let xx = area.x + 2 + i as u16;
            if xx >= area.x + area.width {
                break;
            }
            let cell = &mut buf[(xx, area.y)];
            cell.set_char(ch);
            cell.set_fg(SPACE_DEEP);
            cell.set_bg(accent);
        }
    }
    // Bottom footer hints.
    if area.height >= 2 {
        let y = area.y + area.height - 1;
        for x in 0..area.width {
            let cell = &mut buf[(area.x + x, y)];
            cell.set_char(' ');
            cell.set_bg(PANEL_RAISED);
        }
        let mut x = 2u16;
        for (i, (key, label)) in hints.iter().enumerate() {
            if i > 0 {
                x = x.saturating_add(3);
            }
            let key_text = format!(" {key} ");
            for ch in key_text.chars() {
                let xx = area.x + x;
                if xx >= area.x + area.width {
                    break;
                }
                let cell = &mut buf[(xx, y)];
                cell.set_char(ch);
                cell.set_fg(accent);
                cell.set_bg(PANEL_RAISED);
                cell.set_style(
                    Style::default()
                        .fg(accent)
                        .bg(PANEL_RAISED)
                        .add_modifier(Modifier::BOLD),
                );
                x = x.saturating_add(1);
            }
            x = x.saturating_add(1);
            for ch in label.chars() {
                let xx = area.x + x;
                if xx >= area.x + area.width {
                    break;
                }
                let cell = &mut buf[(xx, y)];
                cell.set_char(ch);
                cell.set_fg(INK_2);
                cell.set_bg(PANEL_RAISED);
                x = x.saturating_add(1);
            }
        }
    }
    // Inner body rect: between the top rule and the footer, with 2-col left/right margin.
    Rect {
        x: area.x + 2,
        y: area.y + 2,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(3),
    }
}

fn draw_preset_menu(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    draw_modal_dim(f.buffer_mut(), area);
    let menu_area = centered_rect(78, 70, area);
    let inner = draw_modal_card(
        f,
        menu_area,
        "presets",
        &[("j/k", "navigate"), ("enter", "apply"), ("d", "delete"), ("esc", "back")],
        accent,
    );

    let total = app.total_preset_count();
    let visible_rows = inner.height as usize;
    let start = app
        .menu_index
        .saturating_add(1)
        .saturating_sub(visible_rows)
        .min(total.saturating_sub(visible_rows));
    let end = (start + visible_rows).min(total);

    let buf = f.buffer_mut();
    for (row_i, idx) in (start..end).enumerate() {
        let y = inner.y + row_i as u16;
        let selected = idx == app.menu_index;
        let (name, beat, freq_clr, descriptor, kind_label) = if let Some(preset) = PRESETS.get(idx) {
            (
                preset.name.to_string(),
                preset.beat_freq,
                preset.color,
                preset.description.to_string(),
                "built-in",
            )
        } else {
            let local_idx = idx - PRESETS.len();
            let Some(preset) = app.local_presets.get(local_idx) else {
                continue;
            };
            (
                preset.name.clone(),
                preset.beat_freq,
                freq_color(preset.beat_freq),
                preset.short_description(),
                "user",
            )
        };
        let band = freq_band_name(beat);
        draw_preset_row(
            buf,
            inner,
            y,
            idx + 1,
            &name,
            &band,
            beat,
            freq_clr,
            &descriptor,
            kind_label,
            selected,
        );
    }
}

/// One row of the preset menu. Rendered cell-by-cell so the selection
/// gets a luminous left rail, a band-tinted frequency badge, and the
/// description fades on either side of the selected glow.
#[allow(clippy::too_many_arguments)]
fn draw_preset_row(
    buf: &mut Buffer,
    inner: Rect,
    y: u16,
    number: usize,
    name: &str,
    band: &str,
    beat: f32,
    band_color: Color,
    description: &str,
    kind: &str,
    selected: bool,
) {
    if y >= inner.y + inner.height {
        return;
    }
    let row_bg = if selected {
        theme::mix(PANEL, band_color, 0.20)
    } else {
        PANEL
    };
    let text_fg = if selected { INK_0 } else { INK_2 };
    let dim_fg = if selected { INK_1 } else { INK_3 };

    // Paint the row background first.
    for x in 0..inner.width {
        let cell = &mut buf[(inner.x + x, y)];
        cell.set_char(' ');
        cell.set_bg(row_bg);
    }
    // Selection rail (2 cells wide) — luminous accent when selected.
    if selected {
        for dx in 0..2 {
            let cell = &mut buf[(inner.x + dx, y)];
            cell.set_char(if dx == 1 { '\u{2588}' } else { '\u{258C}' });
            cell.set_fg(band_color);
            cell.set_bg(row_bg);
        }
    }

    let mut x = 3u16;
    // Number chip.
    let n = format!("{number:>2}");
    for ch in n.chars() {
        write_cell(buf, inner.x + x, y, ch, dim_fg, row_bg, false);
        x += 1;
    }
    x += 2;
    // Name (bold).
    for ch in name.chars().take(18) {
        write_cell(buf, inner.x + x, y, ch, text_fg, row_bg, true);
        x += 1;
    }
    while x < 24 {
        write_cell(buf, inner.x + x, y, ' ', text_fg, row_bg, false);
        x += 1;
    }
    // Frequency badge — band-colored, dark text.
    let badge = format!(" {beat:.1} Hz · {band} ");
    for ch in badge.chars() {
        if inner.x + x >= inner.x + inner.width {
            break;
        }
        write_cell(buf, inner.x + x, y, ch, SPACE_DEEP, band_color, true);
        x += 1;
    }
    x += 1;
    // Kind tag.
    for ch in kind.chars() {
        if inner.x + x >= inner.x + inner.width {
            break;
        }
        write_cell(buf, inner.x + x, y, ch, dim_fg, row_bg, false);
        x += 1;
    }
    x += 2;
    // Description.
    for ch in description.chars() {
        if inner.x + x >= inner.x + inner.width {
            break;
        }
        write_cell(buf, inner.x + x, y, ch, dim_fg, row_bg, false);
        x += 1;
    }
}

fn write_cell(buf: &mut Buffer, x: u16, y: u16, ch: char, fg: Color, bg: Color, bold: bool) {
    if x >= buf.area.width || y >= buf.area.height {
        return;
    }
    let cell = &mut buf[(x, y)];
    cell.set_char(ch);
    let mut style = Style::default().fg(fg).bg(bg);
    if bold {
        style = style.add_modifier(Modifier::BOLD);
    }
    cell.set_style(style);
}

fn draw_save_preset_prompt(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    draw_modal_dim(f.buffer_mut(), area);
    let prompt_area = centered_rect(64, 30, area);
    let inner = draw_modal_card(
        f,
        prompt_area,
        "save preset",
        &[("type", "name"), ("enter", "save"), ("esc", "cancel")],
        accent,
    );

    let blink_on = (app.start_time.elapsed().as_secs_f64() * 2.0).fract() < 0.55;
    let max_len = 32;
    let input = &app.preset_name_input;
    let count_str = format!("{}/{max_len}", input.chars().count());

    let rows = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(" name  ", Style::default().fg(INK_3)),
            Span::styled(
                input.clone(),
                Style::default().fg(INK_0).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if blink_on { "\u{2588}" } else { " " },
                Style::default().fg(accent),
            ),
            Span::styled(
                format!("   {count_str}"),
                Style::default().fg(INK_4),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" file  ", Style::default().fg(INK_3)),
            Span::styled(
                app.preset_storage_path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "unavailable".to_string()),
                Style::default().fg(INK_2),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(rows), inner);
}

fn draw_sequence_menu(f: &mut Frame, area: Rect, app: &App, accent: Color) {
    draw_modal_dim(f.buffer_mut(), area);
    let menu_area = centered_rect(76, 60, area);
    let inner = draw_modal_card(
        f,
        menu_area,
        "sequences",
        &[("j/k", "navigate"), ("enter", "start"), ("esc", "back")],
        accent,
    );

    let buf = f.buffer_mut();
    for (i, sequence) in SEQUENCES.iter().enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }
        let selected = i == app.menu_index;
        let row_bg = if selected {
            theme::mix(PANEL, SEMANTIC.warm, 0.18)
        } else {
            PANEL
        };
        let text_fg = if selected { INK_0 } else { INK_2 };
        let dim_fg = if selected { INK_1 } else { INK_3 };
        for x in 0..inner.width {
            let cell = &mut buf[(inner.x + x, y)];
            cell.set_char(' ');
            cell.set_bg(row_bg);
        }
        if selected {
            for dx in 0..2 {
                let cell = &mut buf[(inner.x + dx, y)];
                cell.set_char(if dx == 1 { '\u{2588}' } else { '\u{258C}' });
                cell.set_fg(SEMANTIC.warm);
                cell.set_bg(row_bg);
            }
        }
        let mut x = 3u16;
        let n = format!("{:>2}", i + 1);
        for ch in n.chars() {
            write_cell(buf, inner.x + x, y, ch, dim_fg, row_bg, false);
            x += 1;
        }
        x += 2;
        for ch in sequence.name.chars().take(22) {
            write_cell(buf, inner.x + x, y, ch, text_fg, row_bg, true);
            x += 1;
        }
        while x < 28 {
            write_cell(buf, inner.x + x, y, ' ', text_fg, row_bg, false);
            x += 1;
        }
        // Sequence arc — a mini sparkline of beat_freq across the steps.
        let arc_width = 14usize;
        let arc = sequence_arc(sequence, arc_width);
        for (j, ch) in arc.chars().enumerate() {
            if inner.x + x + j as u16 >= inner.x + inner.width {
                break;
            }
            write_cell(buf, inner.x + x + j as u16, y, ch, accent, row_bg, false);
        }
        x += arc_width as u16 + 2;
        // Duration.
        let dur_min = (sequence.total_duration_secs / 60.0).round() as u32;
        let dur = format!("{dur_min}m");
        for ch in dur.chars() {
            write_cell(buf, inner.x + x, y, ch, dim_fg, row_bg, false);
            x += 1;
        }
        x += 2;
        // Description.
        for ch in sequence.description.chars() {
            if inner.x + x >= inner.x + inner.width {
                break;
            }
            write_cell(buf, inner.x + x, y, ch, dim_fg, row_bg, false);
            x += 1;
        }
    }
}

/// 8-glyph sparkline representing a sequence's beat-frequency arc across
/// its steps. Higher beat → taller glyph. Purely decorative; gives each
/// sequence a recognizable silhouette in the menu.
fn sequence_arc(sequence: &crate::presets::Sequence, width: usize) -> String {
    let glyphs = [
        '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    if sequence.steps.is_empty() || width == 0 {
        return " ".repeat(width);
    }
    let max_beat = sequence
        .steps
        .iter()
        .map(|s| s.beat_freq)
        .fold(0.0_f32, f32::max)
        .max(1.0);
    (0..width)
        .map(|i| {
            // Map column to a step (proportional to step duration).
            let t = i as f32 / width as f32;
            let target = t * sequence.total_duration_secs;
            let mut acc = 0.0_f32;
            let mut chosen = sequence.steps[0].beat_freq;
            for step in sequence.steps {
                if target < acc + step.duration_secs {
                    chosen = step.beat_freq;
                    break;
                }
                acc += step.duration_secs;
            }
            let h = (chosen / max_beat).clamp(0.0, 1.0);
            let g = (h * (glyphs.len() - 1) as f32).round() as usize;
            glyphs[g.min(glyphs.len() - 1)]
        })
        .collect()
}

fn draw_help(f: &mut Frame, area: Rect, accent: Color) {
    draw_modal_dim(f.buffer_mut(), area);
    let help_area = centered_rect(82, 80, area);
    let inner = draw_modal_card(f, help_area, "help", &[("any", "back")], accent);

    // Two columns: left = command groups, right = lore.
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(inner);

    draw_help_commands(f, cols[0], accent);
    draw_help_lore(f, cols[1], accent);
}

fn draw_help_commands(f: &mut Frame, area: Rect, accent: Color) {
    let groups: &[(&str, Color, &[(&str, &str)])] = &[
        (
            "navigation",
            accent,
            &[
                ("j/k", "choose parameter"),
                ("h/l", "adjust value"),
                ("H/L", "coarse adjust (×5)"),
                ("space", "play / pause"),
            ],
        ),
        (
            "programs",
            SEMANTIC.warm,
            &[
                ("1–5", "quick preset"),
                ("p", "preset menu"),
                ("S", "save preset"),
                ("s", "sequence menu"),
                ("d", "delete local preset"),
            ],
        ),
        (
            "spectacle",
            SEMANTIC.life,
            &[
                ("v / V", "next / prev visual"),
                ("e", "toggle emergence"),
                ("g", "spawn mode"),
                ("r / R", "drift / reverse"),
                ("n / m", "mist / cycle"),
                ("t", "cycle timbre"),
            ],
        ),
        (
            "session",
            SEMANTIC.gain,
            &[
                ("tab", "swap to Knowledge"),
                ("?", "help"),
                ("q / esc", "quit"),
            ],
        ),
    ];

    let mut lines: Vec<Line<'static>> = Vec::with_capacity(64);
    for (title, color, entries) in groups {
        lines.push(Line::from(vec![
            Span::styled(
                format!(" {title} "),
                Style::default()
                    .fg(SPACE_DEEP)
                    .bg(*color)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        for (key, descr) in *entries {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("   {:<8}", *key),
                    Style::default().fg(*color).add_modifier(Modifier::BOLD),
                ),
                Span::styled((*descr).to_string(), Style::default().fg(INK_1)),
            ]));
        }
        lines.push(Line::from(""));
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_help_lore(f: &mut Frame, area: Rect, accent: Color) {
    let lore = vec![
        Line::from(vec![Span::styled(
            " lore ",
            Style::default()
                .fg(SPACE_DEEP)
                .bg(accent)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(Span::styled(
            "Emergence voices spawn by simple",
            Style::default().fg(INK_1),
        )),
        Line::from(Span::styled(
            "harmonic ratios, mutate gently, and",
            Style::default().fg(INK_1),
        )),
        Line::from(Span::styled(
            "fade by consonance.",
            Style::default().fg(INK_1),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Harmonics view maps the live stereo",
            Style::default().fg(INK_1),
        )),
        Line::from(Span::styled(
            "phase trace and the active H1–H6 timbre",
            Style::default().fg(INK_1),
        )),
        Line::from(Span::styled(
            "partials.",
            Style::default().fg(INK_1),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Penrose mode walks a Fibonacci-word",
            Style::default().fg(INK_1),
        )),
        Line::from(Span::styled(
            "Conway worm; tile pairs (LL/LS/SL)",
            Style::default().fg(INK_1),
        )),
        Line::from(Span::styled(
            "pick 3:2, 5:4, 4:3 intervals.",
            Style::default().fg(INK_1),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Drift adds a Shepard-Risset glissando:",
            Style::default().fg(INK_1),
        )),
        Line::from(Span::styled(
            "seven octaves under a raised-cosine",
            Style::default().fg(INK_1),
        )),
        Line::from(Span::styled(
            "bell, endlessly rising or falling.",
            Style::default().fg(INK_1),
        )),
    ];
    f.render_widget(
        Paragraph::new(lore).wrap(Wrap { trim: false }),
        area,
    );
}

/// Parameters that drive the living backdrop. Threading these through a
/// single struct keeps the call sites readable when more signals get added.
#[derive(Clone, Copy)]
struct BackdropCtx {
    elapsed: f64,
    accent: Color,
    /// 0..~1 — current audio RMS, used to scale the haze intensity.
    rms: f32,
    /// Beat frequency in Hz — scales aurora drift speed (alpha drifts slowly,
    /// gamma moves with momentum).
    beat_freq: f32,
}

fn backdrop_ctx(app: &App, accent: Color) -> BackdropCtx {
    BackdropCtx {
        elapsed: app.start_time.elapsed().as_secs_f64(),
        accent,
        rms: 0.5 * (app.signals.rms_l + app.signals.rms_r),
        beat_freq: app.params.get_beat_freq(),
    }
}

impl BackdropCtx {
    fn drift_speed(self) -> f64 {
        // ~0.5 at delta (2Hz), ~1.0 at alpha (10Hz), ~1.6 at gamma (40Hz).
        let scale = (self.beat_freq.max(0.5) / 10.0).log2() * 0.35 + 1.0;
        scale.clamp(0.45, 1.7) as f64
    }
}

/// Shell-level backdrop: deep gradient + slow aurora curtains + sparse stars.
/// The curtains drift horizontally; their tint is a desaturated accent and
/// their intensity is gated by a vertical falloff so they bunch toward the
/// midline like real aurora.
fn draw_backdrop(buf: &mut Buffer, area: Rect, ctx: BackdropCtx) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let drift = ctx.drift_speed();
    let aurora_tint = theme::dim(ctx.accent, 0.55);
    let haze = ctx.rms.clamp(0.0, 1.0);

    for y in 0..area.height {
        let ty = y as f32 / area.height.max(1) as f32;
        let base = vertical_gradient(ty);
        // Two stacked curtains at different horizontal frequencies and
        // vertical falloffs. Curtain A sits high and broad; B sits lower
        // and tighter — they cross around the midline.
        let fall_a = 1.0 - ((ty - 0.30).abs() / 0.45).clamp(0.0, 1.0);
        let fall_b = 1.0 - ((ty - 0.62).abs() / 0.55).clamp(0.0, 1.0);
        let fall_a = fall_a * fall_a;
        let fall_b = fall_b * fall_b;

        for x in 0..area.width {
            let xf = x as f64;
            // Curtain intensity in 0..1 — sin wave drifting along x, time
            // advances at the beat-frequency-modulated drift speed.
            let curt_a = ((xf * 0.045 + ctx.elapsed * 0.18 * drift).sin() * 0.5 + 0.5) as f32;
            let curt_b = ((xf * 0.028 + ctx.elapsed * 0.10 * drift + 1.7).sin() * 0.5 + 0.5) as f32;
            let aurora = (curt_a * fall_a * 0.22 + curt_b * fall_b * 0.18).min(0.40);
            let bg = if aurora > 0.0 {
                theme::mix(base, aurora_tint, aurora)
            } else {
                base
            };

            let cell = &mut buf[(area.x + x, area.y + y)];
            cell.set_char(' ');
            cell.set_bg(bg);

            // Star field — deterministic positions, slow twinkle. Roughly
            // one star per 220 cells — sparse enough to feel like distance.
            let star_hash = x.wrapping_mul(73).wrapping_add(y.wrapping_mul(151));
            if star_hash % 211 == 0 {
                let twinkle =
                    ((ctx.elapsed * 1.3 + x as f64 * 0.7 + y as f64 * 1.1).sin() * 0.5 + 0.5) as f32;
                let luminance = 0.30 + twinkle * 0.55;
                let glyph = if star_hash % 633 == 0 {
                    '\u{2217}' // ∗  — a few brighter stars
                } else if star_hash % 422 == 0 {
                    '\u{2219}' // ∙
                } else {
                    '\u{00B7}' // ·
                };
                cell.set_char(glyph);
                cell.set_fg(theme::dim(INK_1, luminance));
            } else if haze > 0.05 {
                // RMS-reactive haze — sparse dim particles that thicken
                // with the music. Independent hash so it never collides
                // with the star field.
                let haze_hash = x
                    .wrapping_mul(29)
                    .wrapping_add(y.wrapping_mul(53))
                    .wrapping_add((ctx.elapsed * 4.0) as u16);
                let gate = (90.0 - haze * 75.0).max(20.0) as u16;
                if haze_hash % gate == 0 {
                    cell.set_char('\u{00B7}');
                    cell.set_fg(theme::dim(ctx.accent, 0.18 + haze * 0.20));
                }
            }
        }
    }
}

/// Vertical gradient stop used by both the shell backdrop and the modal
/// wash. Pure function — same input always yields the same color.
fn vertical_gradient(ty: f32) -> Color {
    if ty < 0.50 {
        theme::mix(SPACE_DEEP, SPACE_MID, ty / 0.50)
    } else {
        theme::mix(SPACE_MID, HORIZON, (ty - 0.50) / 0.50)
    }
}

/// In-panel backdrop for the visualization stage. Quieter than the shell
/// backdrop — the visualization itself is the focal element, not the field
/// it sits in. A horizontal sweep of accent glow tracks the audio energy.
fn draw_visual_backdrop(buf: &mut Buffer, area: Rect, ctx: BackdropCtx) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    let drift = ctx.drift_speed();
    let haze = ctx.rms.clamp(0.0, 1.0);
    let glow_tint = theme::dim(ctx.accent, 0.42);

    for y in 0..area.height {
        let ty = y as f32 / area.height.max(1) as f32;
        let panel = theme::mix(PANEL, PANEL_RAISED, ty * 0.55);
        // Soft horizontal sweep — a low-amplitude sin moving slowly across
        // the panel, brighter on the audio's loud passages.
        let sweep_falloff = (1.0 - (ty - 0.45).abs() * 1.6).max(0.0);
        for x in 0..area.width {
            let xf = x as f64;
            let sweep = ((xf * 0.06 - ctx.elapsed * 0.22 * drift).sin() * 0.5 + 0.5) as f32;
            let intensity = sweep * sweep_falloff * (0.10 + haze * 0.18);
            let bg = if intensity > 0.0 {
                theme::mix(panel, glow_tint, intensity.min(0.45))
            } else {
                panel
            };
            let cell = &mut buf[(area.x + x, area.y + y)];
            cell.set_char(' ');
            cell.set_bg(bg);
        }
    }
}

fn draw_spectrum_floor(buf: &mut Buffer, area: Rect, accent: Color) {
    if area.height < 2 {
        return;
    }
    // Width-adaptive labels: long names need ~36 cells to avoid colliding;
    // narrower stages get short names; very narrow gets single letters.
    let labels: &[(&str, u16)] = if area.width >= 40 {
        &[
            ("Delta", 1),
            ("Theta", 2),
            ("Alpha", 4),
            ("Beta", 7),
            ("Gamma", 11),
        ]
    } else if area.width >= 24 {
        &[
            ("Δ", 1),
            ("θ", 2),
            ("α", 4),
            ("β", 7),
            ("γ", 11),
        ]
    } else {
        &[]
    };
    let y = area.height.saturating_sub(1);
    for (label, position) in labels {
        let x = area.width.saturating_mul(*position) / 12;
        write_str(
            buf,
            area,
            x,
            y,
            label,
            Style::default().fg(dim_color(accent, 0.56)),
        );
    }
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

fn meter_line(
    spec: MeterSpec,
    active: ActiveParam,
    width: usize,
    since_adjust: Option<f32>,
) -> Line<'static> {
    let is_active = spec.param == active;
    // Afterglow: spike for ~600ms after adjustment, exponentially decays.
    // Multiplicative bonus on top of the per-cell luminance ramp.
    let glow = since_adjust
        .map(|t| (1.0 - (t / 0.6).min(1.0)).powf(1.5))
        .unwrap_or(0.0);
    let bar_bg = if is_active {
        theme::mix(PANEL, PANEL_RAISED, 0.85)
    } else {
        PANEL
    };

    let mut spans = Vec::with_capacity(4 + width);

    // Selection marker — accent-glow when active, breath-quiet when not.
    spans.push(Span::styled(
        if is_active { " \u{25B8}" } else { "  " },
        Style::default().fg(spec.color).bg(bar_bg),
    ));
    spans.push(Span::styled(
        format!(" {:<5}", spec.label),
        Style::default()
            .fg(if is_active { INK_0 } else { INK_1 })
            .bg(bar_bg)
            .add_modifier(if is_active {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
    ));
    spans.push(Span::styled(
        format!(" {:>8} ", spec.value),
        Style::default()
            .fg(if is_active { INK_1 } else { INK_3 })
            .bg(bar_bg),
    ));
    spans.extend(meter_bar(spec.ratio, width, spec.color, glow, bar_bg));
    Line::from(spans)
}

fn parameter_meter_width(panel_width: u16) -> usize {
    (panel_width as usize)
        .saturating_sub(METER_LINE_FIXED_WIDTH)
        .max(1)
}

fn shepard_base_ratio(freq: f32) -> f32 {
    let min = MIN_BASE_FREQ_HZ as f32;
    let max = MAX_BASE_FREQ_HZ as f32;
    ((freq / min).log2() / (max / min).log2()).clamp(0.0, 1.0)
}

/// Per-cell gradient meter bar.
///
/// Filled cells ramp from a calm base luminance on the left to a glowing
/// tip on the right, with an extra brightness bonus applied across the
/// whole filled section while afterglow is active. Unfilled cells fade to
/// a deep veil of the same hue so the meter still "reads" as one form.
fn meter_bar(
    value: f32,
    width: usize,
    color: Color,
    afterglow: f32,
    bg: Color,
) -> Vec<Span<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let value = value.clamp(0.0, 1.0);
    let filled_f = value * width as f32;
    let filled = filled_f.round() as usize;
    let mut spans: Vec<Span<'static>> = Vec::with_capacity(width);

    for i in 0..filled {
        // Position 0..1 along the filled portion.
        let t = if filled <= 1 {
            1.0
        } else {
            i as f32 / (filled - 1) as f32
        };
        // Base luminance ramp: 0.55 at root → 1.00 at tip.
        let mut lum = 0.55 + t * 0.45;
        // Afterglow boost — biggest at the tip, falls toward the root.
        lum += afterglow * (0.18 + t * 0.30);
        spans.push(Span::styled(
            "\u{2588}".to_string(),
            Style::default().fg(theme::dim(color, lum.min(1.35))).bg(bg),
        ));
    }
    let empty = width.saturating_sub(filled);
    for _ in 0..empty {
        spans.push(Span::styled(
            "\u{2591}".to_string(),
            Style::default().fg(theme::dim(color, 0.16)).bg(bg),
        ));
    }
    spans
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

fn panel_block(title: &'static str, border: Color) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(PANEL_BG))
        .title(Span::styled(title, Style::default().fg(DIM)))
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

// Color math now lives in `crate::theme`. Thin wrappers keep call sites
// readable during the rework; they will be dropped in later phases as
// renderers switch to AccentRamp / SemanticPalette directly.
fn breathing_color(color: Color, elapsed: f64) -> Color {
    theme::breathing(color, elapsed)
}

fn dim_color(color: Color, factor: f32) -> Color {
    theme::dim(color, factor)
}

fn shift_color(color: Color, dr: u8, dg: u8, db: u8) -> Color {
    theme::shift(color, dr, dg, db)
}

fn blend_color(start: Color, end: Color, amount: f32) -> Color {
    theme::mix(start, end, amount)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parameter_meter_line_fills_available_width() {
        let width = 42;
        let meter_width = parameter_meter_width(width);
        let line = meter_line(
            MeterSpec {
                param: ActiveParam::Volume,
                label: "gain",
                value: " 50%".to_string(),
                ratio: 0.5,
                color: SEMANTIC.gain,
            },
            ActiveParam::Volume,
            meter_width,
            None,
        );

        assert_eq!(meter_width, width as usize - METER_LINE_FIXED_WIDTH);
        assert_eq!(line.width(), width as usize);
    }
}
