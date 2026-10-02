//! What a screen can say back, and what the session has to do next.
//!
//! A component never changes [`AppState`](crate::state::AppState) itself: its
//! `on()` turns an event into a [`Msg`], and `update` — in the same file as the
//! component — applies it. That is what keeps a screen's whole history in one
//! place instead of spread across a component, a key handler and a render table.
//!
//! Navigation ([`Msg::Open`], [`Msg::Back`]) and quitting are *not* per screen, so
//! they are handled once in [`crate::model`] rather than twenty times.

use crate::state::screens::ActiveScreen;

use crate::screens;

/// Everything one screen can ask for.
#[derive(Debug, PartialEq, Clone)]
pub enum Msg {
    /// The cursor moved, so whatever the last action said has been read.
    ///
    /// A message from the last key replaces the row's own explanation, and it is
    /// a movement — not a repaint — that brings the explanation back. Keeping
    /// the two apart is what lets an action say something and still redraw.
    Moved,
    /// Something changed that the state has to look at; draw another frame.
    Redraw,
    /// Esc / q: one step up the back stack, or leave from the main menu.
    Back,
    /// One step up, then say this on the screen we land on.
    ///
    /// The status line is cleared by a normal step back, so an action that has
    /// something to report — "strategy set to …" — has to say it on the way out
    /// rather than before it.
    BackWith(String),
    /// Leave the TUI for good.
    Quit,
    /// Go one step down into `screen`, remembering where we came from.
    Open(ActiveScreen),

    Main(screens::main::MainMsg),
    Defender(screens::defender::DefenderMsg),
    Strategy(screens::strategy::StrategyMsg),
    DownloadDeps(screens::download::DownloadDepsMsg),
    Download(screens::download::DownloadMsg),
    Tag(screens::tag::TagMsg),
    Gamefilter(screens::gamefilter::GamefilterMsg),
    Service(screens::service::ServiceMsg),
    Lists(screens::lists::ListsMsg),
    Extended(screens::extended::ExtendedMsg),
    Ttl(screens::ttl::TtlMsg),
    Fakes(screens::fakes::FakesMsg),
    Autotune(screens::autotune::AutotuneMsg),
    Domains(screens::autotune::DomainsMsg),
    Protocols(screens::autotune::ProtocolsMsg),
    BlockChecks(screens::autotune::BlockChecksMsg),
    Presets(screens::autotune::PresetsMsg),
    AutotuneStrategies(screens::autotune::AutotuneStrategiesMsg),
    NumRequests(screens::autotune::NumRequestsMsg),
    Report(screens::report_screen::ReportMsg),
}

/// What the session owes the rest of the process once the messages are done.
///
/// One value rather than six booleans on the state: a flag cannot be taken back
/// once set, so a task that never ran leaves it armed for the next key.
#[derive(Debug, PartialEq, Clone)]
pub enum Pending {
    /// Hand the terminal to zapret and leave the TUI.
    Run,
    /// Leave the TUI without running anything.
    Quit,
    DownloadZapret,
    DownloadStrategies,
    DownloadDefaults,
    /// Open this file in the user's editor, then come back.
    OpenEditor(String),
    StartAutotune,
    StartTtl,
}
