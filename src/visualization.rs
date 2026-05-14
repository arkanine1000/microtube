use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::emergence::{EmergenceSnapshot, SpawnMode};
use crate::penrose::Tile;

pub const HARMONIC_PARTIAL_LABELS: [(&str, &str); 6] = [
    ("H1", "root"),
    ("H2", "oct"),
    ("H3", "5th"),
    ("H4", "oct"),
    ("H5", "3rd"),
    ("H6", "5th"),
];

/// Render a waveform using braille characters into a ratatui Buffer.
pub fn render_braille_waveform(buf: &mut Buffer, area: Rect, samples: &[f32], color: Color) {
    if area.width == 0 || area.height == 0 || samples.is_empty() {
        return;
    }

    let dots_w = area.width as usize * 2;
    let dots_h = area.height as usize * 4;
    let center_y = dots_h / 2;

    let step = samples.len() as f64 / dots_w as f64;
    let mut cells = vec![0u8; area.width as usize * area.height as usize];

    for x in 0..dots_w {
        let sample_idx = ((x as f64 * step) as usize).min(samples.len() - 1);
        let sample = samples[sample_idx];

        let y = center_y as f64 - sample as f64 * (center_y as f64 * 0.9);
        let y = y.round().clamp(0.0, (dots_h - 1) as f64) as usize;

        set_braille_dot(&mut cells, area.width as usize, x, y);
        if y > 0 {
            set_braille_dot(&mut cells, area.width as usize, x, y - 1);
        }
        if y + 1 < dots_h {
            set_braille_dot(&mut cells, area.width as usize, x, y + 1);
        }
    }

    for cy in 0..area.height as usize {
        for cx in 0..area.width as usize {
            let code = cells[cy * area.width as usize + cx];
            if code != 0 {
                let ch = char::from_u32(0x2800 + code as u32).unwrap_or(' ');
                let cell = &mut buf[(area.x + cx as u16, area.y + cy as u16)];
                cell.set_char(ch);
                cell.set_fg(color);
            }
        }
    }
}

fn set_braille_dot(cells: &mut [u8], cell_width: usize, dot_x: usize, dot_y: usize) {
    let cell_x = dot_x / 2;
    let cell_y = dot_y / 4;
    let idx = cell_y * cell_width + cell_x;
    if idx >= cells.len() {
        return;
    }

    let bit = match (dot_x % 2, dot_y % 4) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (0, _) => 0x40,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (1, _) => 0x80,
        _ => 0,
    };
    cells[idx] |= bit;
}

/// Render spectrum bars (cava-style) with gravity falloff.
pub fn render_spectrum_bars(
    buf: &mut Buffer,
    area: Rect,
    samples: &[f32],
    bars: &mut Vec<f32>,
    color: Color,
) {
    if area.width == 0 || area.height == 0 || samples.is_empty() {
        return;
    }

    let num_bars = area.width as usize;
    bars.resize(num_bars, 0.0);

    let bin_size = samples.len() / num_bars;
    if bin_size == 0 {
        return;
    }

    for (i, bar) in bars.iter_mut().enumerate().take(num_bars) {
        let start = i * bin_size;
        let end = (start + bin_size).min(samples.len());
        let energy: f32 = samples[start..end].iter().map(|s| s * s).sum::<f32>() / bin_size as f32;
        let amplitude = energy.sqrt() * 3.0;

        if amplitude > *bar {
            *bar = amplitude;
        } else {
            *bar *= 0.92;
        }
    }

    let block_chars = [
        ' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    let height = area.height as f32;
    let (cr, cg, cb) = rgb_from_color(color);

    for (i, &bar_val) in bars.iter().enumerate().take(area.width as usize) {
        let bar_height = (bar_val * height).min(height);
        let full_cells = bar_height as usize;
        let frac = bar_height - full_cells as f32;
        let frac_idx = (frac * 8.0) as usize;

        for row in 0..area.height as usize {
            let y = area.height as usize - 1 - row;
            let cell = &mut buf[(area.x + i as u16, area.y + y as u16)];

            let gradient_t = row as f32 / height;
            let r = (cr as f32 * gradient_t + 60.0 * (1.0 - gradient_t)) as u8;
            let g = (cg as f32 * gradient_t + 30.0 * (1.0 - gradient_t)) as u8;
            let b = (cb as f32 * (1.0 - gradient_t * 0.5) + 100.0 * gradient_t) as u8;
            let grad_color = Color::Rgb(r, g, b);

            if row < full_cells {
                cell.set_char('\u{2588}');
                cell.set_fg(grad_color);
            } else if row == full_cells && frac_idx > 0 {
                cell.set_char(block_chars[frac_idx]);
                cell.set_fg(grad_color);
            }
        }
    }
}

pub struct HarmonicLattice<'a> {
    pub samples_l: &'a [f32],
    pub samples_r: &'a [f32],
    pub elapsed: f64,
    pub beat_freq: f64,
    pub harmonics: f64,
    pub harmonic_weights: [f64; 5],
    pub color: Color,
}

/// Render the live stereo phase trace plus the actual H1-H6 partial stack
/// produced by the current timbre/warmth settings.
pub fn render_harmonic_lattice(buf: &mut Buffer, area: Rect, lattice: HarmonicLattice<'_>) {
    if area.width < 8 || area.height < 4 {
        return;
    }

    let samples_l = lattice.samples_l;
    let samples_r = lattice.samples_r;
    let elapsed = lattice.elapsed;
    let beat_freq = lattice.beat_freq;
    let harmonics = lattice.harmonics;
    let partial_levels = harmonic_partial_levels(harmonics, lattice.harmonic_weights);
    let max_level = partial_levels
        .iter()
        .copied()
        .fold(0.0_f64, f64::max)
        .max(f64::EPSILON);
    let color = lattice.color;

    let floor_height = if area.height >= 12 { 3 } else { 0 };
    let plot_height = area.height.saturating_sub(floor_height).max(1);
    let w = area.width as f64;
    let h = plot_height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let rx = w * 0.38;
    let ry = h * 0.34;

    let beat_phase = (std::f64::consts::PI * beat_freq * elapsed).cos().abs();
    let pulse = 0.86 + 0.14 * beat_phase;
    let rotation = elapsed * 0.11;
    let (cr, cg, cb) = rgb_from_color(color);

    // 1. Reference rings for the phase portrait.
    for ring in 1..=3 {
        let ring_scale = ring as f64 / 3.6 * pulse;
        let ring_color = Color::Rgb(
            (cr as f32 * 0.18) as u8,
            (cg as f32 * 0.18) as u8,
            (cb as f32 * 0.22) as u8,
        );
        let steps = (area.width as usize).saturating_mul(4).max(48);
        for i in (0..steps).step_by(2) {
            let theta = i as f64 / steps as f64 * std::f64::consts::TAU + rotation;
            let x = cx + theta.cos() * rx * ring_scale;
            let y = cy + theta.sin() * ry * ring_scale;
            put_braille(buf, area, x, y, ring_color);
        }
    }

    let mut positions = [(0.0, 0.0); 6];
    for partial in 1..=6 {
        positions[partial - 1] = harmonic_node_position(partial, cx, cy, rx, ry, rotation, pulse);
    }

    // 2. Connections show active integer partial relationships, not fixed ratios.
    for idx in 1..positions.len() {
        let relative = (partial_levels[idx] / max_level).sqrt() as f32;
        if relative <= 0.02 {
            continue;
        }
        let strength = 0.10 + relative * 0.48 + beat_phase as f32 * 0.10;
        let line_color = Color::Rgb(
            (cr as f32 * strength) as u8,
            (cg as f32 * strength) as u8,
            (cb as f32 * strength) as u8,
        );
        draw_line_braille(buf, area, positions[0], positions[idx], line_color);
    }

    for i in 1..positions.len() {
        for j in (i + 1)..positions.len() {
            let pair_level = (partial_levels[i] * partial_levels[j]).sqrt() / max_level;
            if pair_level <= 0.03 {
                continue;
            }
            let ratio = (j + 1) as f32 / (i + 1) as f32;
            if is_near_simple_ratio(ratio) {
                let strength = 0.08 + pair_level as f32 * 0.34;
                let line_color = Color::Rgb(
                    (cr as f32 * strength) as u8,
                    (cg as f32 * strength) as u8,
                    (cb as f32 * strength) as u8,
                );
                draw_line_braille(buf, area, positions[i], positions[j], line_color);
            }
        }
    }

    // 3. Live L/R Lissajous trace from the visualization buffer.
    let sample_count = samples_l.len().min(samples_r.len());
    if sample_count > 1 {
        let max_points = 800usize;
        let step = (sample_count / max_points).max(1);
        let point_total = sample_count.div_ceil(step).max(1);
        let theta = rotation * 0.7;
        let cos_t = theta.cos();
        let sin_t = theta.sin();

        for (point_idx, sample_idx) in (0..sample_count).step_by(step).enumerate() {
            let l = samples_l[sample_idx].clamp(-1.0, 1.0) as f64;
            let r = samples_r[sample_idx].clamp(-1.0, 1.0) as f64;
            let x = l * rx * 0.62;
            let y = r * ry * 0.78;
            let rotated_x = x * cos_t - y * sin_t;
            let rotated_y = x * sin_t + y * cos_t;

            let age = point_idx as f32 / point_total as f32;
            let harmonic_glow = harmonics as f32;
            let brightness = (0.34 + age * 0.42 + harmonic_glow * 0.28).min(1.0);
            let sample_color = Color::Rgb(
                (cr as f32 * brightness).max(35.0) as u8,
                (cg as f32 * (0.65 + brightness * 0.35)).max(35.0) as u8,
                (cb as f32 * (0.65 + harmonic_glow * 0.35)).max(45.0) as u8,
            );

            // Draw as braille instead of coarse characters
            put_braille(buf, area, cx + rotated_x, cy + rotated_y, sample_color);
        }
    }

    // 4. Partial nodes. Digits are intentionally compact: H1-H6 are shown
    // in the floor/panel, while the node strength shows current amplitude.
    for (idx, &(x, y)) in positions.iter().enumerate() {
        let relative = (partial_levels[idx] / max_level).sqrt();
        let node_gain = if idx == 0 {
            0.72 + relative * 0.34
        } else {
            0.18 + relative * 0.82
        };
        let node_color = Color::Rgb(
            (cr as f64 * node_gain).max(45.0) as u8,
            (cg as f64 * node_gain).max(40.0) as u8,
            (cb as f64 * node_gain).max(45.0) as u8,
        );
        let ch = char::from_digit((idx + 1) as u32, 10).unwrap_or('\u{25CF}');
        put_cell(buf, area, x, y, ch, node_color);

        if area.width > 54 && plot_height > 9 {
            draw_text(
                buf,
                area,
                x + 1.0,
                y,
                HARMONIC_PARTIAL_LABELS[idx].0,
                node_color,
            );
        }
    }

    if floor_height >= 3 {
        draw_partial_floor(buf, area, plot_height, &partial_levels, max_level, color);
    }
}

pub fn harmonic_partial_levels(harmonics: f64, harmonic_weights: [f64; 5]) -> [f64; 6] {
    let warmth = harmonics.clamp(0.0, 1.0);
    let harmonic_weights = harmonic_weights.map(|weight| weight.max(0.0));
    let total_weight = harmonic_weights.iter().sum::<f64>();
    let norm = 1.0 + warmth * total_weight;

    let mut levels = [0.0; 6];
    levels[0] = 1.0 / norm;
    for (idx, weight) in harmonic_weights.into_iter().enumerate() {
        levels[idx + 1] = weight * warmth / norm;
    }
    levels
}

fn harmonic_node_position(
    partial: usize,
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    rotation: f64,
    pulse: f64,
) -> (f64, f64) {
    if partial == 1 {
        return (cx, cy);
    }

    let log_partial = (partial as f64).log2();
    let log_span = 6.0_f64.log2();
    let radius = (0.24 + log_partial / log_span * 0.70) * pulse;
    let angle = rotation + log_partial * std::f64::consts::TAU * 0.62 - std::f64::consts::FRAC_PI_2;

    (
        cx + angle.cos() * rx * radius,
        cy + angle.sin() * ry * radius,
    )
}

fn draw_partial_floor(
    buf: &mut Buffer,
    area: Rect,
    start_y: u16,
    levels: &[f64; 6],
    max_level: f64,
    color: Color,
) {
    if start_y + 2 >= area.height || area.width == 0 {
        return;
    }

    let (cr, cg, cb) = rgb_from_color(color);
    let divider_color = Color::Rgb(
        (cr as f32 * 0.18) as u8,
        (cg as f32 * 0.18) as u8,
        (cb as f32 * 0.22) as u8,
    );
    for x in 0..area.width {
        let cell = &mut buf[(area.x + x, area.y + start_y)];
        cell.set_char('\u{2500}');
        cell.set_fg(divider_color);
    }

    let bar_y = start_y + 1;
    let label_y = start_y + 2;
    for (idx, (id, _)) in HARMONIC_PARTIAL_LABELS.iter().enumerate() {
        let x0 = idx as u16 * area.width / 6;
        let x1 = ((idx as u16 + 1) * area.width / 6).saturating_sub(1);
        let segment_width = x1.saturating_sub(x0).max(1);
        let bar_width = segment_width.saturating_sub(2).max(1) as usize;
        let relative = (levels[idx] / max_level).clamp(0.0, 1.0);
        let filled = (relative * bar_width as f64).round() as usize;
        let color_gain = 0.24 + relative as f32 * 0.76;
        let bar_color = Color::Rgb(
            (cr as f32 * color_gain).max(30.0) as u8,
            (cg as f32 * color_gain).max(30.0) as u8,
            (cb as f32 * color_gain).max(35.0) as u8,
        );

        for offset in 0..bar_width {
            let x = x0 + 1 + offset as u16;
            if x >= area.width {
                break;
            }
            let cell = &mut buf[(area.x + x, area.y + bar_y)];
            cell.set_char(if offset < filled {
                '\u{2584}'
            } else {
                '\u{00B7}'
            });
            cell.set_fg(bar_color);
        }

        let label_x = x0 + segment_width.saturating_sub(id.len() as u16) / 2;
        draw_text(buf, area, label_x as f64, label_y as f64, id, bar_color);
    }
}

/// Render beat envelope visualization.
pub fn render_beat_envelope(
    buf: &mut Buffer,
    area: Rect,
    elapsed: f64,
    beat_freq: f64,
    color: Color,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let block_chars = [
        ' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
        '\u{2588}',
    ];
    let (cr, cg, cb) = rgb_from_color(color);

    for x in 0..area.width as usize {
        let t = elapsed + (x as f64 * 0.02);
        let envelope = (std::f64::consts::PI * beat_freq * t).cos().abs();
        let bar_height = envelope * area.height as f64;
        let full_cells = bar_height as usize;
        let frac = bar_height - full_cells as f64;
        let frac_idx = (frac * 8.0) as usize;

        for row in 0..area.height as usize {
            let y = area.height as usize - 1 - row;
            let cell = &mut buf[(area.x + x as u16, area.y + y as u16)];

            let intensity = (row as f32 / area.height as f32 * 255.0) as u8;
            let r = (cr as u16 * intensity as u16 / 255) as u8;
            let g = (cg as u16 * intensity as u16 / 255) as u8;
            let b = (cb as u16 * intensity as u16 / 255) as u8;
            let grad_color = Color::Rgb(r, g, b);

            if row < full_cells {
                cell.set_char('\u{2588}');
                cell.set_fg(grad_color);
            } else if row == full_cells && frac_idx > 0 {
                cell.set_char(block_chars[frac_idx]);
                cell.set_fg(grad_color);
            }
        }
    }
}

/// Penrose-inspired geometric visualization using braille sub-cell vectors.
pub fn render_penrose(buf: &mut Buffer, area: Rect, elapsed: f64, beat_freq: f64, color: Color) {
    if area.width < 4 || area.height < 4 {
        return;
    }

    let w = area.width as f64;
    let h = area.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;

    let phi: f64 = 1.618033988749895;
    let beat_phase = (std::f64::consts::PI * beat_freq * elapsed).cos();
    let pulse = 0.75 + 0.25 * beat_phase.abs();
    let rotation = elapsed * 0.3;

    let (cr, cg, cb) = rgb_from_color(color);

    // 1. Draw smooth golden spiral
    let spiral_points = 180;
    for i in 0..spiral_points {
        let t = i as f64 / spiral_points as f64;
        let spiral_angle = t * std::f64::consts::TAU * 3.0 + rotation * phi;
        let spiral_r = t * cy * 0.85 * pulse;

        let sx = cx + spiral_r * spiral_angle.cos() * 2.0;
        let sy = cy + spiral_r * spiral_angle.sin();

        let brightness =
            (0.5 + 0.5 * (t * std::f64::consts::TAU * beat_freq + elapsed).sin()) as f32;
        let r = (cr as f32 * brightness * 0.9) as u8;
        let g = (cg as f32 * brightness * 0.9) as u8;
        let b = (cb as f32 * brightness * 0.9) as u8;

        put_braille(buf, area, sx, sy, Color::Rgb(r, g, b));
    }

    // 2. Draw vector polygons
    let layers = 5;
    for layer in 0..layers {
        let base_radius = (layer as f64 + 1.0) / layers as f64;
        let radius = base_radius * pulse * (cy.min(cx * 0.5));
        let sides = if layer % 2 == 0 { 5 } else { 10 };
        let layer_rotation = rotation + (layer as f64) * std::f64::consts::PI / (phi * 5.0);
        let brightness = 1.0 - (layer as f64 / layers as f64) * 0.4;
        let dim_beat = brightness * (0.65 + 0.35 * beat_phase.abs());

        let r = (cr as f64 * dim_beat) as u8;
        let g = (cg as f64 * dim_beat) as u8;
        let b = (cb as f64 * dim_beat) as u8;
        let layer_color = Color::Rgb(r, g, b);

        for i in 0..sides {
            let angle = layer_rotation + (i as f64 * std::f64::consts::TAU / sides as f64);
            let x = cx + radius * angle.cos() * 2.0;
            let y = cy + radius * angle.sin();

            let next_angle =
                layer_rotation + ((i + 1) as f64 * std::f64::consts::TAU / sides as f64);
            let nx = cx + radius * next_angle.cos() * 2.0;
            let ny = cy + radius * next_angle.sin();

            // Draw solid braille line for polygon edges
            draw_line_braille(buf, area, (x, y), (nx, ny), layer_color);

            // Draw vertex
            let ch = match layer % 5 {
                0 => '\u{25C6}',
                1 => '\u{25CB}',
                2 => '\u{25B2}',
                3 => '\u{2B21}',
                _ => '\u{25CF}',
            };
            put_cell(buf, area, x, y, ch, layer_color);
        }
    }
}

/// Emergence visualization: a living constellation of harmonic voices.
pub fn render_emergence(
    buf: &mut Buffer,
    area: Rect,
    snapshot: &EmergenceSnapshot,
    elapsed: f64,
    color: Color,
) {
    if area.width < 6 || area.height < 4 {
        return;
    }

    let status_height = if area.height >= 8 { 2 } else { 1 };
    let field_height = area.height.saturating_sub(status_height).max(1);
    let w = area.width as f64;
    let h = field_height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let (cr, cg, cb) = rgb_from_color(color);

    // 1. Draw connections using braille lines
    for (i, v1) in snapshot.voices.iter().enumerate() {
        for v2 in snapshot.voices.iter().skip(i + 1) {
            let ratio = if v1.freq_ratio > v2.freq_ratio {
                v1.freq_ratio / v2.freq_ratio
            } else {
                v2.freq_ratio / v1.freq_ratio
            };

            let is_consonant = is_near_simple_ratio(ratio);
            if !is_consonant {
                continue;
            }

            let (x1, y1) = voice_position(v1.freq_ratio, v1.pan, cx, cy, elapsed);
            let (x2, y2) = voice_position(v2.freq_ratio, v2.pan, cx, cy, elapsed);

            let connection_brightness = (v1.amplitude * v2.amplitude).min(1.0) * 0.8;
            let r = (cr as f32 * connection_brightness * 0.5) as u8;
            let g = (cg as f32 * connection_brightness * 0.5) as u8;
            let b = (cb as f32 * connection_brightness * 0.5) as u8;
            let line_color = Color::Rgb(r.max(30), g.max(30), b.max(30));

            draw_line_braille(buf, area, (x1, y1), (x2, y2), line_color);
        }
    }

    // 2. Draw each voice as a node on top of connections
    for voice in &snapshot.voices {
        let (vx, vy) = voice_position(voice.freq_ratio, voice.pan, cx, cy, elapsed);

        let amp = voice.amplitude.min(1.0);
        let pulse = 0.8 + 0.2 * ((elapsed * 2.0 + voice.freq_ratio as f64 * 3.0).sin() as f32);
        let life = (1.0 - (voice.age_normalized * 2.0 - 1.0).abs()).clamp(0.0, 1.0);
        let brightness = (0.16 + amp * 0.78 + life * 0.06) * pulse;

        let r = (cr as f32 * brightness).max(28.0) as u8;
        let g = (cg as f32 * brightness).max(24.0) as u8;
        let b = (cb as f32 * brightness).max(30.0) as u8;
        let voice_color = Color::Rgb(r, g, b);

        let ch = match voice.generation {
            0 => '\u{2736}', // Six-pointed star (root)
            1 => '\u{25C9}', // Fisheye
            2 => '\u{25CF}', // Filled circle
            3 => '\u{25CB}', // Circle
            4 => '\u{25E6}', // Small circle
            5 => '\u{2022}', // Bullet
            _ => '\u{00B7}', // Dot
        };

        // Halo for loud voices
        if amp > 0.3 {
            let halo_brightness = (0.1 + (amp - 0.3) * 0.6).min(1.0);
            let hr = (cr as f32 * halo_brightness) as u8;
            let hg = (cg as f32 * halo_brightness) as u8;
            let hb = (cb as f32 * halo_brightness) as u8;
            let halo_color = Color::Rgb(hr.max(35), hg.max(35), hb.max(35));

            // Circular braille halo
            let halo_radius = 1.8;
            for step in 0..12 {
                let angle = step as f64 * std::f64::consts::TAU / 12.0;
                let hx = vx + angle.cos() * halo_radius * 2.0;
                let hy = vy + angle.sin() * halo_radius;
                put_braille(buf, area, hx, hy, halo_color);
            }
        }

        put_cell(buf, area, vx, vy, ch, voice_color);
    }

    draw_emergence_status(buf, area, snapshot, field_height, color);
}

/// Map a voice's frequency ratio to a screen position.
fn voice_position(freq_ratio: f32, pan: f32, cx: f64, cy: f64, elapsed: f64) -> (f64, f64) {
    let octave = (freq_ratio.max(0.001) as f64).log2();
    let radius = (0.22 + (octave.abs() / 2.0).min(1.0) * 0.62).min(0.84);
    let angle =
        octave * std::f64::consts::TAU * 0.42 + elapsed * 0.08 - std::f64::consts::FRAC_PI_2;
    let pan_offset = pan.clamp(-1.0, 1.0) as f64 * cx * 0.34;

    let x = cx + pan_offset + angle.cos() * cx * 0.48 * radius;
    let y = cy + angle.sin() * cy * 0.88 * radius;
    (x, y)
}

fn draw_emergence_status(
    buf: &mut Buffer,
    area: Rect,
    snapshot: &EmergenceSnapshot,
    field_height: u16,
    color: Color,
) {
    if area.height == 0 {
        return;
    }

    let status_y = area.height - 1;
    let status = format!(
        " {} e:{:.2} voices:{} gen:{} epoch:{} ",
        snapshot.spawn_mode.label(),
        snapshot.total_energy,
        snapshot.voices.len(),
        snapshot.generation_count,
        snapshot.epoch
    );
    draw_centered_text(buf, area, status_y, &status, Color::Rgb(140, 140, 160));

    if snapshot.spawn_mode == SpawnMode::Penrose && field_height + 1 < area.height {
        let ribbon: String = snapshot
            .recent_tiles
            .iter()
            .map(|tile| match tile {
                Tile::Long => 'L',
                Tile::Short => 'S',
            })
            .collect();
        if !ribbon.is_empty() {
            let text = format!(" worm:{} {ribbon} ", snapshot.walk_position);
            draw_centered_text(buf, area, field_height, &text, dim_rgb(color, 0.62));
        }
    }
}

/// Check if a ratio is near a simple harmonic fraction.
fn is_near_simple_ratio(ratio: f32) -> bool {
    crate::emergence::consonance_score(ratio as f64) > 0.4
}

fn put_cell(buf: &mut Buffer, area: Rect, x: f64, y: f64, ch: char, color: Color) {
    let ix = x.round() as i16;
    let iy = y.round() as i16;
    if ix >= 0 && iy >= 0 && (ix as u16) < area.width && (iy as u16) < area.height {
        let cell = &mut buf[(area.x + ix as u16, area.y + iy as u16)];
        cell.set_char(ch);
        cell.set_fg(color);
    }
}

fn draw_text(buf: &mut Buffer, area: Rect, x: f64, y: f64, text: &str, color: Color) {
    let start_x = x.round() as i16;
    let iy = y.round() as i16;
    if iy < 0 || (iy as u16) >= area.height {
        return;
    }

    for (offset, ch) in text.chars().enumerate() {
        let ix = start_x + offset as i16;
        if ix >= 0 && (ix as u16) < area.width {
            let cell = &mut buf[(area.x + ix as u16, area.y + iy as u16)];
            cell.set_char(ch);
            cell.set_fg(color);
        }
    }
}

fn draw_centered_text(buf: &mut Buffer, area: Rect, y: u16, text: &str, color: Color) {
    if y >= area.height || area.width == 0 {
        return;
    }

    let width = area.width as usize;
    let text_len = text.chars().count();
    let skip = text_len.saturating_sub(width);
    let visible_len = text_len.min(width);
    let start_x = (width - visible_len) / 2;

    for (offset, ch) in text.chars().skip(skip).take(width).enumerate() {
        let x = start_x + offset;
        if x >= width {
            break;
        }
        let cell = &mut buf[(area.x + x as u16, area.y + y)];
        cell.set_char(ch);
        cell.set_fg(color);
    }
}

fn dim_rgb(color: Color, factor: f32) -> Color {
    match color {
        Color::Rgb(red, green, blue) => Color::Rgb(
            (red as f32 * factor).clamp(0.0, 255.0) as u8,
            (green as f32 * factor).clamp(0.0, 255.0) as u8,
            (blue as f32 * factor).clamp(0.0, 255.0) as u8,
        ),
        other => other,
    }
}

/// Sub-cell braille drawing for smooth, jitter-free lines and curves.
fn put_braille(buf: &mut Buffer, area: Rect, x: f64, y: f64, color: Color) {
    let dot_x = (x * 2.0).round() as isize;
    let dot_y = (y * 4.0).round() as isize;
    if dot_x < 0 || dot_y < 0 {
        return;
    }
    let cx = (dot_x / 2) as u16;
    let cy = (dot_y / 4) as u16;
    if cx >= area.width || cy >= area.height {
        return;
    }

    let bx = dot_x % 2;
    let by = dot_y % 4;
    let bit = match (bx, by) {
        (0, 0) => 0x01,
        (1, 0) => 0x08,
        (0, 1) => 0x02,
        (1, 1) => 0x10,
        (0, 2) => 0x04,
        (1, 2) => 0x20,
        (0, 3) => 0x40,
        (1, 3) => 0x80,
        _ => 0,
    };

    let cell = &mut buf[(area.x + cx, area.y + cy)];
    let current = cell.symbol();
    let mut code = 0;
    if current.chars().count() == 1 {
        let c = current.chars().next().unwrap();
        if c as u32 >= 0x2800 && c as u32 <= 0x28FF {
            code = (c as u32) - 0x2800;
        } else if !is_braille_backdrop(c) {
            return;
        }
    }
    code |= bit;
    cell.set_char(char::from_u32(0x2800 + code).unwrap_or(' '));
    cell.set_fg(color);
}

fn is_braille_backdrop(ch: char) -> bool {
    matches!(ch, ' ' | '\u{00B7}' | '\u{2219}')
}

/// Draw a straight line using braille sub-cell resolution.
fn draw_line_braille(buf: &mut Buffer, area: Rect, from: (f64, f64), to: (f64, f64), color: Color) {
    let (x1, y1) = from;
    let (x2, y2) = to;
    let dx = x2 - x1;
    let dy = y2 - y1;
    let steps = ((dx * 2.0).abs().max((dy * 4.0).abs())) as usize;
    if steps == 0 {
        return;
    }

    let steps = steps.min(400); // safety cap
    for s in 0..=steps {
        let t = s as f64 / steps as f64;
        let x = x1 + dx * t;
        let y = y1 + dy * t;
        put_braille(buf, area, x, y, color);
    }
}

fn rgb_from_color(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128),
    }
}

#[cfg(test)]
mod tests {
    use super::harmonic_partial_levels;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1.0e-12,
            "actual {actual} expected {expected}"
        );
    }

    #[test]
    fn zero_warmth_leaves_only_the_fundamental() {
        let levels = harmonic_partial_levels(0.0, [0.5, 0.25, 0.125, 0.0625, 0.03125]);

        assert_close(levels[0], 1.0);
        for level in &levels[1..] {
            assert_close(*level, 0.0);
        }
    }

    #[test]
    fn partial_levels_match_audio_mixer_normalization() {
        let weights = [0.5, 0.25, 0.125, 0.0625, 0.03125];
        let levels = harmonic_partial_levels(1.0, weights);
        let norm = 1.0 + weights.iter().sum::<f64>();

        assert_close(levels[0], 1.0 / norm);
        for (idx, weight) in weights.iter().enumerate() {
            assert_close(levels[idx + 1], weight / norm);
        }
    }
}
