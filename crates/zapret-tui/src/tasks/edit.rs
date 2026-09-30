//! Handing the terminal to the user's text editor.
//!
//! The editor and the background event reader share stdin, so the reader has to
//! be paused for the duration, otherwise the two fight over keystrokes and the
//! editor misses keys.

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::Event;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc::Receiver;

use crate::editor::open_editor;
use crate::event::drain_events;
use crate::event::EventReader;
use crate::screen::{begin_external_output, end_external_output};
use crate::state::{ActiveScreen, AppState};

pub fn open_in_editor(
    app: &mut AppState,
    file_path: &str,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
    reader: &EventReader,
) -> Result<(), io::Error> {
    let return_screen = app.active_screen;
    reader.pause();

    begin_external_output(terminal)?;

    let _ = open_editor(file_path);

    end_external_output(terminal, rx)?;
    reader.resume();
    drain_events(rx);

    app.status_message = Some(format!(
        "{}{}",
        rust_i18n::t!("msg_closed_editor"),
        std::path::Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    ));
    app.active_screen = return_screen;
    if return_screen == ActiveScreen::ListsEditorSubmenu {
        app.refresh_ipset_status();
    }
    Ok(())
}
