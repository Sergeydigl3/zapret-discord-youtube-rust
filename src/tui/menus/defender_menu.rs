use ratatui::widgets::ListItem;
use crate::tui::state::{AppState, DefenderMenuState};
use crate::tui::theme::Theme;

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, String, usize) {
    let mut selected_index = 0;
    
    let status_str = match app.defender_status_cache {
        Some(true) => rust_i18n::t!("status_def_active"),
        Some(false) => rust_i18n::t!("status_def_inactive"),
        None => rust_i18n::t!("status_def_unknown"),
    };

    let menu_items = vec![
        format!("{}{}", rust_i18n::t!("status_def_curr"), status_str),
        " ------------------------------------".to_string(),
        format!(" {}", rust_i18n::t!("menu_def_add")),
        format!(" {}", rust_i18n::t!("menu_def_remove")),
        format!(" {}", rust_i18n::t!("menu_def_back")),
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

    (items, rust_i18n::t!("menu_def_title").into_owned(), selected_index)
}
