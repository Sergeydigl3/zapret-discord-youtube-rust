use ratatui::widgets::ListItem;
use crate::tui::state::AppState;
use crate::tui::theme::Theme;

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, String, usize) {
    let mut menu_items = vec![];
    
    if !app.service_installed {
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_install")));
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_back")));
    } else if app.service_active {
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_stop")));
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_restart")));
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_uninstall")));
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_back")));
    } else {
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_start")));
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_uninstall")));
        menu_items.push(format!(" {}", rust_i18n::t!("menu_srv_back")));
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

    (items, rust_i18n::t!("menu_srv_title").into_owned(), selected_index)
}
