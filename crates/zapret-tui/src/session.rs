//! The TUI session: draw a frame, take a key, react, run any pending task.

use crossterm::event::{Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc::RecvTimeoutError;

use crate::draw::draw;
use crate::event::{drain_events, EventReader};
use crate::state::actions;
use crate::state::AppState;
use crate::tasks;

pub fn run_tui(app: &mut AppState, reader: &EventReader) -> Result<(), io::Error> {
    let rx = reader.rx();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Drop events queued while the app was outside the TUI (e.g. keys pressed
    // during a foreground zapret run) so a fresh session starts clean.
    drain_events(rx);

    loop {
        terminal.draw(|f| draw(f, app))?;

        match rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(Event::Key(key)) => {
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key.code);
                }
            }
            Ok(_) => {}
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                // The reader is immortal (see spawn_event_reader), so this
                // should never happen while the TUI is active.
            }
        }

        if app.should_download_zapret {
            app.should_download_zapret = false;
            tasks::download::download_zapret(app, &mut terminal, rx)?;
        }

        if app.should_download_strategies {
            app.should_download_strategies = false;
            tasks::download::download_strategies(app, &mut terminal, rx)?;
        }

        if app.should_download_defaults {
            app.should_download_defaults = false;
            tasks::download::download_defaults(app, &mut terminal, rx)?;
        }

        if let Some(file_path) = app.should_open_editor.take() {
            tasks::edit::open_in_editor(app, &file_path, &mut terminal, rx, reader)?;
        }

        if app.should_run_autotune {
            app.should_run_autotune = false;
            tasks::autotune::run_autotune(app, &mut terminal, rx)?;
        }

        if app.should_run_ttl {
            app.should_run_ttl = false;
            tasks::ttl_run::run_ttl_autopick(app, &mut terminal, rx)?;
        }

        if app.should_run || app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// Translate one key press into state changes.
///
/// The navigation keys go to [`AppState`]; everything else is the current
/// screen's business, and a screen that starts a long job only raises a
/// `should_*` flag that the session loop picks up below.
fn handle_key(app: &mut AppState, code: KeyCode) {
    if app.autotune_request_editing {
        match code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                app.autotune_request_buf.push(c);
            }
            KeyCode::Backspace => {
                app.autotune_request_buf.pop();
            }
            KeyCode::Enter => {
                if let Ok(n) = app.autotune_request_buf.parse::<usize>() {
                    app.autotune_config.num_requests = n.max(1);
                }
                app.autotune_request_editing = false;
                app.autotune_request_buf.clear();
            }
            KeyCode::Esc => {
                app.autotune_request_editing = false;
                app.autotune_request_buf.clear();
            }
            _ => {}
        }
        return;
    }
    match code {
        KeyCode::Up | KeyCode::Char('k') => app.prev_menu(),
        KeyCode::Down | KeyCode::Char('j') => app.next_menu(),
        KeyCode::Left | KeyCode::Char('h') => {
            if app.is_ttl_autopick_selected() {
                app.change_ttl(false);
            } else {
                actions::on_cycle(app, false);
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if app.is_ttl_autopick_selected() {
                app.change_ttl(true);
            } else {
                actions::on_cycle(app, true);
            }
        }
        KeyCode::Enter => {
            if app.is_ttl_autopick_selected() {
                if app.check_dependencies() {
                    app.should_run_ttl = true;
                }
            } else {
                actions::on_cycle(app, true);
            }
        }
        KeyCode::Char(' ') => {
            if app.is_ttl_autopick_selected() {
                app.change_ttl(true);
            } else {
                actions::on_cycle(app, true);
            }
        }
        KeyCode::Char('q') | KeyCode::Esc => actions::on_back(app),
        _ => {}
    }

    actions::refresh_after_key(app);
}
