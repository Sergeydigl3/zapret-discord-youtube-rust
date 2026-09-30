//! The strategy picker.

use crate::state::screens::ActiveScreen;
use crate::state::AppState;

pub fn activate(app: &mut AppState) {
    if app.strategy_menu_index < app.strategies.len() {
        app.selected_strategy = app.strategy_menu_index;
        app.save_current_config();
        app.active_screen = ActiveScreen::Main;
        app.status_message = Some(format!(
            "{}{}",
            rust_i18n::t!("msg_strat_sel"),
            app.strategies[app.selected_strategy]
        ));
    } else {
        app.active_screen = ActiveScreen::Main;
        app.status_message = None;
    }
}
