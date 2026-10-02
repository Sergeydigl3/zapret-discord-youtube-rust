//! Handing the terminal to the user's text editor.
//!
//! The editor and the background event reader share stdin, so the reader has to
//! be paused for the duration, otherwise the two fight over keystrokes and the
//! editor misses keys. The listener's port is locked by the caller, for the same
//! reason: it is a second reader of the same channel.

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::Event;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc::Receiver;

use crate::editor::open_editor;
use crate::event::drain_events;
use crate::event::EventReader;
use crate::screen::{begin_external_output, end_external_output};
use crate::state::AppState;

pub fn open_in_editor(
    app: &mut AppState,
    file_path: &str,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
    reader: &EventReader,
) -> Result<(), io::Error> {
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
    // The screen is not restored on purpose: the editor was opened from a file
    // row and the row the user chose is still under the cursor. Only the list of
    // files itself has to be re-read, since one of them may have just been
    // created, renamed or deleted.
    app.lists_files = zapret_wrapper::lists::get_lists_files();
    app.refresh_ipset_status();
    Ok(())
}
