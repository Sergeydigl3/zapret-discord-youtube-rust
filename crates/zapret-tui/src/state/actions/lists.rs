//! The list-file editor.

use crate::state::screens::ActiveScreen;
use crate::state::AppState;

pub fn activate(app: &mut AppState) {
    if app.lists_menu_index < app.lists_files.len() {
        let file = app.lists_files[app.lists_menu_index].clone();
        app.should_open_editor = Some(file);
    } else {
        app.active_screen = ActiveScreen::Main;
        app.status_message = None;
    }
}
