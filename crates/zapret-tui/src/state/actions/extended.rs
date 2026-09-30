//! The Extended submenu: the settings that are real but not part of the
//! everyday run.

use crate::state::screens::{ActiveScreen, ExtendedMenuState, FakesMenuState, TtlMenuState};
use crate::state::AppState;

pub fn activate(app: &mut AppState) {
    match app.extended_menu {
        ExtendedMenuState::Ttl => {
            app.open(ActiveScreen::TtlSubmenu);
            app.ttl_menu = TtlMenuState::DontTouch;
        }
        ExtendedMenuState::Fakes => {
            app.fakes_state = zapret_wrapper::fakes::load_fakes_state();
            app.open(ActiveScreen::FakesSubmenu);
            app.fakes_menu = FakesMenuState::DiscordUdp;
        }
        ExtendedMenuState::Back => {
            app.back();
        }
    }
}
