//! Console input: one reader thread for the whole process, plus the helpers
//! that drain it and wait for a key.
//!
//! The channel is shared rather than cloned, because `mpsc::Receiver` cannot be
//! cloned and there are two consumers: the reader's own tools ([`drain_events`],
//! [`wait_for_key`], the cancel key on a job screen) and tui-realm's listener
//! worker, which polls it through [`crate::channel_port::ChannelPort`]. They
//! never run at the same time — the ports are locked before this crate reads the
//! channel directly — which is the only thing that makes sharing safe.

use ratatui::crossterm::event::Event;
use ratatui::crossterm::terminal::enable_raw_mode;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::sync::mpsc::Receiver;

/// The channel the reader thread forwards console events into, shared by every
/// consumer in the process.
pub type SharedRx = Arc<Mutex<Receiver<Event>>>;

/// Read crossterm events on a dedicated thread and forward them over a channel.
///
/// Exactly one reader is created for the whole process (in `main`), so at any
/// moment only one thread waits on the console input handle: several waiters on
/// the same handle race for events and starve each other, which makes the menu
/// stop reacting to keys.
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

        // A transient console error must not kill the reader and leave the UI
        // without input, so every Err is a pause and a retry.
        match ratatui::crossterm::event::poll(std::time::Duration::from_millis(50)) {
            Ok(true) => match ratatui::crossterm::event::read() {
                Ok(event) => {
                    if tx.send(event).is_err() {
                        break;
                    }
                }
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(20)),
            },
            Ok(false) => {}
            Err(_) => std::thread::sleep(std::time::Duration::from_millis(20)),
        }
    });
    EventReader {
        rx: Arc::new(Mutex::new(rx)),
        paused,
    }
}

/// Handle for the process-wide event reader: the channel the reader thread
/// forwards keystrokes into, plus the ability to pause it from touching the
/// console while a child program needs exclusive access to stdin.
pub struct EventReader {
    rx: SharedRx,
    paused: Arc<AtomicBool>,
}

impl EventReader {
    /// The channel, behind a lock. Held for as short a time as possible: every
    /// consumer of the console events goes through here.
    pub fn rx(&self) -> MutexGuard<'_, Receiver<Event>> {
        // A poisoned lock would mean a consumer panicked mid-receive, and the
        // channel itself is still perfectly usable.
        self.rx.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// The channel shared with tui-realm's listener, which has to own its own
    /// handle because it polls from another thread.
    pub fn shared(&self) -> SharedRx {
        Arc::clone(&self.rx)
    }

    /// Stop the reader from polling the terminal so a child process (editor,
    /// prompt, ...) can read stdin without racing this thread for input.
    ///
    /// Blocks briefly until the reader thread is guaranteed to be off the console
    /// handle. The caller must balance every `pause` with a `resume`.
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
        // Polling has a 50 ms timeout, so give the thread a moment to notice the
        // flag before returning to the caller.
        std::thread::sleep(std::time::Duration::from_millis(150));
    }

    /// Allow the reader to poll the terminal again after an [`EventReader::pause`].
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
            Ok(Event::Key(_)) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Ok(_) | Err(mpsc::RecvTimeoutError::Timeout) => continue,
        }
    }
    Ok(())
}
