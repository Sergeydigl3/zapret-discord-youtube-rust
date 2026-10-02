use crate::menus::{Menu, Row};
use crate::state::AppState;

pub fn render(app: &AppState) -> Menu {
    let rows = vec![
        Row::new(rust_i18n::t!("menu_dl_zapret")),
        Row::new(rust_i18n::t!("menu_dl_strat")),
        Row::new(rust_i18n::t!("menu_dl_defaults")),
        Row::new(rust_i18n::t!("menu_dl_back")),
    ];

    Menu::new(rust_i18n::t!("menu_dl_title"), rows).at(app.download_deps_menu.index())
}
