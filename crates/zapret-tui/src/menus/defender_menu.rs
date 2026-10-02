use crate::menus::{Menu, Row};
use crate::state::AppState;
use crate::theme::Theme;

/// Where the cursor sits: the first two rows are scenery.
const FIRST_ACTION: usize = 2;

pub fn render(app: &AppState) -> Menu {
    let status = match app.defender_status_cache {
        Some(true) => rust_i18n::t!("status_def_active"),
        Some(false) => rust_i18n::t!("status_def_inactive"),
        None => rust_i18n::t!("status_def_unknown"),
    };

    // The status line is scenery above the actions, not an action: the cursor
    // starts below it and up/down never lands on it.
    let rows = vec![
        Row::value(rust_i18n::t!("status_def_curr"), status).styled_value(Theme::accent()),
        Row::new(""),
        Row::new(rust_i18n::t!("menu_def_add")),
        Row::new(rust_i18n::t!("menu_def_remove")),
        Row::new(rust_i18n::t!("menu_def_back")),
    ];

    Menu::new(rust_i18n::t!("menu_def_title"), rows).at(FIRST_ACTION + app.defender_menu.index())
}
