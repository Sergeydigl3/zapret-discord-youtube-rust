use crate::menus::{Menu, Row};
use crate::state::AppState;

pub fn render(app: &AppState) -> Menu {
    let mut rows: Vec<Row> = app
        .strategies
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let row = Row::new(name.clone());
            if i == app.selected_strategy {
                row.mark()
            } else {
                row
            }
        })
        .collect();
    rows.push(Row::new(rust_i18n::t!("menu_subdl_back")));

    // The bound is `len + 1` rather than `len`, because the last row is the way
    // out and the cursor has to be able to sit on it.
    let index = app.strategy_menu_index.min(app.strategies.len());

    Menu::new(rust_i18n::t!("tui_title_strategy"), rows).at(index)
}
