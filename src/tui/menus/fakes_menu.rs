use crate::fakes::FakesState;
use crate::tui::state::{AppState, FakesMenuState, FakesSelectTarget};
use crate::tui::theme::Theme;
use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, String, usize) {
    let mut selected_index = 0;
    let mut items = vec![];
    let mut index = 0;

    let none_label = rust_i18n::t!("menu_fakes_none").into_owned();

    // Discord UDP
    {
        let is_sel = app.fakes_menu == FakesMenuState::DiscordUdp;
        if is_sel {
            selected_index = index;
        }
        let label_style = if is_sel {
            Theme::selected_item()
        } else {
            Theme::normal_item()
        };
        let val_style = if is_sel {
            Theme::selected_value()
        } else {
            Theme::normal_value()
        };

        let current = app
            .fakes_state
            .discord_active
            .as_deref()
            .unwrap_or(&none_label);
        let spans = vec![
            Span::styled(
                format!(" {}: ", rust_i18n::t!("menu_fakes_discord")),
                label_style,
            ),
            Span::styled(format!("< {} >", current), val_style),
        ];
        items.push(ListItem::new(Line::from(spans)));
        index += 1;
    }

    // GameFilter UDP
    {
        let is_sel = app.fakes_menu == FakesMenuState::GameUdp;
        if is_sel {
            selected_index = index;
        }
        let label_style = if is_sel {
            Theme::selected_item()
        } else {
            Theme::normal_item()
        };
        let val_style = if is_sel {
            Theme::selected_value()
        } else {
            Theme::normal_value()
        };

        let current = app
            .fakes_state
            .game_active
            .as_deref()
            .unwrap_or(&none_label);
        let spans = vec![
            Span::styled(
                format!(" {}: ", rust_i18n::t!("menu_fakes_game")),
                label_style,
            ),
            Span::styled(format!("< {} >", current), val_style),
        ];
        items.push(ListItem::new(Line::from(spans)));
        index += 1;
    }

    // Back
    {
        let is_sel = app.fakes_menu == FakesMenuState::Back;
        if is_sel {
            selected_index = index;
        }
        let style = if is_sel {
            Theme::selected_item()
        } else {
            Theme::normal_item()
        };
        items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_fakes_back"))).style(style));
    }

    (
        items,
        rust_i18n::t!("menu_fakes_title").into_owned(),
        selected_index,
    )
}

pub fn render_select(
    state: &FakesState,
    target: &FakesSelectTarget,
    selected_index: usize,
) -> (Vec<ListItem<'static>>, String, usize) {
    let mut items = vec![];
    let mut index = 0;

    let none_label = rust_i18n::t!("menu_fakes_none").into_owned();
    let current_label = match target {
        FakesSelectTarget::DiscordUdp => state.discord_active.as_deref().unwrap_or(&none_label),
        FakesSelectTarget::GameUdp => state.game_active.as_deref().unwrap_or(&none_label),
    };

    // Header: Current value (index 0, not selectable as a file)
    {
        let is_sel = index == selected_index;
        let current_spans = vec![
            Span::styled(
                format!(" {}: ", rust_i18n::t!("menu_fakes_current")),
                if is_sel {
                    Theme::selected_item()
                } else {
                    Theme::dim_item()
                },
            ),
            Span::styled(
                current_label.to_string(),
                if is_sel {
                    Theme::selected_value()
                } else {
                    Theme::normal_value()
                },
            ),
        ];
        items.push(ListItem::new(Line::from(current_spans)));
        index += 1;
    }

    // Available .bin files
    for fake in &state.available {
        let is_sel = index == selected_index;
        items.push(
            ListItem::new(format!("   {}", fake.filename)).style(if is_sel {
                Theme::selected_item()
            } else {
                Theme::normal_item()
            }),
        );
        index += 1;
    }

    // Back
    {
        let is_sel = index == selected_index;
        items.push(
            ListItem::new(format!(" {}", rust_i18n::t!("menu_fakes_back"))).style(if is_sel {
                Theme::selected_item()
            } else {
                Theme::normal_item()
            }),
        );
    }

    let title = rust_i18n::t!("menu_fakes_select_title").into_owned();
    (items, title, selected_index)
}
