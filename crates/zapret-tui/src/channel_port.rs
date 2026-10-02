//! The one bridge between this app's reader thread and tui-realm's listener.
//!
//! `EventReader` is the single thread in the process that touches the console
//! input handle — see [`crate::event`] for why more than one is a bug, not a
//! feature. tui-realm would rather read the console itself through
//! `CrosstermInputListener`, which would put a *second* thread on the same handle
//! and resurrect exactly that bug. So the listener gets this port instead and the
//! reader thread stays the only reader.

use std::sync::mpsc::TryRecvError;

use ratatui::crossterm::event::{Event, KeyEventKind, MouseEventKind};
use tuirealm::event::{
    Event as TmEvent, Key, KeyEvent, KeyModifiers, MouseButton as TmMouseButton, MouseEvent as TmMouseEvent,
    MouseEventKind as TmMouseKind, NoUserEvent,
};
use tuirealm::listener::{Poll, PortError, PortResult};

use crate::event::SharedRx;

/// Turns the reader's channel into something the event listener can poll.
pub struct ChannelPort {
    rx: SharedRx,
}

impl ChannelPort {
    pub fn new(rx: SharedRx) -> Self {
        Self { rx }
    }
}

impl Poll<NoUserEvent> for ChannelPort {
    /// Must not block: the listener calls this from its own worker thread on its
    /// own schedule, and a port that waits turns the listener into a second
    /// reader.
    fn poll(&mut self) -> PortResult<Option<TmEvent<NoUserEvent>>> {
        let rx = self.rx.lock().unwrap_or_else(|e| e.into_inner());
        match rx.try_recv() {
            Ok(ev) => Ok(to_tuirealm(ev)),
            Err(TryRecvError::Empty) => Ok(None),
            // The sender is the reader thread, which lives for the whole process.
            // If it ever goes away the listener must stop asking, not retry.
            Err(TryRecvError::Disconnected) => Err(PortError::PermanentError("event reader gone".into())),
        }
    }
}

/// What is worth forwarding.
///
/// Key releases and repeats were never acted on, and they would make one press
/// count three times in a menu that toggles on Enter. Resizes are ignored: the
/// frame loop repaints on a timer anyway, and ratatui picks the new size up by
/// itself.
fn to_tuirealm(ev: Event) -> Option<TmEvent<NoUserEvent>> {
    match ev {
        Event::Key(key) if key.kind == KeyEventKind::Press => Some(TmEvent::Keyboard(KeyEvent::new(
            to_key(key.code),
            KeyModifiers::NONE,
        ))),
        Event::Mouse(m) => {
            let kind = match m.kind {
                MouseEventKind::Down(b) => TmMouseKind::Down(button(b)),
                MouseEventKind::Up(b) => TmMouseKind::Up(button(b)),
                MouseEventKind::Drag(b) => TmMouseKind::Drag(button(b)),
                MouseEventKind::Moved => TmMouseKind::Moved,
                MouseEventKind::ScrollUp => TmMouseKind::ScrollUp,
                MouseEventKind::ScrollDown => TmMouseKind::ScrollDown,
                MouseEventKind::ScrollLeft => TmMouseKind::ScrollLeft,
                MouseEventKind::ScrollRight => TmMouseKind::ScrollRight,
            };
            Some(TmEvent::Mouse(TmMouseEvent {
                kind,
                modifiers: KeyModifiers::NONE,
                column: m.column,
                row: m.row,
            }))
        }
        _ => None,
    }
}

fn button(b: ratatui::crossterm::event::MouseButton) -> TmMouseButton {
    use ratatui::crossterm::event::MouseButton as B;
    match b {
        B::Left => TmMouseButton::Left,
        B::Right => TmMouseButton::Right,
        B::Middle => TmMouseButton::Middle,
    }
}

fn to_key(code: ratatui::crossterm::event::KeyCode) -> Key {
    use ratatui::crossterm::event::KeyCode as C;
    match code {
        C::Backspace => Key::Backspace,
        C::Enter => Key::Enter,
        C::Left => Key::Left,
        C::Right => Key::Right,
        C::Up => Key::Up,
        C::Down => Key::Down,
        C::Home => Key::Home,
        C::End => Key::End,
        C::PageUp => Key::PageUp,
        C::PageDown => Key::PageDown,
        C::Tab => Key::Tab,
        C::BackTab => Key::BackTab,
        C::Delete => Key::Delete,
        C::Insert => Key::Insert,
        C::F(n) => Key::Function(n),
        C::Char(c) => Key::Char(c),
        C::Null => Key::Null,
        C::CapsLock => Key::CapsLock,
        C::ScrollLock => Key::ScrollLock,
        C::NumLock => Key::NumLock,
        C::PrintScreen => Key::PrintScreen,
        C::Pause => Key::Pause,
        C::Menu => Key::Menu,
        C::KeypadBegin => Key::KeypadBegin,
        C::Media(k) => Key::Media(media(k)),
        C::Esc => Key::Esc,
        C::Modifier(_) => Key::Null,
    }
}

fn media(k: ratatui::crossterm::event::MediaKeyCode) -> tuirealm::event::MediaKeyCode {
    use ratatui::crossterm::event::MediaKeyCode as M;
    use tuirealm::event::MediaKeyCode as T;
    match k {
        M::Play => T::Play,
        M::Pause => T::Pause,
        M::PlayPause => T::PlayPause,
        M::Reverse => T::Reverse,
        M::Stop => T::Stop,
        M::FastForward => T::FastForward,
        M::Rewind => T::Rewind,
        M::TrackNext => T::TrackNext,
        M::TrackPrevious => T::TrackPrevious,
        M::Record => T::Record,
        M::LowerVolume => T::LowerVolume,
        M::MuteVolume => T::MuteVolume,
        M::RaiseVolume => T::RaiseVolume,
    }
}
