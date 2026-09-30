//! The three decisions about the fixed DPI-TTL.

use crate::menus::{Menu, Row};
use crate::state::AppState;

/// A short label for the current setting: the hop count, or "off".
pub fn current_label(app: &AppState) -> String {
    match app.dpi_desync_ttl {
        Some(n) => n.to_string(),
        None => rust_i18n::t!("ttl_dont_touch_short").into_owned(),
    }
}

pub fn render(app: &AppState) -> Menu {
    let value = current_label(app);

    let rows = vec![
        Row::value(rust_i18n::t!("menu_ttl_dont_touch"), value.clone()),
        // Only the row that actually takes a number is wrapped in arrows, so a
        // value is never mistaken for something you can nudge here.
        Row::value(rust_i18n::t!("menu_ttl_set_value"), format!("< {value} >")),
        Row::new(rust_i18n::t!("menu_ttl_autopick")),
        Row::new(rust_i18n::t!("menu_ttl_back")),
    ];

    Menu::new(rust_i18n::t!("tui_title_ttl"), rows).at(app.ttl_menu.index())
}
