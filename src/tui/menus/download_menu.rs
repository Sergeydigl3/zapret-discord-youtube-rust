use ratatui::widgets::ListItem;
use crate::tui::state::{AppState, DownloadDepsMenuState};
use crate::tui::theme::Theme;

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, String, usize) {
    let mut selected_index = 0;
    let menu_items = vec![
        format!(" {}", rust_i18n::t!("menu_dl_zapret")),
        format!(" {}", rust_i18n::t!("menu_dl_strat")),
        format!(" {}", rust_i18n::t!("menu_dl_defaults")),
        format!(" {}", rust_i18n::t!("menu_dl_back")),
    ];
    
    let items: Vec<ListItem> = menu_items
        .into_iter()
        .enumerate()
        .map(|(i, m)| {
            let is_selected = match app.download_deps_menu {
                DownloadDepsMenuState::ZapretDownloader if i == 0 => true,
                DownloadDepsMenuState::StrategiesDownloader if i == 1 => true,
                DownloadDepsMenuState::DownloadDefaults if i == 2 => true,
                DownloadDepsMenuState::Back if i == 3 => true,
                _ => false,
            };
            
            if is_selected {
                selected_index = i;
                ListItem::new(m).style(Theme::selected_item())
            } else {
                ListItem::new(m).style(Theme::normal_item())
            }
        })
        .collect();

    (items, rust_i18n::t!("menu_dl_title").into_owned(), selected_index)
}
