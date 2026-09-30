//! The game-filter toggles.

use crate::state::screens::{ActiveScreen, GamefilterMenuState};
use crate::state::AppState;

use super::on_activate;

pub fn activate(app: &mut AppState) {
    match app.gamefilter_menu {
        GamefilterMenuState::Tcp => {
            app.tcp_gamefilter = !app.tcp_gamefilter;
            app.save_current_config();
        }
        GamefilterMenuState::Udp => {
            app.udp_gamefilter = !app.udp_gamefilter;
            app.save_current_config();
        }
        GamefilterMenuState::Back => {
            app.active_screen = ActiveScreen::Main;
            app.status_message = None;
        }
    }
}

pub fn cycle(app: &mut AppState, forward: bool) {
    match app.gamefilter_menu {
        GamefilterMenuState::Tcp => {
            app.tcp_gamefilter = !app.tcp_gamefilter;
            app.save_current_config();
        }
        GamefilterMenuState::Udp => {
            app.udp_gamefilter = !app.udp_gamefilter;
            app.save_current_config();
        }
        _ => {
            if forward {
                on_activate(app);
            }
        }
    }
}
