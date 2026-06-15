use ratatui::widgets::ListItem;
use crate::tui::theme::Theme;

pub fn render(
    tags: &[String],
    selected_tag_index: usize,
    title: &str,
) -> (Vec<ListItem<'static>>, String, usize) {
    let mut selected_index = 0;
    let mut items: Vec<ListItem> = tags
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let prefix = if i == selected_tag_index { "➡️ " } else { "   " };
            let m = format!(" {}{}", prefix, t);
            if i == selected_tag_index {
                selected_index = i;
                ListItem::new(m).style(Theme::selected_item())
            } else {
                ListItem::new(m).style(Theme::normal_item())
            }
        })
        .collect();
    
    let back_selected = selected_tag_index == tags.len();
    if back_selected {
        selected_index = tags.len();
    }
    
    let back_item = if back_selected {
        ListItem::new(format!(" {}", rust_i18n::t!("menu_subdl_back"))).style(Theme::selected_item())
    } else {
        ListItem::new(format!(" {}", rust_i18n::t!("menu_subdl_back"))).style(Theme::normal_item())
    };
    
    items.push(back_item);
    (items, title.to_string(), selected_index)
}
