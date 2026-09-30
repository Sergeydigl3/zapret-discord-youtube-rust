use crate::menus::{Menu, Row};

pub fn render(tags: &[String], selected_tag_index: usize, title: &str) -> Menu {
    let mut rows: Vec<Row> = tags.iter().map(Row::new).collect();
    rows.push(Row::new(rust_i18n::t!("menu_subdl_back")));

    Menu::new(title, rows).at(selected_tag_index.min(tags.len()))
}
