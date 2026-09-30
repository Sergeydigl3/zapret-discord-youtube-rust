use crate::menus::{Menu, Row};
use crate::state::screens::DownloadDepsMenuState;
use crate::state::AppState;

pub fn render(app: &AppState) -> Menu {
    let rows = vec![
        Row::new(rust_i18n::t!("menu_dl_zapret")),
        Row::new(rust_i18n::t!("menu_dl_strat")),
        Row::new(rust_i18n::t!("menu_dl_defaults")),
        Row::new(rust_i18n::t!("menu_dl_back")),
    ];

    let index = match app.download_deps_menu {
        DownloadDepsMenuState::ZapretDownloader => 0,
        DownloadDepsMenuState::StrategiesDownloader => 1,
        DownloadDepsMenuState::DownloadDefaults => 2,
        DownloadDepsMenuState::Back => 3,
    };

    Menu::new(rust_i18n::t!("menu_dl_title"), rows).at(index)
}
