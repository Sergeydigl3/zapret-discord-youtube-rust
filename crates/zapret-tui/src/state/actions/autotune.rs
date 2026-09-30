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
            // Both of these are values the arrows set; Enter has nothing to add.
            AutotuneMenuState::PresetSelection | AutotuneMenuState::NumRequests => {}
            AutotuneMenuState::Strategies => {
                if app.strategies.is_empty() {
                    app.show_error(rust_i18n::t!("err_no_strats").into_owned());
                } else {
                    app.autotune_strat_index = 0;
                    app.open(ActiveScreen::AutotuneStrategiesSubmenu);
                }
            }
            AutotuneMenuState::Protocols => {
                app.autotune_protocols_menu = AutotuneProtocolsState::Http;
                app.open(ActiveScreen::AutotuneProtocolsSubmenu);
            }
            AutotuneMenuState::BlockChecks => {
                app.autotune_block_checks_menu = AutotuneBlockChecksState::DnsSpoof;
                app.open(ActiveScreen::AutotuneBlockChecksSubmenu);
            }
            AutotuneMenuState::EditDomains => {
                app.domain_files_index = 0;
                app.open(ActiveScreen::AutotuneEditDomainsSubmenu);
            }
            AutotuneMenuState::Results => {
                app.autotune_results_index = 0;
                app.open(ActiveScreen::AutotuneResultsSubmenu);
            }
            AutotuneMenuState::Run => {
                if queue_in_use() {
                    app.status_message = Some(rust_i18n::t!("autotune_err_nfqws_running").into_owned());
                } else {
                    app.should_run_autotune = true;
                }
            }
            AutotuneMenuState::Back => {
                app.back();
            }
        },
        ActiveScreen::AutotuneEditDomainsSubmenu => {
            if app.domain_files_index < app.domain_files.len() {
                app.should_open_editor = Some(app.domain_files[app.domain_files_index].1.clone());
            } else {
                app.back();
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
                app.back();
            }
        },
        ActiveScreen::AutotuneBlockChecksSubmenu => match app.autotune_block_checks_menu.check_index() {
            Some(i) => app.toggle_block_check(i),
            None => {
                app.back();
            }
        },
        ActiveScreen::AutotunePresetSelectionSubmenu => {
            let idx = app.autotune_preset_index;
            if idx >= PRESETS.len() {
                app.back();
                return;
            }
            toggle(&mut app.autotune_config.preset_indices, idx);
            app.status_message = Some(format!(
                "{}: {}",
                rust_i18n::t!("autotune_preset_sel"),
                selected_presets(&app.autotune_config.preset_indices)
            ));
        }
        ActiveScreen::AutotuneStrategiesSubmenu => {
            // The last row is the way out, so a cursor past the strategies is
            // not a strategy that does not exist.
            if app.autotune_strat_index < app.strategies.len() {
                toggle(&mut app.autotune_config.strategy_indices, app.autotune_strat_index);
            } else {
                app.back();
            }
        }
        ActiveScreen::AutotuneResultsSubmenu => {
            app.back();
        }
        _ => unreachable!("autotune screens only"),
    }
}

pub fn cycle(app: &mut AppState, forward: bool) {
    match app.active_screen {
        ActiveScreen::AutotuneSubmenu => match app.autotune_menu {
            AutotuneMenuState::PresetSelection => {
                // Never open past the last preset: the row below them is Back.
                let last = PRESETS.len().saturating_sub(1);
                app.autotune_preset_index = app.autotune_preset_index.min(last);
                app.open(ActiveScreen::AutotunePresetSelectionSubmenu);
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
        ActiveScreen::AutotuneProtocolsSubmenu => on_activate(app),
        ActiveScreen::AutotuneBlockChecksSubmenu => on_activate(app),
        // The multi-selects act on both arrows, exactly as they always have.
        ActiveScreen::AutotunePresetSelectionSubmenu | ActiveScreen::AutotuneStrategiesSubmenu => on_activate(app),
        ActiveScreen::AutotuneResultsSubmenu => {
            app.back();
        }
        _ => {
            if forward {
                on_activate(app);
            }
        }
    }
}

/// Add or remove one index, then put the list back in a predictable order.
fn toggle(indices: &mut Vec<usize>, index: usize) {
    match indices.iter().position(|i| *i == index) {
        Some(pos) => {
            indices.remove(pos);
        }
        None => indices.push(index),
    }
    indices.sort_unstable();
    indices.dedup();
}

/// The presets currently ticked, by name, or a placeholder if none are.
fn selected_presets(indices: &[usize]) -> String {
    let names: Vec<&str> = indices.iter().filter_map(|i| PRESETS.get(*i).map(|p| p.name)).collect();
    if names.is_empty() {
        rust_i18n::t!("menu_autotune_preset_none").into_owned()
    } else {
        names.join(", ")
    }
}
