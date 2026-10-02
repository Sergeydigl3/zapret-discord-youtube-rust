//! The screens: one component per screen, and the key handling they share.
//!
//! **The rule that keeps this from becoming what it replaced:** a screen's whole
//! history lives in its own file — the component, its `on`, its own `Msg`
//! variants, and the `update` arms that apply them. There is no
//! `state/actions/<screen>.rs` and no `match ActiveScreen` anywhere: adding a
//! screen is one file and one line of [`all`].
//!
//! Adding a line to a screen that already exists is one line in one file, which
//! is the whole point — before, the same line had to be added to the key
//! handler, the mouse handler, the draw table and the help table in four
//! different files.

pub mod autotune;
#[cfg(target_os = "windows")]
pub mod defender;
pub mod download;
pub mod extended;
pub mod fakes;
pub mod gamefilter;
pub mod lists;
pub mod main;
pub mod report_screen;
pub mod service;
pub mod strategy;
pub mod tag;
pub mod ttl;

use tuirealm::application::Application;
use tuirealm::event::{Event, Key, MouseButton, MouseEventKind, NoUserEvent};

use crate::menu::MenuList;
use crate::msg::Msg;
use crate::state::AppState;
use crate::state::screens::ActiveScreen;

/// The application this crate mounts its screens into.
pub type App = Application<Id, Msg, NoUserEvent>;

/// Every mounted screen. One variant per component, one per [`ActiveScreen`].
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
pub enum Id {
    Main,
    #[cfg(target_os = "windows")]
    Defender,
    Strategy,
    DownloadDeps,
    DownloadZapret,
    DownloadStrategies,
    ZapretTag,
    StrategyTag,
    Gamefilter,
    Service,
    Lists,
    Extended,
    Ttl,
    Fakes,
    FakesSelect,
    Autotune,
    NumRequests,
    Domains,
    Protocols,
    BlockChecks,
    Presets,
    AutotuneStrategies,
    Report,
}

/// The component that answers for a screen.
pub fn id_of(screen: ActiveScreen) -> Id {
    match screen {
        ActiveScreen::Main => Id::Main,
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => Id::Defender,
        ActiveScreen::StrategySubmenu => Id::Strategy,
        ActiveScreen::DownloadDepsSubmenu => Id::DownloadDeps,
        ActiveScreen::DownloadZapretSubmenu => Id::DownloadZapret,
        ActiveScreen::DownloadStrategiesSubmenu => Id::DownloadStrategies,
        ActiveScreen::ZapretTagSelect => Id::ZapretTag,
        ActiveScreen::StrategyTagSelect => Id::StrategyTag,
        ActiveScreen::GamefilterSubmenu => Id::Gamefilter,
        ActiveScreen::ServiceSubmenu => Id::Service,
        ActiveScreen::ListsEditorSubmenu => Id::Lists,
        ActiveScreen::ExtendedSubmenu => Id::Extended,
        ActiveScreen::TtlSubmenu => Id::Ttl,
        ActiveScreen::FakesSubmenu => Id::Fakes,
        ActiveScreen::FakesSelectSubmenu => Id::FakesSelect,
        ActiveScreen::AutotuneSubmenu => Id::Autotune,
        ActiveScreen::AutotuneNumRequests => Id::NumRequests,
        ActiveScreen::AutotuneEditDomainsSubmenu => Id::Domains,
        ActiveScreen::AutotuneProtocolsSubmenu => Id::Protocols,
        ActiveScreen::AutotuneBlockChecksSubmenu => Id::BlockChecks,
        ActiveScreen::AutotunePresetSelectionSubmenu => Id::Presets,
        ActiveScreen::AutotuneStrategiesSubmenu => Id::AutotuneStrategies,
        ActiveScreen::AutotuneResultsSubmenu => Id::Report,
    }
}

/// One screen, as the model sees it: what to mount, and how to re-seed it.
pub struct ScreenDef {
    pub id: Id,
    /// Build the component and put it in the view.
    pub mount: fn(&mut App, &AppState),
    /// Re-read the state the component is drawn from.
    ///
    /// Called before every frame rather than after every key, so a row can never
    /// show a state that has already changed underneath it — which is what the
    /// old `refresh_after_key` existed to patch over.
    pub sync: fn(&mut App, &AppState),
    /// Run once, when the screen is opened: where the cursor starts, and what
    /// outside state it has to re-read before it can be believed.
    pub enter: fn(&mut App, &mut AppState),
}

/// Every screen in the app. The one list that has to be touched to add one.
pub fn all() -> Vec<ScreenDef> {
    use Id::*;
    vec![
        ScreenDef { id: Main, mount: main::mount, sync: main::sync, enter: main::enter },
        ScreenDef { id: Strategy, mount: strategy::mount, sync: strategy::sync, enter: strategy::enter },
        ScreenDef {
            id: DownloadDeps,
            mount: download::mount_deps,
            sync: download::sync_deps,
            enter: download::enter_deps,
        },
        ScreenDef {
            id: DownloadZapret,
            mount: download::mount_sub,
            sync: download::sync_sub,
            enter: download::enter_sub,
        },
        ScreenDef {
            id: DownloadStrategies,
            mount: download::mount_sub,
            sync: download::sync_sub,
            enter: download::enter_sub,
        },
        ScreenDef { id: ZapretTag, mount: tag::mount, sync: tag::sync, enter: tag::enter },
        ScreenDef { id: StrategyTag, mount: tag::mount, sync: tag::sync, enter: tag::enter },
        ScreenDef {
            id: Gamefilter,
            mount: gamefilter::mount,
            sync: gamefilter::sync,
            enter: gamefilter::enter,
        },
        ScreenDef { id: Service, mount: service::mount, sync: service::sync, enter: service::enter },
        ScreenDef { id: Lists, mount: lists::mount, sync: lists::sync, enter: lists::enter },
        ScreenDef { id: Extended, mount: extended::mount, sync: extended::sync, enter: extended::enter },
        ScreenDef { id: Ttl, mount: ttl::mount, sync: ttl::sync, enter: ttl::enter },
        ScreenDef { id: Fakes, mount: fakes::mount, sync: fakes::sync, enter: fakes::enter },
        ScreenDef {
            id: FakesSelect,
            mount: fakes::mount_select,
            sync: fakes::sync_select,
            enter: fakes::enter_select,
        },
        ScreenDef { id: Autotune, mount: autotune::mount, sync: autotune::sync, enter: autotune::enter },
        ScreenDef {
            id: NumRequests,
            mount: autotune::mount_requests,
            sync: autotune::sync_requests,
            enter: autotune::enter_requests,
        },
        ScreenDef {
            id: Domains,
            mount: autotune::mount_domains,
            sync: autotune::sync_domains,
            enter: autotune::enter_domains,
        },
        ScreenDef {
            id: Protocols,
            mount: autotune::mount_protocols,
            sync: autotune::sync_protocols,
            enter: autotune::enter_protocols,
        },
        ScreenDef {
            id: BlockChecks,
            mount: autotune::mount_block_checks,
            sync: autotune::sync_block_checks,
            enter: autotune::enter_block_checks,
        },
        ScreenDef {
            id: Presets,
            mount: autotune::mount_presets,
            sync: autotune::sync_presets,
            enter: autotune::enter_presets,
        },
        ScreenDef {
            id: AutotuneStrategies,
            mount: autotune::mount_strategies,
            sync: autotune::sync_strategies,
            enter: autotune::enter_strategies,
        },
        ScreenDef { id: Report, mount: report_screen::mount, sync: report_screen::sync, enter: report_screen::enter },
        #[cfg(target_os = "windows")]
        ScreenDef {
            id: Defender,
            mount: defender::mount,
            sync: defender::sync,
            enter: defender::enter,
        },
    ]
}

/// Downcast a mounted component back to its own type, or give up quietly.
macro_rules! mounted {
    ($app:expr, $id:expr, $ty:ty) => {
        $app.get_component_mut(&$id)
            .and_then(|c| c.as_any_mut().downcast_mut::<$ty>())
    };
}
pub(crate) use mounted;

/// The keys every menu shares, so twenty screens spell them once.
///
/// Two ways to act, kept apart on purpose: **port** changes a value where the
/// cursor already is (`Left`/`Right`), **open** goes somewhere else (`Enter`).
/// A row with a value does the first, a row that is a command does the second,
/// and a screen that wants a row to do both asks for both.
pub fn menu_event(
    ev: &Event<NoUserEvent>,
    list: &mut MenuList,
    port: impl FnOnce(bool) -> Option<Msg>,
    open: impl FnOnce() -> Option<Msg>,
) -> Option<Msg> {
    match ev {
        Event::Keyboard(k) => match k.code {
            Key::Up | Key::Char('k') => list.step(false),
            Key::Down | Key::Char('j') => list.step(true),
            Key::Left | Key::Char('h') => return port(false),
            Key::Right | Key::Char('l') => return port(true),
            Key::Enter | Key::Char(' ') => return open(),
            Key::Esc | Key::Char('q') => return Some(Msg::Back),
            _ => return None,
        },
        Event::Mouse(m) => match m.kind {
            // Moving the pointer over a row puts the cursor there. A heading is
            // scenery: the cursor never lands on one, so it does not move either.
            MouseEventKind::Moved => {
                if let Some(row) = list.row_at(m.column, m.row) {
                    list.select(row);
                }
            }
            // A click is a press, so a row that opens on Enter opens on a click
            // too — otherwise every setting would take two clicks.
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(row) = list.row_at(m.column, m.row) {
                    if list.select(row) {
                        return open();
                    }
                }
                return None;
            }
            MouseEventKind::ScrollUp => list.step(false),
            MouseEventKind::ScrollDown => list.step(true),
            // Drag, the other buttons and the release are not used: there is
            // nothing to drag, and acting on the release too would toggle twice.
            _ => {}
        },
        _ => {}
    }
    Some(Msg::Moved)
}
