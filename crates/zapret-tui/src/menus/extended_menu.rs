//! The Extended submenu: the settings that are real but not part of the
//! everyday run.

use crate::menus::{Menu, Row};
use crate::state::AppState;

pub fn render(app: &AppState) -> Menu {
    let mut rows = vec![
        Row::value(
            rust_i18n::t!("menu_main_ttl"),
            crate::menus::ttl_menu::current_label(app),
        ),
        Row::new(rust_i18n::t!("menu_main_fakes")),
    ];

    #[cfg(target_os = "linux")]
    rows.extend(crate::menus::toggles(vec![(
        rust_i18n::t!("menu_extended_router").into_owned(),
        app.router_mode,
    )]));

    rows.push(Row::new(rust_i18n::t!("menu_extended_back")));

    Menu::new(rust_i18n::t!("menu_extended_title"), rows).at(app.extended_menu.index())
}
