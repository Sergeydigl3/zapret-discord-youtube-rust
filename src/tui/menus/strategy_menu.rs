use ratatui::widgets::ListItem;
use crate::tui::state::AppState;
use crate::tui::theme::Theme;

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, &'static str, usize) {
    let mut selected_index = 0;
    let mut items: Vec<ListItem> = app.strategies
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let prefix = if i == app.selected_strategy { "\u{F00C} " } else { "   " };
            let m = format!(" {}{}", prefix, s);
            if i == app.strategy_menu_index {
                selected_index = i;
                ListItem::new(m).style(Theme::selected_item())
            } else {
                ListItem::new(m).style(Theme::normal_item())
            }
        })
        .collect();
    
    let back_selected = app.strategy_menu_index == app.strategies.len();
    if back_selected {
        selected_index = app.strategies.len();
    }
    
    let back_item = if back_selected {
        ListItem::new(" \u{F04A} Back to Main Menu").style(Theme::selected_item())
    } else {
        ListItem::new(" \u{F04A} Back to Main Menu").style(Theme::normal_item())
    };
    
    items.push(back_item);
    (items, " Select Strategy ", selected_index)
}
