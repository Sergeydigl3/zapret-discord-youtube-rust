use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};
use crate::tui::state::{AppState, GamefilterMenuState};
use crate::tui::theme::Theme;

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, String, usize) {
    let mut selected_index = 0;
    let mut items = vec![];
    let mut index = 0;

    let on_label = rust_i18n::t!("val_on");
    let off_label = rust_i18n::t!("val_off");

    // TCP Gamefilter option
    {
        let is_sel = app.gamefilter_menu == GamefilterMenuState::Tcp;
        if is_sel {
            selected_index = index;
        }

        let label_style = if is_sel { Theme::selected_item() } else { Theme::normal_item() };

        let mut spans = vec![
            Span::styled(format!(" {}      ", rust_i18n::t!("menu_gf_tcp")), label_style),
        ];

        if app.tcp_gamefilter {
            spans.push(Span::styled(format!(" [● {}] ", on_label), if is_sel { Theme::selected_value() } else { Theme::active_value() }));
            spans.push(Span::styled(format!(" [ {} ] ", off_label), if is_sel { Theme::dim_item().patch(Theme::selected_item()) } else { Theme::dim_item() }));
        } else {
            spans.push(Span::styled(format!(" [ {} ] ", on_label), if is_sel { Theme::dim_item().patch(Theme::selected_item()) } else { Theme::dim_item() }));
            spans.push(Span::styled(format!(" [● {}] ", off_label), if is_sel { Theme::selected_value() } else { Theme::inactive_value() }));
        }

        items.push(ListItem::new(Line::from(spans)));
        index += 1;
    }

    // UDP Gamefilter option
    {
        let is_sel = app.gamefilter_menu == GamefilterMenuState::Udp;
        if is_sel {
            selected_index = index;
        }

        let label_style = if is_sel { Theme::selected_item() } else { Theme::normal_item() };

        let mut spans = vec![
            Span::styled(format!(" {}      ", rust_i18n::t!("menu_gf_udp")), label_style),
        ];

        if app.udp_gamefilter {
            spans.push(Span::styled(format!(" [● {}] ", on_label), if is_sel { Theme::selected_value() } else { Theme::active_value() }));
            spans.push(Span::styled(format!(" [ {} ] ", off_label), if is_sel { Theme::dim_item().patch(Theme::selected_item()) } else { Theme::dim_item() }));
        } else {
            spans.push(Span::styled(format!(" [ {} ] ", on_label), if is_sel { Theme::dim_item().patch(Theme::selected_item()) } else { Theme::dim_item() }));
            spans.push(Span::styled(format!(" [● {}] ", off_label), if is_sel { Theme::selected_value() } else { Theme::inactive_value() }));
        }

        items.push(ListItem::new(Line::from(spans)));
        index += 1;
    }

    // Back option
    {
        let is_sel = app.gamefilter_menu == GamefilterMenuState::Back;
        if is_sel {
            selected_index = index;
        }

        let style = if is_sel { Theme::selected_item() } else { Theme::normal_item() };
        items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_gf_back"))).style(style));
    }

    (items, rust_i18n::t!("menu_gf_title").into_owned(), selected_index)
}
