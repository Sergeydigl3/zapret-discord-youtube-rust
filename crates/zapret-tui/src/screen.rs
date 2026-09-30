//! Terminal mode: raw mode, the alternate screen, and handing the terminal over
//! to an external program.
//!
//! On Windows the screen is deliberately *not* torn down between the TUI and an
//! external program; see [`begin_external_output`].

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::Event;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::enable_raw_mode;
#[cfg(target_os = "windows")]
use ratatui::crossterm::terminal::Clear;
#[cfg(target_os = "windows")]
use ratatui::crossterm::terminal::ClearType;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc::Receiver;

use crate::event::{drain_events, wait_for_key};

// Only the POSIX path leaves the alternate screen, so these are Windows-unused.
#[cfg(not(target_os = "windows"))]
use ratatui::crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};

/// Prepare the Windows console for a colorful, UTF-8 TUI:
/// - switch it to UTF-8 (codepage 65001) so emoji and localized text emitted
///   by Rust's UTF-8 stdout are decoded correctly instead of being garbled by
///   the OEM codepage;
/// - explicitly enable ANSI/VT processing so ratatui's color output is
///   rendered even before crossterm happens to enable it lazily.
///
/// Non-Windows platforms need no preparation.
#[cfg(target_os = "windows")]
pub fn setup_console() {
    #[link(name = "kernel32")]
    extern "system" {
        fn SetConsoleCP(wCodePageID: u32) -> i32;
        fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
    }

    unsafe {
        SetConsoleCP(65001);
        SetConsoleOutputCP(65001);
    }
    // Asking crossterm whether the console understands ANSI also switches VT
    // processing on; there is no separate "enable it" call to make.
    let _ = ratatui::crossterm::ansi_support::supports_ansi();
}

#[cfg(not(target_os = "windows"))]
pub fn setup_console() {}

/// On Windows keep raw mode and the alternate screen active and print into it.
/// Toggling raw mode / alternate screen on ConPTY desyncs crossterm's event
/// reader, which makes the menu stop reacting to keys afterwards.
#[cfg(target_os = "windows")]
pub fn begin_external_output(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), io::Error> {
    execute!(terminal.backend_mut(), Clear(ClearType::All))?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn begin_external_output(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), io::Error> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn end_external_output(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
) -> Result<(), io::Error> {
    enable_raw_mode()?;
    terminal.clear()?;
    drain_events(rx);
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn end_external_output(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
) -> Result<(), io::Error> {
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;
    drain_events(rx);
    Ok(())
}

/// Hand the terminal to an external program, report the outcome, and take the
/// terminal back once the user presses a key.
pub fn run_download(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
    download: impl FnOnce() -> Result<(), String>,
) -> Result<Result<(), String>, io::Error> {
    begin_external_output(terminal)?;

    let res = download();

    match &res {
        Ok(_) => println!("{}", rust_i18n::t!("msg_dl_ok")),
        Err(err_msg) => {
            println!("{}{}", rust_i18n::t!("msg_dl_fail"), err_msg);
            println!("{}", rust_i18n::t!("msg_dl_key"));
        }
    }

    wait_for_key(rx)?;
    end_external_output(terminal, rx)?;

    Ok(res)
}
