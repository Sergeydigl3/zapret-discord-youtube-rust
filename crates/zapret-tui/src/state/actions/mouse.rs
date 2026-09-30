//! The mouse, in terms of what it points at.
//!
//! Three gestures, each the same gesture the keyboard already has:
//!
//! - Moving the pointer over a row puts the cursor there. Nothing happens yet.
//! - Clicking a row puts the cursor there and presses it. A click on a setting
//!   row changes it, which is what a click on a radio button does; making the
//!   first click only select would mean every setting takes two clicks.
//! - The wheel moves the cursor on a menu and scrolls the report.
//!
//! Rows that are not rows — a section heading, the frame, the help line — are
//! scenery. The cursor never lands on one, so a click on one does nothing.
//!
//! A row that comes back from the hit map was drawn this frame, so it always
//! exists and no bounds check is needed: the map is the list of rows that are on
//! screen, not a rectangle of coordinates to be validated.

use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

use crate::menus;
use crate::mouse::WHEEL_LINES;
#[cfg(target_os = "windows")]
use crate::state::screens::DefenderMenuState;
use crate::state::screens::{
    ActiveScreen, AutotuneBlockChecksState, AutotuneMenuState, AutotuneProtocolsState, DownloadDepsMenuState,
    DownloadSubmenuState, ExtendedMenuState, FakesMenuState, GamefilterMenuState, TtlMenuState,
};
use crate::state::AppState;

use super::on_activate;

pub fn on_mouse(app: &mut AppState, event: MouseEvent) {
    match event.kind {
        MouseEventKind::Moved => {
            if let Some(row) = app.hit.row_at(event.column, event.row) {
                cursor_at(app, row);
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            // A row that is not selectable — a section heading — falls through
            // without being pressed.
            if let Some(row) = app.hit.row_at(event.column, event.row) {
                if cursor_at(app, row) {
                    on_activate(app);
                }
            }
        }
        MouseEventKind::ScrollUp => wheel(app, event, false),
        MouseEventKind::ScrollDown => wheel(app, event, true),
        // Drag, the other buttons and the release are not used: there is nothing
        // to drag, and acting on the release as well would toggle a setting
        // twice.
        _ => {}
    }
}

fn wheel(app: &mut AppState, event: MouseEvent, forward: bool) {
    if app.hit.scrolls_at(event.column, event.row) {
        app.scroll_report(forward, WHEEL_LINES);
    } else {
        app.move_menu(forward);
    }
}

/// Put the cursor on a drawn row. False when that row cannot be selected.
///
/// This is the inverse of drawing: menus are drawn from the cursor's own list, so
/// a click has to be turned back into a cursor position before it can mean
/// anything. One function rather than one per menu, because it is one lookup and
/// fourteen functions would drift apart.
///
/// The awkward screens are the two kinds: the ones that draw scenery above their
/// first real row, where the drawn row number is not the cursor number, and the
/// main menu, where the headings do the same.
fn cursor_at(app: &mut AppState, row: usize) -> bool {
    match app.active_screen {
        ActiveScreen::Main => match menus::main_menu::state_at(row) {
            Some(state) => {
                app.main_menu = state;
                true
            }
            None => false,
        },

        // The Defender status line and the gap under it are drawn above the
        // actions, and the cursor starts below them.
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => match row.checked_sub(2).and_then(|i| DefenderMenuState::ALL.get(i)) {
            Some(state) => {
                app.defender_menu = *state;
                true
            }
            None => false,
        },

        // Row 0 shows which fake is in use; the files and the way back are rows
        // 1 and up, and the file list already numbers itself that way.
        ActiveScreen::FakesSelectSubmenu => {
            if row == 0 {
                false
            } else {
                app.fakes_select_index = row;
                true
            }
        }

        ActiveScreen::ExtendedSubmenu => set(&ExtendedMenuState::ALL, row, |s| app.extended_menu = s),
        ActiveScreen::TtlSubmenu => set(&TtlMenuState::ALL, row, |s| app.ttl_menu = s),
        ActiveScreen::GamefilterSubmenu => set(&GamefilterMenuState::ALL, row, |s| app.gamefilter_menu = s),
        ActiveScreen::FakesSubmenu => set(&FakesMenuState::ALL, row, |s| app.fakes_menu = s),
        ActiveScreen::AutotuneSubmenu => set(&AutotuneMenuState::ALL, row, |s| {
            app.autotune_menu = s;
            app.autotune_menu_index = s.index();
        }),
        ActiveScreen::AutotuneProtocolsSubmenu => {
            set(&AutotuneProtocolsState::ALL, row, |s| app.autotune_protocols_menu = s)
        }
        ActiveScreen::AutotuneBlockChecksSubmenu => set(&AutotuneBlockChecksState::ALL, row, |s| {
            app.autotune_block_checks_menu = s
        }),
        ActiveScreen::DownloadDepsSubmenu => set(&DownloadDepsMenuState::ALL, row, |s| app.download_deps_menu = s),
        ActiveScreen::DownloadZapretSubmenu => set(&DownloadSubmenuState::ALL, row, |s| app.download_zapret_menu = s),
        ActiveScreen::DownloadStrategiesSubmenu => {
            set(&DownloadSubmenuState::ALL, row, |s| app.download_strategies_menu = s)
        }

        // Everything else draws its rows in cursor order, so the drawn row *is*
        // the cursor.
        ActiveScreen::StrategySubmenu => {
            app.strategy_menu_index = row;
            true
        }
        ActiveScreen::ZapretTagSelect => {
            app.nfqws_tag_index = row;
            true
        }
        ActiveScreen::StrategyTagSelect => {
            app.strat_tag_index = row;
            true
        }
        ActiveScreen::ServiceSubmenu => {
            app.service_menu_index = row;
            true
        }
        ActiveScreen::ListsEditorSubmenu => {
            app.lists_menu_index = row;
            true
        }
        ActiveScreen::AutotuneEditDomainsSubmenu => {
            app.domain_files_index = row;
            true
        }
        ActiveScreen::AutotunePresetSelectionSubmenu => {
            app.autotune_preset_index = row;
            true
        }
        ActiveScreen::AutotuneStrategiesSubmenu => {
            app.autotune_strat_index = row;
            true
        }

        // The report has no cursor: it scrolls, and the wheel is the only thing
        // that means anything on it.
        ActiveScreen::AutotuneResultsSubmenu => false,
    }
}

/// Put a cursor that is an enum on `all[row]`, if the row is one of them.
fn set<T: Copy>(all: &[T], row: usize, put: impl FnOnce(T)) -> bool {
    match all.get(row) {
        Some(state) => {
            put(*state);
            true
        }
        None => false,
    }
}
