use ratatui::widgets::ListItem;
use crate::tui::state::AppState;
use crate::tui::theme::Theme;

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, &'static str, usize) {
    let mut menu_items = vec![];
    
    if !app.service_installed {
        menu_items.push(" ▶️ Start Service".to_string());
        menu_items.push(" 🔙 Back".to_string());
    } else if app.service_active {
        menu_items.push(" ⏹️ Stop Service".to_string());
        menu_items.push(" 🔄 Restart Service".to_string());
        menu_items.push(" 🗑️ Uninstall Service".to_string());
        menu_items.push(" 🔙 Back".to_string());
    } else {
        menu_items.push(" ▶️ Start Service".to_string());
        menu_items.push(" 🗑️ Uninstall Service".to_string());
        menu_items.push(" 🔙 Back".to_string());
    }

    let selected_index = app.service_menu_index;

    let items: Vec<ListItem> = menu_items
        .into_iter()
        .enumerate()
        .map(|(i, m)| {
            if i == selected_index {
                ListItem::new(m).style(Theme::selected_item())
            } else {
                ListItem::new(m).style(Theme::normal_item())
            }
        })
        .collect();

    (items, " Service Management ", selected_index)
}
