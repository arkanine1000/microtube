use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use crate::emergence::EmergenceSnapshot;

/// Render a waveform using braille characters into a ratatui Buffer.
pub fn render_braille_waveform(
    buf: &mut Buffer,
    area: Rect,
    samples: &[f32],
    color: Color,
) {
    if area.width == 0 || area.height == 0 || samples.is_empty() {
        return;
    }

    let dots_w = area.width as usize * 2;
    let dots_h = area.height as usize * 4;
    let center_y = dots_h / 2;

    let step = samples.len() as f64 / dots_w as f64;
    let mut dots = vec![vec![false; dots_w]; dots_h];

    for x in 0..dots_w {
        let sample_idx = ((x as f64 * step) as usize).min(samples.len() - 1);
        let sample = samples[sample_idx];

        let y = center_y as f64 - sample as f64 * (center_y as f64 * 0.9);
        let y = (y as usize).clamp(0, dots_h - 1);

        dots[y][x] = true;
        if y > 0 { dots[y - 1][x] = true; }
        if y + 1 < dots_h { dots[y + 1][x] = true; }
    }

    for cy in 0..area.height as usize {
        for cx in 0..area.width as usize {
            let mut code: u8 = 0;
            let dx = cx * 2;
            let dy = cy * 4;

            if dy < dots_h && dx < dots_w && dots[dy][dx] { code |= 0x01; }
            if dy + 1 < dots_h && dx < dots_w && dots[dy + 1][dx] { code |= 0x02; }
            if dy + 2 < dots_h && dx < dots_w && dots[dy + 2][dx] { code |= 0x04; }
            if dy + 3 < dots_h && dx < dots_w && dots[dy + 3][dx] { code |= 0x40; }
            if dy < dots_h && dx + 1 < dots_w && dots[dy][dx + 1] { code |= 0x08; }
            if dy + 1 < dots_h && dx + 1 < dots_w && dots[dy + 1][dx + 1] { code |= 0x10; }
            if dy + 2 < dots_h && dx + 1 < dots_w && dots[dy + 2][dx + 1] { code |= 0x20; }
            if dy + 3 < dots_h && dx + 1 < dots_w && dots[dy + 3][dx + 1] { code |= 0x80; }

            if code != 0 {
                let ch = char::from_u32(0x2800 + code as u32).unwrap_or(' ');
                let cell = &mut buf[(area.x + cx as u16, area.y + cy as u16)];
                cell.set_char(ch);
                cell.set_fg(color);
            }
        }
    }
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
    if bin_size == 0 { return; }

    for i in 0..num_bars {
        let start = i * bin_size;
        let end = (start + bin_size).min(samples.len());
        let energy: f32 = samples[start..end].iter().map(|s| s * s).sum::<f32>() / bin_size as f32;
        let amplitude = energy.sqrt() * 3.0;

        if amplitude > bars[i] { bars[i] = amplitude; } else { bars[i] *= 0.92; }
    }

    let block_chars = [' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
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

/// Render beat envelope visualization.
pub fn render_beat_envelope(
    buf: &mut Buffer,
    area: Rect,
    elapsed: f64,
    beat_freq: f64,
    color: Color,
) {
    if area.width == 0 || area.height == 0 { return; }

    let block_chars = [' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}', '\u{2588}'];
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

/// Penrose-inspired geometric visualization.
pub fn render_penrose(
    buf: &mut Buffer,
    area: Rect,
    elapsed: f64,
    beat_freq: f64,
    color: Color,
) {
    if area.width < 4 || area.height < 4 { return; }

    let w = area.width as f64;
    let h = area.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;

    let phi: f64 = 1.618033988749895;
    let beat_phase = (std::f64::consts::PI * beat_freq * elapsed).cos();
    let pulse = 0.7 + 0.3 * beat_phase.abs();
    let rotation = elapsed * 0.3;

    let (cr, cg, cb) = rgb_from_color(color);

    let layers = 5;
    for layer in 0..layers {
        let base_radius = (layer as f64 + 1.0) / layers as f64;
        let radius = base_radius * pulse * (cy.min(cx * 0.5));
        let sides = if layer % 2 == 0 { 5 } else { 10 };
        let layer_rotation = rotation + (layer as f64) * std::f64::consts::PI / (phi * 5.0);
        let brightness = 1.0 - (layer as f64 / layers as f64) * 0.5;
        let dim_beat = brightness * (0.6 + 0.4 * beat_phase.abs());

        let r = (cr as f64 * dim_beat) as u8;
        let g = (cg as f64 * dim_beat) as u8;
        let b = (cb as f64 * dim_beat) as u8;
        let layer_color = Color::Rgb(r, g, b);

        for i in 0..sides {
            let angle = layer_rotation + (i as f64 * std::f64::consts::TAU / sides as f64);
            let x = cx + radius * angle.cos() * 2.0;
            let y = cy + radius * angle.sin();

            let ix = x as u16;
            let iy = y as u16;

            if ix < area.width && iy < area.height {
                let cell = &mut buf[(area.x + ix, area.y + iy)];
                let ch = match layer % 5 {
                    0 => '\u{25C6}',
                    1 => '\u{25CB}',
                    2 => '\u{25B2}',
                    3 => '\u{2B21}',
                    _ => '\u{25CF}',
                };
                cell.set_char(ch);
                cell.set_fg(layer_color);
            }

            let next_angle = layer_rotation + ((i + 1) as f64 * std::f64::consts::TAU / sides as f64);
            let nx = cx + radius * next_angle.cos() * 2.0;
            let ny = cy + radius * next_angle.sin();

            let steps = (radius * 1.5) as usize;
            if steps > 0 {
                for s in 1..steps {
                    let t = s as f64 / steps as f64;
                    let px = (x + (nx - x) * t) as u16;
                    let py = (y + (ny - y) * t) as u16;
                    if px < area.width && py < area.height {
                        let cell = &mut buf[(area.x + px, area.y + py)];
                        cell.set_char('\u{00B7}');
                        cell.set_fg(layer_color);
                    }
                }
            }
        }
    }

    // Golden spiral
    let spiral_points = 60;
    for i in 0..spiral_points {
        let t = i as f64 / spiral_points as f64;
        let spiral_angle = t * std::f64::consts::TAU * 3.0 + rotation * phi;
        let spiral_r = t * cy * 0.8 * pulse;

        let sx = cx + spiral_r * spiral_angle.cos() * 2.0;
        let sy = cy + spiral_r * spiral_angle.sin();

        let ix = sx as u16;
        let iy = sy as u16;
        if ix < area.width && iy < area.height {
            let cell = &mut buf[(area.x + ix, area.y + iy)];
            let brightness = (0.5 + 0.5 * (t * std::f64::consts::TAU * beat_freq + elapsed).sin()) as f32;
            let r = (cr as f32 * brightness * 0.8) as u8;
            let g = (cg as f32 * brightness * 0.8) as u8;
            let b = (cb as f32 * brightness * 0.8) as u8;
            cell.set_char('\u{2219}');
            cell.set_fg(Color::Rgb(r, g, b));
        }
    }
}

/// Emergence visualization: a living constellation of harmonic voices.
/// Each voice is a node; connections show harmonic relationships.
/// Brightness = amplitude, size = generation, position = frequency ratio.
pub fn render_emergence(
    buf: &mut Buffer,
    area: Rect,
    snapshot: &EmergenceSnapshot,
    elapsed: f64,
    color: Color,
) {
    if area.width < 6 || area.height < 4 { return; }

    let w = area.width as f64;
    let h = area.height as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    let (cr, cg, cb) = rgb_from_color(color);

    // Draw connections between harmonically related voices
    for (i, v1) in snapshot.voices.iter().enumerate() {
        for v2 in snapshot.voices.iter().skip(i + 1) {
            let ratio = if v1.freq_ratio > v2.freq_ratio {
                v1.freq_ratio / v2.freq_ratio
            } else {
                v2.freq_ratio / v1.freq_ratio
            };

            // Connect if ratio is near a simple fraction
            let is_consonant = is_near_simple_ratio(ratio);
            if !is_consonant { continue; }

            let (x1, y1) = voice_position(v1.freq_ratio, v1.pan, cx, cy, elapsed);
            let (x2, y2) = voice_position(v2.freq_ratio, v2.pan, cx, cy, elapsed);

            // Draw faint connection line
            let connection_brightness = (v1.amplitude * v2.amplitude).min(1.0) * 0.5;
            let r = (cr as f32 * connection_brightness * 0.4) as u8;
            let g = (cg as f32 * connection_brightness * 0.4) as u8;
            let b = (cb as f32 * connection_brightness * 0.4) as u8;
            let line_color = Color::Rgb(r.max(30), g.max(30), b.max(30));

            draw_line(buf, area, x1, y1, x2, y2, '\u{2500}', line_color);
        }
    }

    // Draw each voice as a node
    for voice in &snapshot.voices {
        let (vx, vy) = voice_position(voice.freq_ratio, voice.pan, cx, cy, elapsed);
        let ix = vx as u16;
        let iy = vy as u16;

        if ix >= area.width || iy >= area.height { continue; }

        // Brightness based on amplitude
        let amp = voice.amplitude.min(1.0);
        let pulse = 0.8 + 0.2 * ((elapsed * 2.0 + voice.freq_ratio as f64 * 3.0).sin() as f32);
        let brightness = amp * pulse;

        let r = (cr as f32 * brightness).max(40.0) as u8;
        let g = (cg as f32 * brightness).max(30.0) as u8;
        let b = (cb as f32 * brightness).max(40.0) as u8;
        let voice_color = Color::Rgb(r, g, b);

        // Character based on generation (deeper = larger)
        let ch = match voice.generation {
            0 => '\u{2726}', // Four-pointed star (root)
            1 => '\u{25CF}', // Filled circle
            2 => '\u{25C9}', // Fisheye
            3 => '\u{25CB}', // Circle
            4 => '\u{25E6}', // Small circle
            5 => '\u{2022}', // Bullet
            _ => '\u{00B7}', // Dot
        };

        let cell = &mut buf[(area.x + ix, area.y + iy)];
        cell.set_char(ch);
        cell.set_fg(voice_color);

        // Halo for loud voices
        if amp > 0.5 {
            let halo_chars = ['\u{2591}', '\u{2592}'];
            let halo_ch = halo_chars[(voice.generation as usize) % 2];
            let halo_brightness = (amp - 0.5) * 0.4;
            let hr = (cr as f32 * halo_brightness) as u8;
            let hg = (cg as f32 * halo_brightness) as u8;
            let hb = (cb as f32 * halo_brightness) as u8;
            let halo_color = Color::Rgb(hr.max(20), hg.max(20), hb.max(20));

            for &(dx, dy) in &[(-1i16, 0i16), (1, 0), (0, -1), (0, 1)] {
                let hx = ix as i16 + dx;
                let hy = iy as i16 + dy;
                if hx >= 0 && hy >= 0 && (hx as u16) < area.width && (hy as u16) < area.height {
                    let hcell = &mut buf[(area.x + hx as u16, area.y + hy as u16)];
                    if hcell.symbol() == " " {
                        hcell.set_char(halo_ch);
                        hcell.set_fg(halo_color);
                    }
                }
            }
        }
    }

    // Draw epoch/generation indicator at bottom
    if area.height > 2 {
        let info = format!(
            " gen:{} voices:{} epoch:{} ",
            snapshot.generation_count,
            snapshot.voices.len(),
            snapshot.epoch
        );
        let info_x = (area.width as usize).saturating_sub(info.len()) / 2;
        let info_y = area.height - 1;
        for (i, ch) in info.chars().enumerate() {
            let x = info_x + i;
            if x < area.width as usize {
                let cell = &mut buf[(area.x + x as u16, area.y + info_y)];
                cell.set_char(ch);
                cell.set_fg(Color::Rgb(140, 140, 160));
            }
        }
    }
}

/// Map a voice's frequency ratio to a screen position.
/// Uses a logarithmic radial layout: ratio determines angle, pan determines radius offset.
fn voice_position(freq_ratio: f32, pan: f32, cx: f64, cy: f64, elapsed: f64) -> (f64, f64) {
    // Log-frequency determines angular position (octave = half rotation)
    let angle = (freq_ratio as f64).ln() * 2.5 + elapsed * 0.1;
    // Radius: pan + a base offset
    let radius = (0.3 + pan.abs() as f64 * 0.5) * cy * 0.85;

    let x = cx + radius * angle.cos() * 2.0; // *2 for terminal aspect ratio
    let y = cy + radius * angle.sin();
    (x, y)
}

/// Check if a ratio is near a simple harmonic fraction.
fn is_near_simple_ratio(ratio: f32) -> bool {
    let simple = [1.0, 1.5, 2.0, 1.333, 1.25, 1.618, 0.667, 0.75];
    simple.iter().any(|&r| (ratio - r).abs() < 0.15)
}

/// Draw a line between two points using a character.
fn draw_line(
    buf: &mut Buffer,
    area: Rect,
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    ch: char,
    color: Color,
) {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let steps = (dx.abs().max(dy.abs())) as usize;
    if steps == 0 { return; }

    let steps = steps.min(40); // Cap to avoid excessive iteration
    for s in 0..=steps {
        let t = s as f64 / steps as f64;
        let x = (x1 + dx * t) as u16;
        let y = (y1 + dy * t) as u16;
        if x < area.width && y < area.height {
            let cell = &mut buf[(area.x + x, area.y + y)];
            if cell.symbol() == " " {
                cell.set_char(ch);
                cell.set_fg(color);
            }
        }
    }
}

fn rgb_from_color(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (128, 128, 128),
    }
}
