//! The downloader screens: category, zapret, strategies, and the two tag pickers.

use crate::state::screens::{ActiveScreen, DownloadDepsMenuState, DownloadSubmenuState, VersionTarget};
use crate::state::AppState;

use super::on_activate;

pub fn activate(app: &mut AppState) {
    match app.active_screen {
        ActiveScreen::DownloadDepsSubmenu => match app.download_deps_menu {
            DownloadDepsMenuState::ZapretDownloader => {
                app.active_screen = ActiveScreen::DownloadZapretSubmenu;
                app.download_zapret_menu = DownloadSubmenuState::Version;
                app.status_message = None;
            }
            DownloadDepsMenuState::StrategiesDownloader => {
                app.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
                app.download_strategies_menu = DownloadSubmenuState::Version;
                app.status_message = None;
            }
            DownloadDepsMenuState::DownloadDefaults => {
                app.should_download_defaults = true;
            }
            DownloadDepsMenuState::Back => {
                app.active_screen = ActiveScreen::Main;
                app.status_message = None;
            }
        },
        ActiveScreen::DownloadZapretSubmenu => match app.download_zapret_menu {
            DownloadSubmenuState::Version => {
                app.nfqws_target = app.nfqws_target.cycle(true);
            }
            DownloadSubmenuState::SelectTag => {
                app.status_message = Some(rust_i18n::t!("msg_fetch_zapret_tags").into_owned());
                match zapret_fetch::fetch_repo_tags("bol-van/zapret") {
                    Ok(tags) => {
                        app.available_nfqws_tags = tags;
                        app.nfqws_tag_index = 0;
                        app.active_screen = ActiveScreen::ZapretTagSelect;
                        app.status_message = None;
                    }
                    Err(e) => {
                        app.show_error(format!("{}{}", rust_i18n::t!("msg_err_fetch_tags"), e));
                    }
                }
            }
            DownloadSubmenuState::Start => {
                app.should_download_zapret = true;
            }
            DownloadSubmenuState::Back => {
                app.active_screen = ActiveScreen::DownloadDepsSubmenu;
                app.status_message = None;
            }
        },
        ActiveScreen::DownloadStrategiesSubmenu => match app.download_strategies_menu {
            DownloadSubmenuState::Version => {
                app.strat_target = app.strat_target.cycle(true);
            }
            DownloadSubmenuState::SelectTag => {
                app.status_message = Some(rust_i18n::t!("msg_fetch_strat_tags").into_owned());
                match zapret_fetch::fetch_repo_tags("Flowseal/zapret-discord-youtube") {
                    Ok(tags) => {
                        app.available_strat_tags = tags;
                        app.strat_tag_index = 0;
                        app.active_screen = ActiveScreen::StrategyTagSelect;
                        app.status_message = None;
                    }
                    Err(e) => {
                        app.show_error(format!("{}{}", rust_i18n::t!("msg_err_fetch_tags"), e));
                    }
                }
            }
            DownloadSubmenuState::Start => {
                app.should_download_strategies = true;
            }
            DownloadSubmenuState::Back => {
                app.active_screen = ActiveScreen::DownloadDepsSubmenu;
                app.status_message = None;
            }
        },
        ActiveScreen::ZapretTagSelect => {
            if app.nfqws_tag_index < app.available_nfqws_tags.len() {
                let selected = app.available_nfqws_tags[app.nfqws_tag_index].clone();
                app.nfqws_target = VersionTarget::Tag(selected);
                app.active_screen = ActiveScreen::DownloadZapretSubmenu;
                app.status_message = Some(rust_i18n::t!("msg_zapret_tag_sel").into_owned());
            } else {
                app.active_screen = ActiveScreen::DownloadZapretSubmenu;
                app.status_message = None;
            }
        }
        ActiveScreen::StrategyTagSelect => {
            if app.strat_tag_index < app.available_strat_tags.len() {
                let selected = app.available_strat_tags[app.strat_tag_index].clone();
                app.strat_target = VersionTarget::Tag(selected);
                app.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
                app.status_message = Some(rust_i18n::t!("msg_strat_tag_sel").into_owned());
            } else {
                app.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
                app.status_message = None;
            }
        }
        _ => unreachable!("download screens only"),
    }
}

pub fn cycle(app: &mut AppState, forward: bool) {
    match app.active_screen {
        // The version row always advances to the next entry, on either arrow.
        ActiveScreen::DownloadZapretSubmenu => match app.download_zapret_menu {
            DownloadSubmenuState::Version => {
                app.nfqws_target = app.nfqws_target.cycle(true);
            }
            _ => {
                if forward {
                    on_activate(app);
                }
            }
        },
        ActiveScreen::DownloadStrategiesSubmenu => match app.download_strategies_menu {
            DownloadSubmenuState::Version => {
                app.strat_target = app.strat_target.cycle(true);
            }
            _ => {
                if forward {
                    on_activate(app);
                }
            }
        },
        _ => {
            if forward {
                on_activate(app);
            }
        }
    }
}
