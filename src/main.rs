#![allow(dead_code)]

mod app;
mod audio;
mod emergence;
mod penrose;
mod presets;
mod shepard;
mod ui;
mod visualization;

use std::io;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use app::{App, AppMode, AudioParams, VizBuffer};
use audio::AudioEngine;
use emergence::EmergenceSnapshot;
use presets::{PRESETS, SEQUENCES};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !io::IsTerminal::is_terminal(&io::stdout()) {
        eprintln!("microtube requires an interactive terminal (e.g. alacritty, kitty, wezterm)");
        std::process::exit(1);
    }

    let params = Arc::new(AudioParams::new());
    let viz_buffer = Arc::new(Mutex::new(VizBuffer::new(2048)));
    let emergence_snapshot = Arc::new(Mutex::new(EmergenceSnapshot::empty()));

    // Default: Alpha/Relaxation with warm harmonics
    params.set_base_freq(220.0);
    params.set_beat_freq(10.0);
    params.set_harmonics(0.3);

    let _audio = match AudioEngine::new(
        Arc::clone(&params),
        Arc::clone(&viz_buffer),
        Arc::clone(&emergence_snapshot),
    ) {
        Ok(engine) => Some(engine),
        Err(e) => {
            eprintln!("Audio: {e} (continuing without sound)");
            None
        }
    };

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new(
        Arc::clone(&params),
        Arc::clone(&viz_buffer),
        Arc::clone(&emergence_snapshot),
    );

    let tick_rate = Duration::from_millis(33);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            handle_key(&mut app, key.code, key.modifiers);
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            app.update_sequence();
        }

        if app.should_quit {
            break;
        }
    }

    terminal::disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
        app.should_quit = true;
        return;
    }

    match app.mode {
        AppMode::Normal => handle_normal(app, code),
        AppMode::PresetSelect => handle_menu(app, code, PRESETS.len(), true),
        AppMode::SequenceSelect => handle_menu(app, code, SEQUENCES.len(), false),
        AppMode::Help => {
            app.mode = AppMode::Normal;
        }
    }
}

fn handle_normal(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,

        KeyCode::Char(' ') => {
            let current = app.params.playing.load(Ordering::Relaxed);
            app.params.playing.store(!current, Ordering::Relaxed);
        }

        // Vim navigation
        KeyCode::Char('j') | KeyCode::Down => app.active_param = app.active_param.next(),
        KeyCode::Char('k') | KeyCode::Up => app.active_param = app.active_param.prev(),
        KeyCode::Char('h') | KeyCode::Left => app.adjust_param(-1.0),
        KeyCode::Char('l') | KeyCode::Right => app.adjust_param(1.0),
        KeyCode::Char('H') => app.adjust_param(-5.0),
        KeyCode::Char('L') => app.adjust_param(5.0),

        // Menus
        KeyCode::Char('p') => {
            app.mode = AppMode::PresetSelect;
            app.menu_index = app.current_preset.unwrap_or(0);
        }
        KeyCode::Char('s') => {
            app.mode = AppMode::SequenceSelect;
            app.menu_index = app.current_sequence.unwrap_or(0);
        }

        // Viz
        KeyCode::Char('v') => app.viz_mode = app.viz_mode.next(),
        KeyCode::Char('V') => app.viz_mode = app.viz_mode.prev(),

        // Emergence toggle
        KeyCode::Char('e') => {
            let current = app.params.get_emergence();
            if current > 0.01 {
                app.params.set_emergence(0.0);
                app.clear_emergence_snapshot();
            } else {
                app.params.set_emergence(0.5);
                // Auto-switch to emergence viz if not already
                if app.viz_mode != app::VizMode::Emergence {
                    app.viz_mode = app::VizMode::Emergence;
                }
            }
        }

        // Noise toggle
        KeyCode::Char('n') => {
            let current = app.params.get_noise_level();
            if current > 0.01 {
                app.params.set_noise_level(0.0);
            } else {
                app.params.set_noise_level(0.15);
            }
        }
        KeyCode::Char('m') => app.cycle_mist_type(),
        KeyCode::Char('g') => app.toggle_spawn_mode(),
        KeyCode::Char('r') => app.toggle_shepard(),
        KeyCode::Char('R') => app.reverse_shepard(),

        // Quick presets
        KeyCode::Char('1') => app.apply_preset(0),
        KeyCode::Char('2') => app.apply_preset(1),
        KeyCode::Char('3') => app.apply_preset(2),
        KeyCode::Char('4') => app.apply_preset(3),
        KeyCode::Char('5') => app.apply_preset(4),

        KeyCode::Char('?') => app.mode = AppMode::Help,

        _ => {}
    }
}

fn handle_menu(app: &mut App, code: KeyCode, len: usize, is_preset: bool) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.mode = AppMode::Normal,

        KeyCode::Char('j') | KeyCode::Down => {
            if app.menu_index < len - 1 {
                app.menu_index += 1;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.menu_index > 0 {
                app.menu_index -= 1;
            }
        }
        KeyCode::Char('g') => app.menu_index = 0,
        KeyCode::Char('G') => app.menu_index = len - 1,

        KeyCode::Enter | KeyCode::Char('l') => {
            if is_preset {
                app.apply_preset(app.menu_index);
            } else {
                app.start_sequence(app.menu_index);
            }
            app.mode = AppMode::Normal;
        }

        KeyCode::Char(c @ '1'..='9') => {
            let idx = (c as usize) - ('1' as usize);
            if idx < len {
                if is_preset {
                    app.apply_preset(idx);
                } else {
                    app.start_sequence(idx);
                }
                app.mode = AppMode::Normal;
            }
        }

        _ => {}
    }
}
