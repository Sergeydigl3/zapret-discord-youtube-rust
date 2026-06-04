use ratatui::widgets::ListItem;
use crate::tui::state::{AppState, DefenderMenuState};
use crate::tui::theme::Theme;

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, &'static str, usize) {
    let mut selected_index = 0;
    
    let status_str = match app.defender_status_cache {
        Some(true) => "✅ Active (Whitelisted)",
        Some(false) => "❌ Inactive (Not Whitelisted)",
        None => "⚠️ Unknown / Error",
    };

    let menu_items = vec![
        format!(" Current Status: {}", status_str),
        " ------------------------------------".to_string(),
        " ➕ Add Current Folder to Exclusions".to_string(),
        " 🗑️ Remove Current Folder from Exclusions".to_string(),
        " 🔙 Back to Main Menu".to_string(),
    ];

    let items: Vec<ListItem> = menu_items
        .into_iter()
        .enumerate()
        .map(|(i, m)| {
            let is_selected = match app.defender_menu {
                DefenderMenuState::Add if i == 2 => true,
                DefenderMenuState::Remove if i == 3 => true,
                DefenderMenuState::Back if i == 4 => true,
                _ => false,
            };
            
            if is_selected {
                selected_index = i;
                ListItem::new(m).style(Theme::selected_item())
            } else {
                ListItem::new(m).style(if i < 2 { Theme::normal_value() } else { Theme::normal_item() })
            }
        })
        .collect();

    (items, " Defender Options ", selected_index)
}
