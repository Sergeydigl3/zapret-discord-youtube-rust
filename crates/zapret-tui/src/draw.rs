//! One frame of the UI.
//!
//! Pure rendering: it reads [`AppState`] and paints. Every screen contributes a
//! title, a list of items and a help line, and the three status lines are
//! shared by the screens that show them.

use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, Paragraph};
use ratatui::Frame;

use crate::menus;
use crate::state::{
    ActiveScreen, AppState, AutotuneBlockChecksState, AutotuneMenuState, AutotuneProtocolsState, DownloadDepsMenuState,
    DownloadSubmenuState, FakesMenuState, GamefilterMenuState, MainMenuState,
};
use crate::theme::Theme;

/// Screens that render the service and dependency status lines under the list.
fn shows_status_lines(screen: ActiveScreen) -> bool {
    matches!(
        screen,
        ActiveScreen::Main
            | ActiveScreen::ServiceSubmenu
            | ActiveScreen::DownloadDepsSubmenu
            | ActiveScreen::DownloadZapretSubmenu
            | ActiveScreen::DownloadStrategiesSubmenu
            | ActiveScreen::ZapretTagSelect
            | ActiveScreen::StrategyTagSelect
    )
}

fn title_text(screen: ActiveScreen) -> String {
    match screen {
        ActiveScreen::Main => rust_i18n::t!("tui_title_main").to_string(),
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => rust_i18n::t!("tui_title_defender").to_string(),
        ActiveScreen::StrategySubmenu => rust_i18n::t!("tui_title_strategy").to_string(),
        ActiveScreen::DownloadDepsSubmenu => rust_i18n::t!("tui_title_download_cat").to_string(),
        ActiveScreen::DownloadZapretSubmenu => rust_i18n::t!("tui_title_download_zapret").to_string(),
        ActiveScreen::DownloadStrategiesSubmenu => rust_i18n::t!("tui_title_download_strat").to_string(),
        ActiveScreen::GamefilterSubmenu => rust_i18n::t!("tui_title_gamefilter").to_string(),
        ActiveScreen::FakesSubmenu => rust_i18n::t!("tui_title_fakes").to_string(),
        ActiveScreen::FakesSelectSubmenu => rust_i18n::t!("menu_fakes_select_title").to_string(),
        ActiveScreen::ZapretTagSelect => rust_i18n::t!("tui_title_tag_zapret").to_string(),
        ActiveScreen::StrategyTagSelect => rust_i18n::t!("tui_title_tag_strat").to_string(),
        ActiveScreen::ServiceSubmenu => rust_i18n::t!("tui_title_service").to_string(),
        ActiveScreen::ListsEditorSubmenu => rust_i18n::t!("tui_title_lists").to_string(),
        ActiveScreen::AutotuneSubmenu => rust_i18n::t!("tui_title_autotune").to_string(),
        ActiveScreen::AutotuneEditDomainsSubmenu => rust_i18n::t!("tui_title_autotune_edit_domains").to_string(),
        ActiveScreen::AutotuneProtocolsSubmenu => rust_i18n::t!("tui_title_autotune_proto").to_string(),
        ActiveScreen::AutotuneBlockChecksSubmenu => rust_i18n::t!("tui_title_autotune_bc").to_string(),
        ActiveScreen::AutotunePresetSelectionSubmenu => rust_i18n::t!("tui_title_autotune_presets").to_string(),
        ActiveScreen::AutotuneStrategiesSubmenu => rust_i18n::t!("tui_title_autotune_strat").to_string(),
        ActiveScreen::AutotuneResultsSubmenu => rust_i18n::t!("tui_title_autotune_results").to_string(),
    }
}

fn menu_items<'a>(app: &'a AppState) -> (Vec<ratatui::widgets::ListItem<'a>>, String, usize) {
    match app.active_screen {
        ActiveScreen::Main => menus::main_menu::render(app),
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => menus::defender_menu::render(app),
        ActiveScreen::StrategySubmenu => menus::strategy_menu::render(app),
        ActiveScreen::DownloadDepsSubmenu => menus::download_menu::render(app),
        ActiveScreen::DownloadZapretSubmenu => menus::download_submenu::render(app, true),
        ActiveScreen::DownloadStrategiesSubmenu => menus::download_submenu::render(app, false),
        ActiveScreen::GamefilterSubmenu => menus::gamefilter_menu::render(app),
        ActiveScreen::FakesSubmenu => menus::fakes_menu::render(app),
        ActiveScreen::FakesSelectSubmenu => {
            menus::fakes_menu::render_select(&app.fakes_state, &app.fakes_select_for, app.fakes_select_index)
        }
        ActiveScreen::ZapretTagSelect => menus::tag_menu::render(
            &app.available_nfqws_tags,
            app.nfqws_tag_index,
            &rust_i18n::t!("menu_tag_title_zapret"),
        ),
        ActiveScreen::StrategyTagSelect => menus::tag_menu::render(
            &app.available_strat_tags,
            app.strat_tag_index,
            &rust_i18n::t!("menu_tag_title_strat"),
        ),
        ActiveScreen::ServiceSubmenu => menus::service_menu::render(app),
        ActiveScreen::ListsEditorSubmenu => menus::lists_menu::render(&app.lists_files, app.lists_menu_index),
        ActiveScreen::AutotuneSubmenu => {
            if app.autotune_running {
                menus::autotune_menu::render_header()
            } else {
                menus::autotune_menu::render_config(app)
            }
        }
        ActiveScreen::AutotuneEditDomainsSubmenu => menus::autotune_menu::render_domain_files(app),
        ActiveScreen::AutotuneProtocolsSubmenu => {
            menus::autotune_menu::render_protocols(app, app.autotune_protocols_menu)
        }
        ActiveScreen::AutotuneBlockChecksSubmenu => {
            menus::autotune_menu::render_blockchecks(app, app.autotune_block_checks_menu)
        }
        ActiveScreen::AutotunePresetSelectionSubmenu => menus::autotune_menu::render_presets(app),
        ActiveScreen::AutotuneStrategiesSubmenu => {
            menus::autotune_menu::render_strategies(app, app.autotune_strat_index)
        }
        ActiveScreen::AutotuneResultsSubmenu => menus::autotune_menu::render_results(app, app.autotune_results_index),
    }
}

fn help_text(app: &AppState) -> String {
    match app.active_screen {
        ActiveScreen::Main => match app.main_menu {
            #[cfg(target_os = "windows")]
            MainMenuState::DefenderSettings => rust_i18n::t!("help_def").to_string(),
            MainMenuState::DownloadDeps => rust_i18n::t!("help_dl").to_string(),
            MainMenuState::Interface => rust_i18n::t!("help_iface").to_string(),
            MainMenuState::IpsetMode => rust_i18n::t!("help_ipset").to_string(),
            MainMenuState::Strategy => rust_i18n::t!("help_strat").to_string(),
            MainMenuState::GamefilterSettings => rust_i18n::t!("help_gf").to_string(),
            #[cfg(target_os = "linux")]
            MainMenuState::BackendSettings => rust_i18n::t!("help_backend").to_string(),
            MainMenuState::ServiceSettings => rust_i18n::t!("help_srv").to_string(),
            MainMenuState::ListsEditor => rust_i18n::t!("help_lists").to_string(),
            MainMenuState::Autotune => rust_i18n::t!("help_autotune").to_string(),
            MainMenuState::TtlAutopick => rust_i18n::t!("help_ttl").to_string(),
            MainMenuState::FakesSettings => rust_i18n::t!("help_fakes").to_string(),
            MainMenuState::Run => rust_i18n::t!("help_run").to_string(),
            MainMenuState::Quit => rust_i18n::t!("help_quit").to_string(),
        },
        ActiveScreen::DownloadDepsSubmenu => match app.download_deps_menu {
            DownloadDepsMenuState::ZapretDownloader => rust_i18n::t!("help_dl_zap").to_string(),
            DownloadDepsMenuState::StrategiesDownloader => rust_i18n::t!("help_dl_str").to_string(),
            DownloadDepsMenuState::DownloadDefaults => rust_i18n::t!("help_dl_def").to_string(),
            DownloadDepsMenuState::Back => rust_i18n::t!("help_back").to_string(),
        },
        ActiveScreen::DownloadZapretSubmenu => match app.download_zapret_menu {
            DownloadSubmenuState::Version => rust_i18n::t!("help_dl_ver").to_string(),
            DownloadSubmenuState::SelectTag => rust_i18n::t!("help_dl_tag").to_string(),
            DownloadSubmenuState::Start => rust_i18n::t!("help_dl_start").to_string(),
            DownloadSubmenuState::Back => rust_i18n::t!("help_back").to_string(),
        },
        ActiveScreen::DownloadStrategiesSubmenu => match app.download_strategies_menu {
            DownloadSubmenuState::Version => rust_i18n::t!("help_dl_ver").to_string(),
            DownloadSubmenuState::SelectTag => rust_i18n::t!("help_dl_tag").to_string(),
            DownloadSubmenuState::Start => rust_i18n::t!("help_dl_start").to_string(),
            DownloadSubmenuState::Back => rust_i18n::t!("help_back").to_string(),
        },
        ActiveScreen::GamefilterSubmenu => match app.gamefilter_menu {
            GamefilterMenuState::Tcp => rust_i18n::t!("help_gf_tcp").to_string(),
            GamefilterMenuState::Udp => rust_i18n::t!("help_gf_udp").to_string(),
            GamefilterMenuState::Back => rust_i18n::t!("help_back").to_string(),
        },
        ActiveScreen::FakesSubmenu => match app.fakes_menu {
            FakesMenuState::DiscordUdp | FakesMenuState::GameUdp => rust_i18n::t!("help_fakes_sel").to_string(),
            FakesMenuState::Back => rust_i18n::t!("help_back").to_string(),
        },
        ActiveScreen::FakesSelectSubmenu => rust_i18n::t!("help_fakes_select").to_string(),
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => rust_i18n::t!("help_def_sel").to_string(),
        ActiveScreen::StrategySubmenu => rust_i18n::t!("help_strat_sel").to_string(),
        ActiveScreen::ZapretTagSelect => rust_i18n::t!("help_tag_sel").to_string(),
        ActiveScreen::StrategyTagSelect => rust_i18n::t!("help_tag_sel").to_string(),
        ActiveScreen::ServiceSubmenu => rust_i18n::t!("help_srv_sel").to_string(),
        ActiveScreen::ListsEditorSubmenu => rust_i18n::t!("help_lists").to_string(),
        ActiveScreen::AutotuneSubmenu => match app.autotune_menu {
            AutotuneMenuState::PresetSelection => rust_i18n::t!("help_autotune_domains").to_string(),
            AutotuneMenuState::NumRequests => rust_i18n::t!("help_autotune_req").to_string(),
            AutotuneMenuState::Strategies => rust_i18n::t!("help_autotune_strat_sel").to_string(),
            AutotuneMenuState::Protocols => rust_i18n::t!("help_autotune_proto").to_string(),
            AutotuneMenuState::BlockChecks => rust_i18n::t!("help_autotune_blockchecks").to_string(),
            AutotuneMenuState::EditDomains => rust_i18n::t!("help_autotune_edit_domains").to_string(),
            AutotuneMenuState::Results => rust_i18n::t!("help_autotune_results_sel").to_string(),
            AutotuneMenuState::Run => rust_i18n::t!("help_autotune_run").to_string(),
            AutotuneMenuState::Back => rust_i18n::t!("help_back").to_string(),
        },
        ActiveScreen::AutotuneProtocolsSubmenu => match app.autotune_protocols_menu {
            AutotuneProtocolsState::Back => rust_i18n::t!("help_back").to_string(),
            _ => rust_i18n::t!("help_autotune_toggle").to_string(),
        },
        ActiveScreen::AutotuneBlockChecksSubmenu => match app.autotune_block_checks_menu {
            AutotuneBlockChecksState::Back => rust_i18n::t!("help_back").to_string(),
            _ => rust_i18n::t!("help_autotune_toggle").to_string(),
        },
        ActiveScreen::AutotuneEditDomainsSubmenu => rust_i18n::t!("help_autotune_edit_domains").to_string(),
        ActiveScreen::AutotunePresetSelectionSubmenu => rust_i18n::t!("help_autotune_presets").to_string(),
        ActiveScreen::AutotuneStrategiesSubmenu => rust_i18n::t!("help_autotune_strat").to_string(),
        ActiveScreen::AutotuneResultsSubmenu => rust_i18n::t!("help_autotune_results").to_string(),
    }
}

fn service_type_str() -> String {
    #[cfg(target_os = "windows")]
    {
        rust_i18n::t!("status_srv_win").to_string()
    }
    #[cfg(target_os = "linux")]
    {
        zapret_core::service::detect_init_system()
            .map(|t| t.as_str().to_string())
            .unwrap_or_else(|| rust_i18n::t!("status_srv_unknown").into_owned())
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        rust_i18n::t!("status_srv_unknown").into_owned()
    }
}

pub fn draw(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(3)
        .constraints([Constraint::Length(3), Constraint::Min(9), Constraint::Length(3)].as_ref())
        .split(f.size());

    let title_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    let title = Paragraph::new(Line::from(vec![Span::styled(
        title_text(app.active_screen),
        Theme::header_style(),
    )]))
    .alignment(Alignment::Center)
    .block(title_block);

    f.render_widget(title, chunks[0]);

    let (items, block_title, selected_index) = menu_items(app);

    let list_block = Block::default()
        .title(Span::styled(block_title, Theme::block_title()))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::dim_item());

    if shows_status_lines(app.active_screen) {
        let inner_area = list_block.inner(chunks[1]);
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
            .split(inner_area);

        f.render_widget(list_block, chunks[1]);

        let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::ITALIC));
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(selected_index));
        f.render_stateful_widget(list, main_chunks[0], &mut list_state);

        // Service Status line
        let (status_icon, status_color, status_desc) = if !app.service_installed {
            ("❌", Color::Red, rust_i18n::t!("status_srv_not_inst").to_string())
        } else if app.service_active {
            ("✅", Color::Green, rust_i18n::t!("status_srv_active").to_string())
        } else {
            ("🟡", Color::Yellow, rust_i18n::t!("status_srv_stopped").to_string())
        };

        let service_status_text = Line::from(vec![
            Span::styled(rust_i18n::t!("status_srv_title"), Style::default().fg(Color::Gray)),
            Span::styled(service_type_str(), Style::default().fg(Color::White)),
            Span::styled("): ", Style::default().fg(Color::Gray)),
            Span::styled(
                status_desc,
                Style::default().fg(status_color).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::raw(status_icon),
        ]);
        let service_status_paragraph = Paragraph::new(service_status_text).alignment(Alignment::Center);
        f.render_widget(service_status_paragraph, main_chunks[1]);

        // Dependencies status line
        let nfqws_status = if app.nfqws_installed { "✅" } else { "❌" };
        let strat_status = if app.strategies_installed { "✅" } else { "❌" };
        let status_text = Line::from(vec![
            Span::styled(rust_i18n::t!("status_deps_title"), Style::default().fg(Color::Gray)),
            Span::styled("nfqws ", Style::default().fg(Color::White)),
            Span::raw(nfqws_status),
            Span::styled(
                format!(" | {} ", rust_i18n::t!("status_deps_strat")),
                Style::default().fg(Color::White),
            ),
            Span::raw(strat_status),
        ]);
        let status_paragraph = Paragraph::new(status_text).alignment(Alignment::Center);

        f.render_widget(status_paragraph, main_chunks[2]);
    } else if app.active_screen == ActiveScreen::AutotuneSubmenu && !app.autotune_running {
        f.render_widget(&list_block, chunks[1]);
        let inner = list_block.inner(chunks[1]);
        let sub = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(inner);
        let warning = Paragraph::new(Span::styled(
            rust_i18n::t!("autotune_warning_disable"),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center);
        f.render_widget(warning, sub[0]);
        let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::ITALIC));
        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(selected_index));
        f.render_stateful_widget(list, sub[1], &mut list_state);
    } else {
        let list = List::new(items)
            .block(list_block)
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC));

        let mut list_state = ratatui::widgets::ListState::default();
        list_state.select(Some(selected_index));

        f.render_stateful_widget(list, chunks[1], &mut list_state);
    }

    let dynamic_help = help_text(app);

    let help_text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        dynamic_help
    };

    let help_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Theme::dim_item());

    let help = Paragraph::new(Span::styled(
        help_text,
        Style::default().fg(if app.status_message.is_some() {
            Color::Cyan
        } else {
            Color::Gray
        }),
    ))
    .alignment(Alignment::Center)
    .block(help_block);

    f.render_widget(help, chunks[2]);
}
