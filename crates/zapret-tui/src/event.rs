//! Console input: one reader thread for the whole process, plus the helpers
//! that drain it and wait for a key.

use crossterm::event::Event;
use crossterm::terminal::enable_raw_mode;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::Arc;

/// Read crossterm events on a dedicated thread and forward them over a channel.
///
/// Exactly one reader is created for the whole process (in `main`), so at any
/// moment only one thread waits on the console input handle. Spawning a fresh
/// reader for every TUI session used to leak zombie reader threads that stayed
/// blocked in `WaitForMultipleObjects` forever, and several waiters on the same
/// input handle race for events and starve each other, which made the menu stop
/// reacting to keys.
///
/// The reader can be paused while an external program (e.g. the text editor)
/// reads the terminal itself. While paused it stops polling the console, so the
/// child process gets every keystroke instead of racing this thread for input.
pub fn spawn_event_reader() -> EventReader {
    let (tx, rx) = mpsc::channel();
    let paused = Arc::new(AtomicBool::new(false));
    let paused_reader = Arc::clone(&paused);
    std::thread::spawn(move || loop {
        if paused_reader.load(Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }

        match crossterm::event::poll(std::time::Duration::from_millis(50)) {
            Ok(true) => match crossterm::event::read() {
                Ok(event) => {
                    if tx.send(event).is_err() {
                        break;
                    }
                }
                Err(_) => {
                    // Transient console error; keep the reader alive and retry
                    // instead of dying and leaving the UI without input.
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            },
            Ok(false) => {}
            Err(_) => {
                // Transient console error; retry like above.
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        }
    });
    EventReader { rx, paused }
}

/// Handle for the process-wide event reader: the channel the reader thread
/// forwards keystrokes into, plus the ability to pause it from touching the
/// console while a child program needs exclusive access to stdin.
pub struct EventReader {
    rx: Receiver<Event>,
    paused: Arc<AtomicBool>,
}

impl EventReader {
    /// The channel the reader thread forwards console events into. The TUI
    /// draws and reacts to keys by receiving from here.
    pub fn rx(&self) -> &Receiver<Event> {
        &self.rx
    }

    /// Stop the reader from polling the terminal so a child process (editor,
    /// prompt, ...) can read stdin without racing this thread for input.
    ///
    /// Blocks briefly until the reader thread is guaranteed to be off the
    /// console handle. The caller must balance every `pause` with a `resume`.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
        // Polling has a 50 ms timeout, so give the thread a moment to notice
        // the flag before returning to the caller.
        std::thread::sleep(std::time::Duration::from_millis(150));
    }

    /// Allow the reader to poll the terminal again after a [`EventReader::pause`].
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }
}

pub fn drain_events(rx: &Receiver<Event>) {
    while rx.try_recv().is_ok() {}
}

pub fn wait_for_key(rx: &Receiver<Event>) -> Result<(), io::Error> {
    enable_raw_mode()?;
    drain_events(rx);
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(Event::Key(_)) => break,
            Ok(_) => continue,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    Ok(())
}
