//! The three decisions about the fixed DPI-TTL.

use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use crate::state::{AppState, TtlMenuState};
use crate::theme::Theme;

/// A short label for the current setting: the hop count, or "off".
pub fn current_label(app: &AppState) -> String {
    match app.dpi_desync_ttl {
        Some(n) => n.to_string(),
        None => rust_i18n::t!("ttl_dont_touch_short").into_owned(),
    }
}

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, String, usize) {
    let value = current_label(app);

    let rows = [
        (
            TtlMenuState::DontTouch,
            rust_i18n::t!("menu_ttl_dont_touch").to_string(),
            value.clone(),
        ),
        (
            TtlMenuState::SetValue,
            rust_i18n::t!("menu_ttl_set_value").to_string(),
            value,
        ),
        (
            TtlMenuState::Autopick,
            rust_i18n::t!("menu_ttl_autopick").to_string(),
            rust_i18n::t!("menu_ttl_autopick_value").to_string(),
        ),
    ];

    let mut items: Vec<ListItem<'static>> = rows
        .iter()
        .map(|(state, label, val)| {
            let selected = app.ttl_menu == *state;
            // Only the row that actually takes a number shows the arrows, so a
            // value is never mistaken for something you can nudge here.
            let value_text = if *state == TtlMenuState::SetValue {
                format!("< {} >", val)
            } else {
                val.clone()
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {}: ", label),
                    if selected {
                        Theme::selected_item()
                    } else {
                        Theme::normal_item()
                    },
                ),
                Span::styled(
                    value_text,
                    if selected {
                        Theme::selected_value()
                    } else {
                        Theme::normal_value()
                    },
                ),
            ]))
        })
        .collect();

    let selected = app.ttl_menu.index();
    let back_selected = app.ttl_menu == TtlMenuState::Back;
    items.push(ListItem::new(Line::from(Span::styled(
        format!(" {}", rust_i18n::t!("menu_ttl_back")),
        if back_selected {
            Theme::selected_item()
        } else {
            Theme::normal_item()
        },
    ))));

    (items, rust_i18n::t!("tui_title_ttl").into_owned(), selected)
}
