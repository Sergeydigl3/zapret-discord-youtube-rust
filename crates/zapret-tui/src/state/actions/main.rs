//! The main menu.

use zapret_wrapper::lists;

#[cfg(target_os = "linux")]
use zapret_core::firewall::LinuxBackend;

use crate::state::screens::{ActiveScreen, AutotuneMenuState, GamefilterMenuState, MainMenuState};
use crate::state::AppState;

use super::on_activate;

pub fn activate(app: &mut AppState) {
    match app.main_menu {
        #[cfg(target_os = "windows")]
        MainMenuState::DefenderSettings => {
            app.defender_menu = crate::state::screens::DefenderMenuState::Add;
            app.open(ActiveScreen::DefenderSubmenu);
            app.refresh_defender_status();
        }
        MainMenuState::DownloadDeps => {
            app.open(ActiveScreen::DownloadDepsSubmenu);
        }
        #[cfg(target_os = "linux")]
        MainMenuState::Interface => {
            if !app.interfaces.is_empty() {
                app.selected_interface = (app.selected_interface + 1) % app.interfaces.len();
                app.save_current_config();
            }
        }
        #[cfg(target_os = "linux")]
        MainMenuState::BackendSettings => {
            let backends = LinuxBackend::variants();
            if !backends.is_empty() {
                let current_idx = backends.iter().position(|b| *b == app.selected_backend).unwrap_or(0);
                app.selected_backend = backends[(current_idx + 1) % backends.len()];
                app.save_current_config();
            }
        }
        MainMenuState::IpsetMode => {
            if !app.available_ipset_modes.is_empty() {
                let old_mode = app.available_ipset_modes[app.selected_ipset_mode];
                app.selected_ipset_mode = (app.selected_ipset_mode + 1) % app.available_ipset_modes.len();
                let new_mode = app.available_ipset_modes[app.selected_ipset_mode];
                lists::apply_ipset_mode(old_mode, new_mode);
                app.available_ipset_modes = lists::get_available_modes();
                app.selected_ipset_mode = app
                    .available_ipset_modes
                    .iter()
                    .position(|m| m == &new_mode)
                    .unwrap_or(0);
            }
        }
        MainMenuState::Strategy => {
            app.open(ActiveScreen::StrategySubmenu);
            app.strategy_menu_index = app.selected_strategy;
        }
        MainMenuState::GamefilterSettings => {
            app.open(ActiveScreen::GamefilterSubmenu);
            app.gamefilter_menu = GamefilterMenuState::Tcp;
        }
        MainMenuState::ServiceSettings => {
            app.open(ActiveScreen::ServiceSubmenu);
            app.service_menu_index = 0;
            app.refresh_service_status();
        }
        MainMenuState::ListsEditor => {
            if !zapret_wrapper::paths::strategies_installed() {
                app.show_error(rust_i18n::t!("err_no_strats").into_owned());
            } else {
                app.lists_files = lists::get_lists_files();
                app.lists_menu_index = 0;
                app.open(ActiveScreen::ListsEditorSubmenu);
            }
        }
        MainMenuState::Autotune => {
            app.open(ActiveScreen::AutotuneSubmenu);
            app.autotune_menu = AutotuneMenuState::PresetSelection;
            app.has_autotune_results_file = zapret_wrapper::autotune::load_results_file().is_some();
        }
        MainMenuState::Extended => {
            app.open(ActiveScreen::ExtendedSubmenu);
            app.extended_menu = crate::state::screens::ExtendedMenuState::Ttl;
        }
        MainMenuState::Run => {
            if app.check_dependencies() {
                app.should_run = true;
            }
        }
        MainMenuState::Quit => app.should_quit = true,
    }
}

pub fn cycle(app: &mut AppState, forward: bool) {
    match app.main_menu {
        #[cfg(target_os = "linux")]
        MainMenuState::Interface => {
            if !app.interfaces.is_empty() {
                let len = app.interfaces.len();
                app.selected_interface = if forward {
                    (app.selected_interface + 1) % len
                } else {
                    (app.selected_interface + len - 1) % len
                };
                app.save_current_config();
            }
        }
        #[cfg(target_os = "linux")]
        MainMenuState::BackendSettings => {
            let backends = LinuxBackend::variants();
            if !backends.is_empty() {
                let current_idx = backends.iter().position(|b| *b == app.selected_backend).unwrap_or(0);
                let len = backends.len();
                app.selected_backend = if forward {
                    backends[(current_idx + 1) % len]
                } else {
                    backends[(current_idx + len - 1) % len]
                };
                app.save_current_config();
            }
        }
        MainMenuState::IpsetMode => {
            if !app.available_ipset_modes.is_empty() {
                let len = app.available_ipset_modes.len();
                let old_mode = app.available_ipset_modes[app.selected_ipset_mode];
                app.selected_ipset_mode = if forward {
                    (app.selected_ipset_mode + 1) % len
                } else {
                    (app.selected_ipset_mode + len - 1) % len
                };
                let new_mode = app.available_ipset_modes[app.selected_ipset_mode];
                lists::apply_ipset_mode(old_mode, new_mode);
                app.available_ipset_modes = lists::get_available_modes();
                app.selected_ipset_mode = app
                    .available_ipset_modes
                    .iter()
                    .position(|m| m == &new_mode)
                    .unwrap_or(0);
            }
        }
        _ => {
            if forward {
                on_activate(app);
            }
        }
    }
}
