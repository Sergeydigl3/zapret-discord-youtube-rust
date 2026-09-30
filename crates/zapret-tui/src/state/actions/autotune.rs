//! The autotune screens: the menu, the domain-file editor, the protocol and
//! block-check toggles, the preset and strategy multi-selects, and the results.

use zapret_wrapper::domains::PRESETS;
use zapret_wrapper::run::queue_in_use;

use crate::state::screens::{ActiveScreen, AutotuneBlockChecksState, AutotuneMenuState, AutotuneProtocolsState};
use crate::state::AppState;

use super::on_activate;

pub fn activate(app: &mut AppState) {
    match app.active_screen {
        ActiveScreen::AutotuneSubmenu => match app.autotune_menu {
            AutotuneMenuState::PresetSelection => {}
            AutotuneMenuState::NumRequests => {}
            AutotuneMenuState::Strategies => {
                if !app.strategies.is_empty() {
                    app.active_screen = ActiveScreen::AutotuneStrategiesSubmenu;
                    app.autotune_strat_index = 0;
                    app.status_message = None;
                } else {
                    app.show_error(rust_i18n::t!("err_no_strats").into_owned());
                }
            }
            AutotuneMenuState::Protocols => {
                app.active_screen = ActiveScreen::AutotuneProtocolsSubmenu;
                app.autotune_protocols_menu = AutotuneProtocolsState::Http;
                app.status_message = None;
            }
            AutotuneMenuState::BlockChecks => {
                app.active_screen = ActiveScreen::AutotuneBlockChecksSubmenu;
                app.autotune_block_checks_menu = AutotuneBlockChecksState::DnsSpoof;
                app.status_message = None;
            }
            AutotuneMenuState::EditDomains => {
                app.active_screen = ActiveScreen::AutotuneEditDomainsSubmenu;
                app.domain_files_index = 0;
                app.status_message = None;
            }
            AutotuneMenuState::Results => {
                app.active_screen = ActiveScreen::AutotuneResultsSubmenu;
                app.autotune_results_index = 0;
                app.status_message = None;
            }
            AutotuneMenuState::Run => {
                if queue_in_use() {
                    app.status_message = Some(rust_i18n::t!("autotune_err_nfqws_running").into_owned());
                } else {
                    app.should_run_autotune = true;
                }
            }
            AutotuneMenuState::Back => {
                app.active_screen = ActiveScreen::Main;
                app.status_message = None;
            }
        },
        ActiveScreen::AutotuneEditDomainsSubmenu => {
            if app.domain_files_index < app.domain_files.len() {
                let file = app.domain_files[app.domain_files_index].1.clone();
                app.should_open_editor = Some(file);
            } else {
                app.active_screen = ActiveScreen::AutotuneSubmenu;
                app.status_message = None;
            }
        }
        ActiveScreen::AutotuneProtocolsSubmenu => match app.autotune_protocols_menu {
            AutotuneProtocolsState::Http => {
                app.autotune_config.check_http = !app.autotune_config.check_http;
            }
            AutotuneProtocolsState::Tls12 => {
                app.autotune_config.check_tls12 = !app.autotune_config.check_tls12;
            }
            AutotuneProtocolsState::Tls13 => {
                app.autotune_config.check_tls13 = !app.autotune_config.check_tls13;
            }
            AutotuneProtocolsState::Quic => {
                app.autotune_config.check_quic = !app.autotune_config.check_quic;
            }
            AutotuneProtocolsState::Back => {
                app.active_screen = ActiveScreen::AutotuneSubmenu;
                app.status_message = None;
            }
        },
        ActiveScreen::AutotuneBlockChecksSubmenu => {
            if app.autotune_block_checks_menu == AutotuneBlockChecksState::Back {
                app.active_screen = ActiveScreen::AutotuneSubmenu;
                app.status_message = None;
            } else {
                app.toggle_block_check(app.autotune_block_checks_menu.index());
            }
        }
        ActiveScreen::AutotunePresetSelectionSubmenu => {
            let idx = app.autotune_preset_index;
            if idx >= PRESETS.len() {
                app.active_screen = ActiveScreen::AutotuneSubmenu;
                app.status_message = None;
                return;
            }
            let was_selected = app.autotune_config.preset_indices.contains(&idx);
            if was_selected {
                app.autotune_config.preset_indices.retain(|&i| i != idx);
            } else {
                app.autotune_config.preset_indices.push(idx);
            }
            app.autotune_config.preset_indices.sort();
            app.autotune_config.preset_indices.dedup();
            let names: Vec<&str> = app
                .autotune_config
                .preset_indices
                .iter()
                .filter_map(|&i| if i < PRESETS.len() { Some(PRESETS[i].name) } else { None })
                .collect();
            let label = if names.is_empty() {
                rust_i18n::t!("menu_autotune_preset_none").into_owned()
            } else {
                names.join(", ")
            };
            app.status_message = Some(format!("{}: {}", rust_i18n::t!("autotune_preset_sel"), label));
        }
        ActiveScreen::AutotuneStrategiesSubmenu => {
            let max = app.strategies.len(); // +1 for Back
            if app.autotune_strat_index < max {
                // Toggle strategy selection
                let idx = app.autotune_strat_index;
                if let Some(pos) = app.autotune_config.strategy_indices.iter().position(|&i| i == idx) {
                    app.autotune_config.strategy_indices.remove(pos);
                } else {
                    app.autotune_config.strategy_indices.push(idx);
                }
            } else {
                // Back
                app.active_screen = ActiveScreen::AutotuneSubmenu;
                app.status_message = None;
            }
        }
        ActiveScreen::AutotuneResultsSubmenu => {
            app.active_screen = ActiveScreen::AutotuneSubmenu;
            app.status_message = None;
        }
        _ => unreachable!("autotune screens only"),
    }
}

pub fn cycle(app: &mut AppState, forward: bool) {
    match app.active_screen {
        ActiveScreen::AutotuneSubmenu => match app.autotune_menu {
            AutotuneMenuState::PresetSelection => {
                app.active_screen = ActiveScreen::AutotunePresetSelectionSubmenu;
                let count = PRESETS.len();
                if app.autotune_preset_index >= count {
                    app.autotune_preset_index = count - 1;
                }
                app.status_message = None;
            }
            AutotuneMenuState::NumRequests => {
                if forward {
                    app.autotune_request_buf = app.autotune_config.num_requests.to_string();
                    app.autotune_request_editing = true;
                }
            }
            _ => {
                if forward {
                    on_activate(app);
                }
            }
        },
        ActiveScreen::AutotuneProtocolsSubmenu => match app.autotune_protocols_menu {
            AutotuneProtocolsState::Http => {
                app.autotune_config.check_http = !app.autotune_config.check_http;
            }
            AutotuneProtocolsState::Tls12 => {
                app.autotune_config.check_tls12 = !app.autotune_config.check_tls12;
            }
            AutotuneProtocolsState::Tls13 => {
                app.autotune_config.check_tls13 = !app.autotune_config.check_tls13;
            }
            AutotuneProtocolsState::Quic => {
                app.autotune_config.check_quic = !app.autotune_config.check_quic;
            }
            _ => {
                if forward {
                    on_activate(app);
                }
            }
        },
        ActiveScreen::AutotuneBlockChecksSubmenu => match app.autotune_block_checks_menu {
            AutotuneBlockChecksState::Back => {
                app.active_screen = ActiveScreen::AutotuneSubmenu;
                app.status_message = None;
            }
            _ => {
                on_activate(app);
            }
        },
        // The multi-selects act on both arrows, exactly as they always have.
        ActiveScreen::AutotunePresetSelectionSubmenu | ActiveScreen::AutotuneStrategiesSubmenu => {
            on_activate(app);
        }
        ActiveScreen::AutotuneResultsSubmenu => {
            app.active_screen = ActiveScreen::AutotuneSubmenu;
            app.status_message = None;
        }
        _ => {
            if forward {
                on_activate(app);
            }
        }
    }
}
