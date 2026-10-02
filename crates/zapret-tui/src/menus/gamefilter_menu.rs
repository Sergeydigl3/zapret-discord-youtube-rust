use crate::menus::{toggles, Menu, Row};
use crate::state::AppState;

pub fn render(app: &AppState) -> Menu {
    let mut rows = toggles([
        (rust_i18n::t!("menu_gf_tcp").to_string(), app.tcp_gamefilter),
        (rust_i18n::t!("menu_gf_udp").to_string(), app.udp_gamefilter),
    ]);
    rows.push(Row::new(rust_i18n::t!("menu_gf_back")));

    Menu::new(rust_i18n::t!("menu_gf_title"), rows).at(app.gamefilter_menu.index())
}
