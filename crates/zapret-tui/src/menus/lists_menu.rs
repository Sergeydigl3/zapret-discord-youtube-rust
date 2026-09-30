use crate::menus::{Menu, Row};

pub fn render(lists_files: &[String], selected_index: usize) -> Menu {
    let mut rows: Vec<Row> = lists_files
        .iter()
        .map(|file| {
            Row::new(
                std::path::Path::new(file)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy(),
            )
        })
        .collect();
    rows.push(Row::new(rust_i18n::t!("menu_dl_back")));

    Menu::new(rust_i18n::t!("tui_title_lists"), rows).at(selected_index)
}
