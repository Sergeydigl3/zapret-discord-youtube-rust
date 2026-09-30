use crate::menus::{Menu, Row};
use crate::state::AppState;

pub fn render(app: &AppState) -> Menu {
    let rows: Vec<Row> = if !app.service_installed {
        vec![
            Row::new(rust_i18n::t!("menu_srv_install")),
            Row::new(rust_i18n::t!("menu_srv_back")),
        ]
    } else if app.service_active {
        vec![
            Row::new(rust_i18n::t!("menu_srv_stop")),
            Row::new(rust_i18n::t!("menu_srv_restart")),
            Row::new(rust_i18n::t!("menu_srv_uninstall")),
            Row::new(rust_i18n::t!("menu_srv_back")),
        ]
    } else {
        vec![
            Row::new(rust_i18n::t!("menu_srv_start")),
            Row::new(rust_i18n::t!("menu_srv_uninstall")),
            Row::new(rust_i18n::t!("menu_srv_back")),
        ]
    };

    Menu::new(rust_i18n::t!("menu_srv_title"), rows).at(app.service_menu_index)
}
