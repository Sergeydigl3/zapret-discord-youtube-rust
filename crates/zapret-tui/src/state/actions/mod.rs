//! Key dispatch: one handler per screen.
//!
//! The three entry points mirror the three kinds of key the UI reacts to:
//!
//! - [`on_activate`] — Enter / Space. Opens a submenu, toggles a flag, or asks
//!   the session loop to start a long job.
//! - [`on_cycle`] — Left / Right. Changes a value in place. A screen that has
//!   nothing to cycle falls back to activation, which is why a single Right
//!   arrow also opens a submenu.
//! - [`on_back`] — Esc / q. One step up the back stack.
//!
//! Every other module here is named after the screen it serves and is reached
//! only through these routers, so a screen's behaviour lives in exactly one
//! file.

mod autotune;
mod download;
mod extended;
mod fakes;
mod gamefilter;
mod lists;
mod main;
mod mouse;
mod service;
mod strategy;
mod ttl;

#[cfg(target_os = "windows")]
mod defender;

use ratatui::crossterm::event::MouseEvent;

use super::screens::ActiveScreen;
use super::AppState;

/// React to Enter or Space on the current screen.
pub fn on_activate(app: &mut AppState) {
    match app.active_screen {
        ActiveScreen::Main => main::activate(app),
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => defender::activate(app),
        ActiveScreen::StrategySubmenu => strategy::activate(app),
        ActiveScreen::DownloadDepsSubmenu
        | ActiveScreen::DownloadZapretSubmenu
        | ActiveScreen::DownloadStrategiesSubmenu
        | ActiveScreen::ZapretTagSelect
        | ActiveScreen::StrategyTagSelect => download::activate(app),
        ActiveScreen::GamefilterSubmenu => gamefilter::activate(app),
        ActiveScreen::ExtendedSubmenu => extended::activate(app),
        ActiveScreen::TtlSubmenu => ttl::activate(app),
        ActiveScreen::FakesSubmenu | ActiveScreen::FakesSelectSubmenu => fakes::activate(app),
        ActiveScreen::ServiceSubmenu => service::activate(app),
        ActiveScreen::ListsEditorSubmenu => lists::activate(app),
        ActiveScreen::AutotuneSubmenu
        | ActiveScreen::AutotuneEditDomainsSubmenu
        | ActiveScreen::AutotuneProtocolsSubmenu
        | ActiveScreen::AutotuneBlockChecksSubmenu
        | ActiveScreen::AutotunePresetSelectionSubmenu
        | ActiveScreen::AutotuneStrategiesSubmenu
        | ActiveScreen::AutotuneResultsSubmenu => autotune::activate(app),
    }
}

/// React to Left (`forward == false`) or Right (`forward == true`).
pub fn on_cycle(app: &mut AppState, forward: bool) {
    match app.active_screen {
        ActiveScreen::Main => main::cycle(app, forward),
        ActiveScreen::StrategySubmenu => {
            if forward {
                on_activate(app);
            }
        }
        ActiveScreen::DownloadDepsSubmenu
        | ActiveScreen::DownloadZapretSubmenu
        | ActiveScreen::DownloadStrategiesSubmenu
        | ActiveScreen::ZapretTagSelect
        | ActiveScreen::StrategyTagSelect => download::cycle(app, forward),
        ActiveScreen::GamefilterSubmenu => gamefilter::cycle(app, forward),
        ActiveScreen::ExtendedSubmenu => {
            if forward {
                on_activate(app);
            }
        }
        ActiveScreen::TtlSubmenu => ttl::cycle(app, forward),
        ActiveScreen::FakesSubmenu | ActiveScreen::FakesSelectSubmenu => fakes::cycle(app, forward),
        ActiveScreen::ServiceSubmenu => {
            if forward {
                on_activate(app);
            }
        }
        ActiveScreen::ListsEditorSubmenu => {
            if forward {
                on_activate(app);
            }
        }
        ActiveScreen::AutotuneSubmenu
        | ActiveScreen::AutotuneEditDomainsSubmenu
        | ActiveScreen::AutotuneProtocolsSubmenu
        | ActiveScreen::AutotuneBlockChecksSubmenu
        | ActiveScreen::AutotunePresetSelectionSubmenu
        | ActiveScreen::AutotuneStrategiesSubmenu
        | ActiveScreen::AutotuneResultsSubmenu => autotune::cycle(app, forward),
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => {
            if forward {
                on_activate(app);
            }
        }
    }
}

/// React to Esc or q: one level up, or leave the app from the main screen.
///
/// The back stack knows where every screen was opened from, so a submenu
/// three levels down — autotune inside the extended settings — walks out one
/// step at a time without this function needing a row for each of them.
pub fn on_back(app: &mut AppState) {
    if !app.back() {
        app.should_quit = true;
    }
}

/// React to a mouse event.
///
/// A click is a press, so a screen that opens a submenu on Enter opens it on a
/// click too, and the screen it lands on is re-read afterwards the same way a
/// key press re-reads it.
pub fn on_mouse(app: &mut AppState, event: MouseEvent) {
    mouse::on_mouse(app, event);
}

/// Re-read whatever the current screen displays from the outside world.
///
/// The download screens re-check the installed files and the service screen
/// re-checks the service on every key, so an action taken by the previous key
/// (or by a previous visit) shows up immediately.
pub fn refresh_after_key(app: &mut AppState) {
    match app.active_screen {
        ActiveScreen::DownloadDepsSubmenu
        | ActiveScreen::DownloadZapretSubmenu
        | ActiveScreen::DownloadStrategiesSubmenu
        | ActiveScreen::ZapretTagSelect
        | ActiveScreen::StrategyTagSelect => {
            app.refresh_dep_status();
        }
        ActiveScreen::ServiceSubmenu => {
            app.refresh_service_status();
        }
        _ => {}
    }
}
