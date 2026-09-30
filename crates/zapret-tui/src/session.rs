//! The TUI session: draw a frame, take a key, react, run any pending job.
//!
//! The loop has two modes. Normally it draws a menu and dispatches keys. While a
//! sweep is in flight it draws the sweep's own screen instead and reads only the
//! cancel key — but it still repaints on the same 50 ms tick, which is the point:
//! the bar, the spinner and the elapsed clock keep moving whether or not the
//! sweep has anything new to say.

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use std::io;
use std::sync::mpsc::RecvTimeoutError;

use crate::draw::draw;
use crate::event::{drain_events, EventReader};
use crate::jobs::{start_autotune, start_ttl, JobOutcome};
use crate::state::actions;
use crate::state::{ActiveScreen, AppState};
use crate::tasks;

/// How long one frame waits for a key before repainting anyway.
const TICK: std::time::Duration = std::time::Duration::from_millis(50);

/// How far PageUp / PageDown move the autotune report.
const PAGE: usize = 10;

pub fn run_tui(app: &mut AppState, reader: &EventReader) -> Result<(), io::Error> {
    let rx = reader.rx();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Drop events queued while the app was outside the TUI (e.g. keys pressed
    // during a foreground zapret run) so a fresh session starts clean.
    drain_events(rx);

    loop {
        if app.job.is_some() {
            if let Some(job) = app.job.as_ref() {
                terminal.draw(|f| job.render(f))?;
            }
            if let Ok(Event::Key(key)) = rx.recv_timeout(TICK) {
                if key.kind == KeyEventKind::Press && is_cancel(key.code) {
                    if let Some(job) = app.job.as_ref() {
                        job.request_cancel();
                    }
                }
            }
            if app.job.as_ref().is_some_and(|j| j.is_finished()) {
                finish_job(app);
            }
        } else {
            terminal.draw(|f| draw(f, app))?;

            match rx.recv_timeout(TICK) {
                Ok(Event::Key(key)) => {
                    if key.kind == KeyEventKind::Press {
                        handle_key(app, key.code);
                    }
                }
                Ok(Event::Mouse(mouse)) => {
                    actions::on_mouse(app, mouse);
                    // A click is a press, so the screen the click landed on has
                    // to be re-read the same way a key press re-reads it — but
                    // only while there is no job, which owns the screen.
                    if app.job.is_none() {
                        actions::refresh_after_key(app);
                    }
                }
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    // The reader is immortal (see spawn_event_reader), so this
                    // should never happen while the TUI is active.
                }
            }

            dispatch(app, &mut terminal, rx, reader)?;
        }

        if app.should_run || app.should_quit {
            break;
        }
    }

    // The mouse has to be handed back explicitly: without this the shell that
    // starts next inherits a console that reports every mouse move.
    execute!(terminal.backend_mut(), DisableMouseCapture)?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// The keys that mean "stop", on a sweep screen. `q` and Esc, like everywhere
/// else; nothing else is read while a job owns the screen.
fn is_cancel(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
}

/// Apply what a finished sweep left behind, and get out of its way.
fn finish_job(app: &mut AppState) {
    let Some(outcome) = app.job.as_mut().and_then(|j| j.take_outcome()) else {
        return;
    };
    app.job = None;

    match outcome {
        JobOutcome::Autotune(results, cancelled) => {
            app.autotune_results = Some(*results);
            app.has_autotune_results_file = true;
            app.dpi_desync_ttl = zapret_wrapper::config::load_ttl();
            // The report is one step further in from where the sweep was
            // started, so Esc from it lands back on the autotune menu. A fresh
            // report opens at the top of its first tab rather than wherever the
            // last one was scrolled to.
            app.autotune_results_index = 0;
            app.autotune_report_tab = crate::state::AutotuneReportTab::Summary;
            app.open(ActiveScreen::AutotuneResultsSubmenu);
            app.status_message = Some(if cancelled {
                rust_i18n::t!("autotune_cancelled").into_owned()
            } else {
                rust_i18n::t!("autotune_done").into_owned()
            });
        }
        JobOutcome::Ttl(Ok(ttl)) => {
            let _ = zapret_wrapper::config::save_ttl(Some(ttl));
            app.dpi_desync_ttl = Some(ttl);
            // Back to the row the sweep was started from, not to the top.
            app.back();
            app.status_message = Some(rust_i18n::t!("ttl_found").replace("{}", &ttl.to_string()));
        }
        JobOutcome::Ttl(Err(e)) => {
            app.show_error(e);
        }
    }
    app.autotune_running = false;
}

/// Start whatever the `should_*` flags are asking for.
fn dispatch(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &std::sync::mpsc::Receiver<Event>,
    reader: &EventReader,
) -> Result<(), io::Error> {
    if app.should_download_zapret {
        app.should_download_zapret = false;
        tasks::download::download_zapret(app, terminal, rx)?;
    }

    if app.should_download_strategies {
        app.should_download_strategies = false;
        tasks::download::download_strategies(app, terminal, rx)?;
    }

    if app.should_download_defaults {
        app.should_download_defaults = false;
        tasks::download::download_defaults(app, terminal, rx)?;
    }

    if let Some(file_path) = app.should_open_editor.take() {
        tasks::edit::open_in_editor(app, &file_path, terminal, rx, reader)?;
    }

    if app.should_run_autotune {
        app.should_run_autotune = false;
        app.autotune_running = true;
        app.job = Some(start_autotune(app));
    }

    if app.should_run_ttl {
        app.should_run_ttl = false;
        app.job = Some(start_ttl(app));
    }

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
        KeyCode::PageUp if app.active_screen == ActiveScreen::AutotuneResultsSubmenu => app.scroll_report(false, PAGE),
        KeyCode::PageDown if app.active_screen == ActiveScreen::AutotuneResultsSubmenu => app.scroll_report(true, PAGE),
        KeyCode::Tab if app.active_screen == ActiveScreen::AutotuneResultsSubmenu => app.switch_report_tab(true),
        KeyCode::BackTab if app.active_screen == ActiveScreen::AutotuneResultsSubmenu => app.switch_report_tab(false),
        KeyCode::Left | KeyCode::Char('h') => actions::on_cycle(app, false),
        KeyCode::Right | KeyCode::Char('l') => actions::on_cycle(app, true),
        KeyCode::Enter | KeyCode::Char(' ') => actions::on_cycle(app, true),
        KeyCode::Char('q') | KeyCode::Esc => actions::on_back(app),
        _ => {}
    }

    actions::refresh_after_key(app);
}
