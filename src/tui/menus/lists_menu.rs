use ratatui::widgets::ListItem;
use crate::tui::theme::Theme;

pub fn render(lists_files: &[String], selected_index: usize) -> (Vec<ListItem<'static>>, String, usize) {
    let mut items = vec![];
    let mut index = 0;

    for file in lists_files {
        let is_sel = index == selected_index;
        
        let filename = std::path::Path::new(file)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        items.push(ListItem::new(format!(" {}", filename)).style(
            if is_sel { Theme::selected_item() } else { Theme::normal_item() }
        ));
        index += 1;
    }

    let is_sel = index == selected_index;
    items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_dl_back"))).style(
        if is_sel { Theme::selected_item() } else { Theme::normal_item() }
    ));

    (items, rust_i18n::t!("tui_title_lists").into_owned(), selected_index)
}
