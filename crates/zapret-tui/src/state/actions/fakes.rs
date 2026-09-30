//! The fake-payload screens: pick a target, then pick a `.bin` file for it.

use zapret_wrapper::fakes;

use crate::state::screens::{ActiveScreen, FakesMenuState, FakesSelectTarget};
use crate::state::AppState;

use super::on_activate;

pub fn activate(app: &mut AppState) {
    match app.active_screen {
        ActiveScreen::FakesSubmenu => match app.fakes_menu {
            FakesMenuState::DiscordUdp => {
                app.fakes_select_for = FakesSelectTarget::DiscordUdp;
                app.fakes_select_index = if app.fakes_state.available.is_empty() { 0 } else { 1 };
                app.active_screen = ActiveScreen::FakesSelectSubmenu;
                app.status_message = None;
            }
            FakesMenuState::GameUdp => {
                app.fakes_select_for = FakesSelectTarget::GameUdp;
                app.fakes_select_index = if app.fakes_state.available.is_empty() { 0 } else { 1 };
                app.active_screen = ActiveScreen::FakesSelectSubmenu;
                app.status_message = None;
            }
            FakesMenuState::Back => {
                app.active_screen = ActiveScreen::Main;
                app.status_message = None;
            }
        },
        ActiveScreen::FakesSelectSubmenu => {
            let file_count = app.fakes_state.available.len();
            if app.fakes_select_index >= 1 && app.fakes_select_index <= file_count {
                let source_idx = app.fakes_select_index - 1;
                let source = app.fakes_state.available[source_idx].clone();
                let target: fakes::FakeTarget = match app.fakes_select_for {
                    FakesSelectTarget::DiscordUdp => fakes::FakeTarget::DiscordUdp,
                    FakesSelectTarget::GameUdp => fakes::FakeTarget::GameUdp,
                };
                match fakes::replace_active_fake(&app.fakes_state, &target, &source) {
                    Ok(()) => {
                        app.fakes_state = fakes::load_fakes_state();
                        app.active_screen = ActiveScreen::FakesSubmenu;
                        app.status_message = Some(rust_i18n::t!("msg_fakes_replaced").into_owned());
                    }
                    Err(e) => {
                        app.show_error(e);
                    }
                }
            } else {
                app.active_screen = ActiveScreen::FakesSubmenu;
                app.status_message = None;
            }
        }
        _ => unreachable!("fakes screens only"),
    }
}

pub fn cycle(app: &mut AppState, forward: bool) {
    match app.active_screen {
        // The file list acts on both arrows, exactly as it always has.
        ActiveScreen::FakesSelectSubmenu => on_activate(app),
        _ => {
            if forward {
                on_activate(app);
            }
        }
    }
}
