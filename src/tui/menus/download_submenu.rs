use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};
use crate::tui::state::{AppState, DownloadSubmenuState, VersionTarget};
use crate::tui::theme::Theme;

pub fn render(
    app: &AppState,
    is_zapret: bool,
) -> (Vec<ListItem<'static>>, String, usize) {
    let mut selected_index = 0;
    
    let menu_state = if is_zapret { app.download_zapret_menu } else { app.download_strategies_menu };
    let target_ver = if is_zapret { &app.nfqws_target } else { &app.strat_target };

    let is_version_selected = menu_state == DownloadSubmenuState::Version;
    let label_style = if is_version_selected {
        selected_index = 0;
        Theme::selected_item()
    } else {
        Theme::normal_item()
    };

    let version_title = if is_zapret {
        rust_i18n::t!("menu_subdl_title_zapret")
    } else {
        rust_i18n::t!("menu_subdl_title_strat")
    };

    let mut version_spans = vec![
        Span::styled(version_title.into_owned(), label_style),
    ];

    let rec_ver_str = if is_zapret {
        crate::download::ZAPRET_REC_VER.to_string()
    } else {
        crate::download::STRAT_REC_VER[..7].to_string()
    };

    let latest_label = rust_i18n::t!("val_latest");

    let options = vec![
        (VersionTarget::Recommended, format!("{} ({})", rust_i18n::t!("val_rec"), rec_ver_str)),
        (VersionTarget::Latest, latest_label.into_owned()),
        (VersionTarget::Tag("".to_string()), match target_ver {
            VersionTarget::Tag(t) => format!("Tag ({})", t),
            _ => "Tag".to_string(),
        }),
    ];

    for (opt_ver, label) in options {
        let is_current = match (target_ver, &opt_ver) {
            (VersionTarget::Recommended, VersionTarget::Recommended) => true,
            (VersionTarget::Latest, VersionTarget::Latest) => true,
            (VersionTarget::Tag(_), VersionTarget::Tag(_)) => true,
            _ => false,
        };

        if is_current {
            version_spans.push(Span::styled(
                format!(" [● {}] ", label),
                if is_version_selected {
                    Theme::selected_value()
                } else {
                    Theme::normal_value()
                }
            ));
        } else {
            version_spans.push(Span::styled(
                format!(" [ {} ] ", label),
                if is_version_selected {
                    Theme::dim_item().patch(Theme::selected_item())
                } else {
                    Theme::dim_item()
                }
            ));
        }
    }

    let mut items = vec![
        ListItem::new(Line::from(version_spans)),
    ];

    let other_items = vec![
        (DownloadSubmenuState::SelectTag, format!("   {}", rust_i18n::t!("menu_subdl_tag"))),
        (DownloadSubmenuState::Start, format!(" {}", rust_i18n::t!("menu_subdl_start"))),
        (DownloadSubmenuState::Back, format!(" {}", rust_i18n::t!("menu_subdl_back"))),
    ];

    for (state, m) in other_items {
        let is_selected = menu_state == state;
        let index = match state {
            DownloadSubmenuState::SelectTag => 1,
            DownloadSubmenuState::Start => 2,
            DownloadSubmenuState::Back => 3,
            _ => 0,
        };
        
        if is_selected {
            selected_index = index;
            items.push(ListItem::new(m).style(Theme::selected_item()));
        } else {
            items.push(ListItem::new(m).style(Theme::normal_item()));
        }
    }

    let block_title = if is_zapret { rust_i18n::t!("tui_title_download_zapret").into_owned() } else { rust_i18n::t!("tui_title_download_strat").into_owned() };
    (items, block_title, selected_index)
}
