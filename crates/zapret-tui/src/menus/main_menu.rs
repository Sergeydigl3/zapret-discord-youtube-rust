//! The main menu.
//!
//! The rows come from `MainMenuState::GROUPS` and the cursor from that same
//! list's index, so a row cannot exist in the menu without being reachable by
//! the arrows, or vice versa. The headings between the groups are drawn from
//! the same place and are not part of the cursor.

use crate::menus::{Menu, Row};
use crate::state::screens::MainMenuState;
use crate::state::AppState;

pub fn render(app: &AppState) -> Menu {
    let mut rows: Vec<Row> = Vec::new();
    for group in MainMenuState::GROUPS {
        if !group.title.is_empty() {
            rows.push(Row::heading(rust_i18n::t!(group.title).into_owned()));
        }
        for state in group.rows {
            let label = rust_i18n::t!(label_key(*state)).into_owned();
            rows.push(match value_of(app, *state) {
                Some(value) => Row::value(label, format!("< {value} >")),
                None => Row::new(label),
            });
        }
    }

    // The cursor is an index into the *selectable* rows, so it has to skip the
    // headings the loop above just added.
    Menu::new(rust_i18n::t!("menu_main_title"), rows).at(row_of(app.main_menu))
}

/// Where a row ends up once the headings are counted.
///
/// `MainMenuState::index` cannot answer this on its own: it counts selectable
/// rows, and the drawn list has a heading in front of every group but the last.
fn row_of(state: MainMenuState) -> usize {
    let mut row = 0usize;
    for group in MainMenuState::GROUPS {
        if !group.title.is_empty() {
            row += 1;
        }
        for candidate in group.rows {
            if *candidate == state {
                return row;
            }
            row += 1;
        }
    }
    0
}

/// The state on a drawn row, headings included. The inverse of [`row_of`], and
/// what a click on the main menu is turned into.
pub fn state_at(row: usize) -> Option<MainMenuState> {
    let mut at = row;
    for group in MainMenuState::GROUPS {
        if !group.title.is_empty() {
            if at == 0 {
                return None;
            }
            at -= 1;
        }
        let len = group.rows.len();
        if at < len {
            return Some(group.rows[at]);
        }
        at -= len;
    }
    None
}

/// The locale key of a row's name.
///
/// The names themselves live next to the row in [`MainMenuState`]; this is only
/// the one line that has to know which key goes with which variant, and the
/// match is over the same list the menu is drawn from.
fn label_key(state: MainMenuState) -> &'static str {
    match state {
        #[cfg(target_os = "windows")]
        MainMenuState::DefenderSettings => "menu_main_defender",
        MainMenuState::DownloadDeps => "menu_main_downloader",
        #[cfg(target_os = "linux")]
        MainMenuState::Interface => "menu_main_interface",
        MainMenuState::Strategy => "menu_main_strategy",
        MainMenuState::GamefilterSettings => "menu_main_gamefilter",
        #[cfg(target_os = "linux")]
        MainMenuState::BackendSettings => "menu_main_backend",
        MainMenuState::IpsetMode => "menu_main_ipset",
        MainMenuState::ListsEditor => "menu_main_lists",
        MainMenuState::Autotune => "menu_main_autotune",
        MainMenuState::Extended => "menu_main_extended",
        MainMenuState::ServiceSettings => "menu_main_service",
        MainMenuState::Run => "menu_main_run",
        MainMenuState::Quit => "menu_main_quit",
    }
}

/// What a row is showing, or `None` for a row that only does something.
fn value_of(app: &AppState, state: MainMenuState) -> Option<String> {
    match state {
        #[cfg(target_os = "windows")]
        MainMenuState::DefenderSettings => None,
        MainMenuState::DownloadDeps => None,
        #[cfg(target_os = "linux")]
        MainMenuState::Interface => Some(app.interface().to_string()),
        MainMenuState::Strategy => Some(app.strategies.get(app.selected_strategy).cloned().unwrap_or_default()),
        MainMenuState::GamefilterSettings => {
            let mut parts: Vec<&str> = Vec::new();
            if app.tcp_gamefilter {
                parts.push("TCP");
            }
            if app.udp_gamefilter {
                parts.push("UDP");
            }
            Some(if parts.is_empty() {
                rust_i18n::t!("val_off").into_owned()
            } else {
                parts.join("+")
            })
        }
        #[cfg(target_os = "linux")]
        MainMenuState::BackendSettings => Some(app.selected_backend.to_string()),
        MainMenuState::IpsetMode => Some(
            app.available_ipset_modes
                .get(app.selected_ipset_mode)
                .map(|m| m.to_string())
                .unwrap_or_else(|| rust_i18n::t!("val_none").into_owned()),
        ),
        MainMenuState::ListsEditor => None,
        MainMenuState::Autotune => None,
        MainMenuState::Extended => None,
        MainMenuState::ServiceSettings => None,
        MainMenuState::Run => None,
        MainMenuState::Quit => None,
    }
}
