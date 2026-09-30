//! The Windows Defender exclusion screen.

use zapret_core::defender;

use crate::state::screens::{ActiveScreen, DefenderMenuState};
use crate::state::AppState;

pub fn activate(app: &mut AppState) {
    match app.defender_menu {
        DefenderMenuState::Add => match defender::add_defender_exclusion() {
            Ok(_) => {
                app.status_message = Some(rust_i18n::t!("msg_def_add_ok").into_owned());
                app.refresh_defender_status();
            }
            Err(e) => app.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), e)),
        },
        DefenderMenuState::Remove => match defender::remove_defender_exclusion() {
            Ok(_) => {
                app.status_message = Some(rust_i18n::t!("msg_def_rm_ok").into_owned());
                app.refresh_defender_status();
            }
            Err(e) => app.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), e)),
        },
        DefenderMenuState::Back => {
            app.active_screen = ActiveScreen::Main;
            app.status_message = None;
        }
    }
}
