//! The TTL submenu: leave it alone, pin a number, or sweep for one.

use zapret_wrapper::config;
use zapret_wrapper::domains::ttl;

use crate::state::screens::{ActiveScreen, TtlMenuState};
use crate::state::AppState;

/// Enter / Space on the TTL submenu.
pub fn activate(app: &mut AppState) {
    match app.ttl_menu {
        TtlMenuState::DontTouch => {
            set_ttl(app, None);
            app.status_message = Some(rust_i18n::t!("ttl_set_off").into_owned());
        }
        TtlMenuState::SetValue => {
            // The arrows already set it; there is nothing to confirm.
            app.active_screen = ActiveScreen::Main;
            app.status_message = None;
        }
        TtlMenuState::Autopick => {
            if app.strategies.is_empty() {
                app.show_error(rust_i18n::t!("err_no_strats").into_owned());
                return;
            }
            if app.check_dependencies() {
                app.should_run_ttl = true;
            }
        }
        TtlMenuState::Back => {
            app.active_screen = ActiveScreen::Main;
            app.status_message = None;
        }
    }
}

/// Left / Right. Only the "set a number" row has something to cycle.
pub fn cycle(app: &mut AppState, forward: bool) {
    if app.ttl_menu != TtlMenuState::SetValue {
        if forward {
            activate(app);
        }
        return;
    }
    // The same range the sweep walks, so a value picked by hand and a value
    // found by the sweep are comparable.
    let min = ttl::TTL_MIN as u32;
    let max = ttl::TTL_MAX as u32;
    // Starting from "off" the first step lands on the bottom of the range, and
    // a value saved before the range moved is snapped into it.
    let current = app
        .dpi_desync_ttl
        .map_or(min.saturating_sub(1), |v| (v as u32).max(min.saturating_sub(1)));
    let next = if forward {
        if current >= max {
            min
        } else {
            current + 1
        }
    } else if current <= min {
        min
    } else {
        current - 1
    };
    set_ttl(app, Some(next as u8));
}

/// Write the setting through to the config, so it survives a restart.
fn set_ttl(app: &mut AppState, value: Option<u8>) {
    app.dpi_desync_ttl = value;
    let _ = config::save_ttl(value);
}
