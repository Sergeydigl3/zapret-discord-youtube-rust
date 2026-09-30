//! The fake-payload screens: pick a target, then pick a `.bin` file for it.

use zapret_wrapper::fakes;

use crate::state::screens::{ActiveScreen, FakesMenuState, FakesSelectTarget};
use crate::state::AppState;

use super::on_activate;

pub fn activate(app: &mut AppState) {
    match app.active_screen {
        ActiveScreen::FakesSubmenu => match app.fakes_menu {
            FakesMenuState::DiscordUdp => {
                open_file_list(app, FakesSelectTarget::DiscordUdp);
            }
            FakesMenuState::GameUdp => {
                open_file_list(app, FakesSelectTarget::GameUdp);
            }
            FakesMenuState::Back => {
                app.back();
            }
        },
        ActiveScreen::FakesSelectSubmenu => {
            // Row 0 is the header showing what is in use, so the first file is
            // row 1 and the back row is one past the last file.
            let file_count = app.fakes_state.available.len();
            if app.fakes_select_index >= 1 && app.fakes_select_index <= file_count {
                let source = app.fakes_state.available[app.fakes_select_index - 1].clone();
                let target = match app.fakes_select_for {
                    FakesSelectTarget::DiscordUdp => fakes::FakeTarget::DiscordUdp,
                    FakesSelectTarget::GameUdp => fakes::FakeTarget::GameUdp,
                };
                match fakes::replace_active_fake(&app.fakes_state, &target, &source) {
                    Ok(()) => {
                        app.fakes_state = fakes::load_fakes_state();
                        app.back();
                        app.status_message = Some(rust_i18n::t!("msg_fakes_replaced").into_owned());
                    }
                    Err(e) => app.show_error(e),
                }
            } else {
                app.back();
            }
        }
        _ => unreachable!("fakes screens only"),
    }
}

/// Land on the first file rather than on the header above it.
fn open_file_list(app: &mut AppState, for_target: FakesSelectTarget) {
    app.fakes_select_for = for_target;
    app.fakes_select_index = usize::from(app.fakes_state.available.is_empty());
    app.open(ActiveScreen::FakesSelectSubmenu);
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
