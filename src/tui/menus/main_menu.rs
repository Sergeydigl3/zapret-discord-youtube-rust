use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
};
use crate::tui::state::{AppState, MainMenuState};
use crate::tui::theme::Theme;

pub fn render(app: &AppState) -> (Vec<ListItem<'static>>, &'static str, usize) {
    let mut selected_index = 0;
    let mut items = vec![];
    let mut index = 0;

    #[cfg(target_os = "windows")]
    {
        let is_sel = app.main_menu == MainMenuState::DefenderSettings;
        if is_sel { selected_index = index; }
        items.push(ListItem::new(" 🛡️ Windows Defender Settings").style(
            if is_sel { Theme::selected_item() } else { Theme::normal_item() }
        ));
        index += 1;
    }
    
    // DownloadDeps
    {
        let is_sel = app.main_menu == MainMenuState::DownloadDeps;
        if is_sel { selected_index = index; }
        let nfqws_status = if app.nfqws_installed { "✅" } else { "❌" };
        let strat_status = if app.strategies_installed { "✅" } else { "❌" };
        items.push(ListItem::new(format!(" 📥 Dependencies: nfqws {} | strategies {} ", nfqws_status, strat_status)).style(
            if is_sel { Theme::selected_item() } else { Theme::normal_item() }
        ));
        index += 1;
    }

    // Interface
    {
        let is_sel = app.main_menu == MainMenuState::Interface;
        if is_sel { selected_index = index; }
        let val = app.interfaces.get(app.selected_interface).unwrap_or(&"None".to_string()).clone();
        let spans = vec![
            Span::styled(" 🌐 Network Interface:   ", if is_sel { Theme::selected_item() } else { Theme::normal_item() }),
            Span::styled(format!(" < {} >", val), if is_sel { Theme::selected_value() } else { Theme::normal_value() }),
        ];
        items.push(ListItem::new(Line::from(spans)));
        index += 1;
    }

    // Strategy
    {
        let is_sel = app.main_menu == MainMenuState::Strategy;
        if is_sel { selected_index = index; }
        let val = app.strategies.get(app.selected_strategy).unwrap_or(&"None".to_string()).clone();
        let spans = vec![
            Span::styled(" 📜 Strategy:            ", if is_sel { Theme::selected_item() } else { Theme::normal_item() }),
            Span::styled(format!(" < {} >", val), if is_sel { Theme::selected_value() } else { Theme::normal_value() }),
        ];
        items.push(ListItem::new(Line::from(spans)));
        index += 1;
    }

    // GamefilterSettings
    {
        let is_sel = app.main_menu == MainMenuState::GamefilterSettings;
        if is_sel { selected_index = index; }
        
        let mut spans = vec![
            Span::styled(" 🎮 Game Filter Settings: ", if is_sel { Theme::selected_item() } else { Theme::normal_item() }),
        ];

        let mut status_parts = vec![];
        if app.tcp_gamefilter {
            status_parts.push("TCP");
        }
        if app.udp_gamefilter {
            status_parts.push("UDP");
        }
        let status_str = if status_parts.is_empty() {
            "Disabled".to_string()
        } else {
            status_parts.join("+")
        };

        spans.push(Span::styled(
            format!(" < {} >", status_str),
            if is_sel { Theme::selected_value() } else { Theme::normal_value() }
        ));

        items.push(ListItem::new(Line::from(spans)));
        index += 1;
    }

    // Run
    {
        let is_sel = app.main_menu == MainMenuState::Run;
        if is_sel { selected_index = index; }
        items.push(ListItem::new(" ▶️  Run Zapret").style(
            if is_sel { Theme::selected_item() } else { Theme::normal_item() }
        ));
        index += 1;
    }

    // Quit
    {
        let is_sel = app.main_menu == MainMenuState::Quit;
        if is_sel { selected_index = index; }
        items.push(ListItem::new(" 🚪 Quit").style(
            if is_sel { Theme::selected_item() } else { Theme::normal_item() }
        ));
    }

    (items, " Configuration ", selected_index)
}
