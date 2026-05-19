#![allow(dead_code)]

mod app;
mod audio;
mod emergence;
mod knowledge;
mod local_presets;
mod penrose;
mod presets;
mod shepard;
mod theme;
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

use app::{App, AppMode, AudioParams, Tab, VizBuffer};
use audio::AudioEngine;
use emergence::EmergenceSnapshot;
use presets::SEQUENCES;

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
        app.update_signals();
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

    // Tab switching is global. Capital Q is a hard-quit reachable from
    // either tab; lowercase q means "go back" inside Knowledge sub-views,
    // and "quit" only from Studio Normal mode.
    if code == KeyCode::Char('Q') {
        app.should_quit = true;
        return;
    }
    if matches!(code, KeyCode::Tab | KeyCode::BackTab) {
        app.tab = app.tab.flipped();
        app.signals.last_tab_switch = Some(Instant::now());
        if app.tab == Tab::Knowledge {
            // Close any Studio modal so it doesn't reappear on tab return.
            app.mode = AppMode::Normal;
        }
        return;
    }

    match app.tab {
        Tab::Studio => match app.mode {
            AppMode::Normal => handle_normal(app, code),
            AppMode::PresetSelect => handle_preset_menu(app, code),
            AppMode::SequenceSelect => handle_sequence_menu(app, code),
            AppMode::PresetName => handle_preset_name(app, code),
            AppMode::Help => {
                app.mode = AppMode::Normal;
            }
        },
        Tab::Knowledge => {
            knowledge::handle_key(app, code);
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
            app.menu_index = app.preset_menu_index();
        }
        KeyCode::Char('s') => {
            app.mode = AppMode::SequenceSelect;
            app.menu_index = app.current_sequence.unwrap_or(0);
        }
        KeyCode::Char('S') => app.begin_preset_save(),

        // Viz
        KeyCode::Char('v') => app.next_viz_mode(),
        KeyCode::Char('V') => app.prev_viz_mode(),

        // Emergence toggle
        KeyCode::Char('e') => app.toggle_emergence(),

        // Noise toggle
        KeyCode::Char('n') => app.toggle_noise(),
        KeyCode::Char('m') => app.cycle_mist_type(),
        KeyCode::Char('t') => app.cycle_timbre(),
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

fn handle_preset_menu(app: &mut App, code: KeyCode) {
    let len = app.total_preset_count();
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
            app.apply_preset_menu_index(app.menu_index);
            app.mode = AppMode::Normal;
        }

        KeyCode::Char('d') => {
            app.delete_preset_menu_index(app.menu_index);
        }

        KeyCode::Char(c @ '1'..='9') => {
            let idx = (c as usize) - ('1' as usize);
            if idx < len {
                app.apply_preset_menu_index(idx);
                app.mode = AppMode::Normal;
            }
        }

        _ => {}
    }
}

fn handle_sequence_menu(app: &mut App, code: KeyCode) {
    let len = SEQUENCES.len();
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
            app.start_sequence(app.menu_index);
            app.mode = AppMode::Normal;
        }

        KeyCode::Char(c @ '1'..='9') => {
            let idx = (c as usize) - ('1' as usize);
            if idx < len {
                app.start_sequence(idx);
                app.mode = AppMode::Normal;
            }
        }

        _ => {}
    }
}

fn handle_preset_name(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => app.cancel_preset_save(),
        KeyCode::Enter => app.finish_preset_save(),
        KeyCode::Backspace => {
            app.preset_name_input.pop();
        }
        KeyCode::Char(c) if !c.is_control() && app.preset_name_input.len() < 48 => {
            app.preset_name_input.push(c);
        }
        _ => {}
    }
}
