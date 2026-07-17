use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, Paragraph},
    Terminal,
};
use std::io::{self, Write};

use crate::autotune::{CheckStatus, StrategyCheckResult};
use crate::tui::state::{
    ActiveScreen, AppState, VersionTarget, MainMenuState,
    DownloadDepsMenuState, DownloadSubmenuState,
    GamefilterMenuState, AutotuneMenuState, AutotuneProtocolsState,
    AutotuneBlockChecksState,
};
use crate::tui::menus;
use crate::tui::theme::Theme;

fn status_str(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pass => "✅",
        CheckStatus::Fail => "❌",
        CheckStatus::Skip => "⏭️",
        CheckStatus::Error => "⚠️",
    }
}

fn status_detail(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pass => "No blocking detected",
        CheckStatus::Fail => "Blocking detected",
        CheckStatus::Skip => "Skipped",
        CheckStatus::Error => "Error during check",
    }
}

pub fn run_tui(app: &mut AppState) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(3)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(9),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let title_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan));
                
            let title_text = match app.active_screen {
                ActiveScreen::Main => rust_i18n::t!("tui_title_main"),
                #[cfg(target_os = "windows")]
                ActiveScreen::DefenderSubmenu => rust_i18n::t!("tui_title_defender"),
                ActiveScreen::StrategySubmenu => rust_i18n::t!("tui_title_strategy"),
                ActiveScreen::DownloadDepsSubmenu => rust_i18n::t!("tui_title_download_cat"),
                ActiveScreen::DownloadZapretSubmenu => rust_i18n::t!("tui_title_download_zapret"),
                ActiveScreen::DownloadStrategiesSubmenu => rust_i18n::t!("tui_title_download_strat"),
                ActiveScreen::GamefilterSubmenu => rust_i18n::t!("tui_title_gamefilter"),
                ActiveScreen::ZapretTagSelect => rust_i18n::t!("tui_title_tag_zapret"),
                ActiveScreen::StrategyTagSelect => rust_i18n::t!("tui_title_tag_strat"),
                ActiveScreen::ServiceSubmenu => rust_i18n::t!("tui_title_service"),
                ActiveScreen::ListsEditorSubmenu => rust_i18n::t!("tui_title_lists"),
                ActiveScreen::AutotuneSubmenu => rust_i18n::t!("tui_title_autotune"),
                ActiveScreen::AutotuneProtocolsSubmenu => rust_i18n::t!("tui_title_autotune_proto"),
                ActiveScreen::AutotuneBlockChecksSubmenu => rust_i18n::t!("tui_title_autotune_bc"),
                ActiveScreen::AutotunePresetSelectionSubmenu => rust_i18n::t!("tui_title_autotune_presets"),
                ActiveScreen::AutotuneStrategiesSubmenu => rust_i18n::t!("tui_title_autotune_strat"),
                ActiveScreen::AutotuneResultsSubmenu => rust_i18n::t!("tui_title_autotune_results"),
            };

            let title = Paragraph::new(Line::from(vec![
                Span::styled(
                    title_text,
                    Theme::header_style()
                ),
            ]))
            .alignment(ratatui::layout::Alignment::Center)
            .block(title_block);
            
            f.render_widget(title, chunks[0]);

            let (items, block_title, selected_index) = match app.active_screen {
                ActiveScreen::Main => menus::main_menu::render(app),
                #[cfg(target_os = "windows")]
                ActiveScreen::DefenderSubmenu => menus::defender_menu::render(app),
                ActiveScreen::StrategySubmenu => menus::strategy_menu::render(app),
                ActiveScreen::DownloadDepsSubmenu => menus::download_menu::render(app),
                ActiveScreen::DownloadZapretSubmenu => menus::download_submenu::render(app, true),
                ActiveScreen::DownloadStrategiesSubmenu => menus::download_submenu::render(app, false),
                ActiveScreen::GamefilterSubmenu => menus::gamefilter_menu::render(app),
                ActiveScreen::ZapretTagSelect => menus::tag_menu::render(&app.available_nfqws_tags, app.nfqws_tag_index, &rust_i18n::t!("menu_tag_title_zapret")),
                ActiveScreen::StrategyTagSelect => menus::tag_menu::render(&app.available_strat_tags, app.strat_tag_index, &rust_i18n::t!("menu_tag_title_strat")),
                ActiveScreen::ServiceSubmenu => menus::service_menu::render(app),
                ActiveScreen::ListsEditorSubmenu => menus::lists_menu::render(&app.lists_files, app.lists_menu_index),
                ActiveScreen::AutotuneSubmenu => {
                    if app.autotune_running {
                        menus::autotune_menu::render_header()
                    } else {
                        menus::autotune_menu::render_config(app)
                    }
                }
                ActiveScreen::AutotuneProtocolsSubmenu => {
                    menus::autotune_menu::render_protocols(app, app.autotune_protocols_menu)
                }
                ActiveScreen::AutotuneBlockChecksSubmenu => {
                    menus::autotune_menu::render_blockchecks(app, app.autotune_block_checks_menu)
                }
                ActiveScreen::AutotunePresetSelectionSubmenu => {
                    menus::autotune_menu::render_presets(app)
                }
                ActiveScreen::AutotuneStrategiesSubmenu => {
                    menus::autotune_menu::render_strategies(app, app.autotune_strat_index)
                }
                ActiveScreen::AutotuneResultsSubmenu => {
                    menus::autotune_menu::render_results(app, app.autotune_results_index)
                }
            };

            let list_block = Block::default()
                .title(Span::styled(block_title, Theme::block_title()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Theme::dim_item());

            if app.active_screen == ActiveScreen::Main
                || app.active_screen == ActiveScreen::ServiceSubmenu
                || app.active_screen == ActiveScreen::DownloadDepsSubmenu
                || app.active_screen == ActiveScreen::DownloadZapretSubmenu
                || app.active_screen == ActiveScreen::DownloadStrategiesSubmenu
                || app.active_screen == ActiveScreen::ZapretTagSelect
                || app.active_screen == ActiveScreen::StrategyTagSelect
            {
                let inner_area = list_block.inner(chunks[1]);
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ])
                    .split(inner_area);

                f.render_widget(list_block, chunks[1]);

                let list = List::new(items)
                    .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::ITALIC));
                let mut list_state = ratatui::widgets::ListState::default();
                list_state.select(Some(selected_index));
                f.render_stateful_widget(list, main_chunks[0], &mut list_state);

                // Service Status line
                let (status_icon, status_color, status_desc) = if !app.service_installed {
                    ("❌", Color::Red, rust_i18n::t!("status_srv_not_inst"))
                } else if app.service_active {
                    ("✅", Color::Green, rust_i18n::t!("status_srv_active"))
                } else {
                    ("⏸️", Color::Yellow, rust_i18n::t!("status_srv_stopped"))
                };

                let service_type_str = {
                    #[cfg(target_os = "windows")]
                    { rust_i18n::t!("status_srv_win") }
                    #[cfg(target_os = "linux")]
                    {
                        crate::inits::detect_init_system()
                            .map(|t| t.as_str().to_string())
                            .unwrap_or_else(|| rust_i18n::t!("status_srv_unknown").into_owned())
                    }
                    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                    { rust_i18n::t!("status_srv_unknown").into_owned() }
                };

                let service_status_text = Line::from(vec![
                    Span::styled(rust_i18n::t!("status_srv_title"), Style::default().fg(Color::Gray)),
                    Span::styled(service_type_str, Style::default().fg(Color::White)),
                    Span::styled("): ", Style::default().fg(Color::Gray)),
                    Span::styled(status_desc, Style::default().fg(status_color).add_modifier(ratatui::style::Modifier::BOLD)),
                    Span::raw(" "),
                    Span::raw(status_icon),
                ]);
                let service_status_paragraph = Paragraph::new(service_status_text)
                    .alignment(ratatui::layout::Alignment::Center);
                f.render_widget(service_status_paragraph, main_chunks[1]);

                // Dependencies status line
                let nfqws_status = if app.nfqws_installed { "✅" } else { "❌" };
                let strat_status = if app.strategies_installed { "✅" } else { "❌" };
                let status_text = Line::from(vec![
                    Span::styled(rust_i18n::t!("status_deps_title"), Style::default().fg(Color::Gray)),
                    Span::styled("nfqws ", Style::default().fg(Color::White)),
                    Span::raw(nfqws_status),
                    Span::styled(format!(" | {} ", rust_i18n::t!("status_deps_strat")), Style::default().fg(Color::White)),
                    Span::raw(strat_status),
                ]);
                let status_paragraph = Paragraph::new(status_text)
                    .alignment(ratatui::layout::Alignment::Center);
                
                f.render_widget(status_paragraph, main_chunks[2]);
            } else if app.active_screen == ActiveScreen::AutotuneSubmenu && !app.autotune_running {
                f.render_widget(&list_block, chunks[1]);
                let inner = list_block.inner(chunks[1]);
                let sub = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Min(1),
                    ])
                    .split(inner);
                let warning = Paragraph::new(Span::styled(
                    rust_i18n::t!("autotune_warning_disable"),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center);
                f.render_widget(warning, sub[0]);
                let list = List::new(items)
                    .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::ITALIC));
                let mut list_state = ratatui::widgets::ListState::default();
                list_state.select(Some(selected_index));
                f.render_stateful_widget(list, sub[1], &mut list_state);
            } else {
                let list = List::new(items)
                    .block(list_block)
                    .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::ITALIC));
                
                let mut list_state = ratatui::widgets::ListState::default();
                list_state.select(Some(selected_index));
                
                f.render_stateful_widget(list, chunks[1], &mut list_state);
            }

            let dynamic_help = match app.active_screen {
                ActiveScreen::Main => match app.main_menu {
                    #[cfg(target_os = "windows")]
                    MainMenuState::DefenderSettings => rust_i18n::t!("help_def"),
                    MainMenuState::DownloadDeps => rust_i18n::t!("help_dl"),
                    MainMenuState::Interface => rust_i18n::t!("help_iface"),
                    MainMenuState::IpsetMode => rust_i18n::t!("help_ipset"),
                    MainMenuState::Strategy => rust_i18n::t!("help_strat"),
                    MainMenuState::GamefilterSettings => rust_i18n::t!("help_gf"),
                    #[cfg(target_os = "linux")]
                    MainMenuState::BackendSettings => rust_i18n::t!("help_backend"),
                    MainMenuState::ServiceSettings => rust_i18n::t!("help_srv"),
                    MainMenuState::ListsEditor => rust_i18n::t!("help_lists"),
                    MainMenuState::Autotune => rust_i18n::t!("help_autotune"),
                    MainMenuState::Run => rust_i18n::t!("help_run"),
                    MainMenuState::Quit => rust_i18n::t!("help_quit"),
                },
                ActiveScreen::DownloadDepsSubmenu => match app.download_deps_menu {
                    DownloadDepsMenuState::ZapretDownloader => rust_i18n::t!("help_dl_zap"),
                    DownloadDepsMenuState::StrategiesDownloader => rust_i18n::t!("help_dl_str"),
                    DownloadDepsMenuState::DownloadDefaults => rust_i18n::t!("help_dl_def"),
                    DownloadDepsMenuState::Back => rust_i18n::t!("help_back"),
                },
                ActiveScreen::DownloadZapretSubmenu => match app.download_zapret_menu {
                    DownloadSubmenuState::Version => rust_i18n::t!("help_dl_ver"),
                    DownloadSubmenuState::SelectTag => rust_i18n::t!("help_dl_tag"),
                    DownloadSubmenuState::Start => rust_i18n::t!("help_dl_start"),
                    DownloadSubmenuState::Back => rust_i18n::t!("help_back"),
                },
                ActiveScreen::DownloadStrategiesSubmenu => match app.download_strategies_menu {
                    DownloadSubmenuState::Version => rust_i18n::t!("help_dl_ver"),
                    DownloadSubmenuState::SelectTag => rust_i18n::t!("help_dl_tag"),
                    DownloadSubmenuState::Start => rust_i18n::t!("help_dl_start"),
                    DownloadSubmenuState::Back => rust_i18n::t!("help_back"),
                },
                ActiveScreen::GamefilterSubmenu => match app.gamefilter_menu {
                    GamefilterMenuState::Tcp => rust_i18n::t!("help_gf_tcp"),
                    GamefilterMenuState::Udp => rust_i18n::t!("help_gf_udp"),
                    GamefilterMenuState::Back => rust_i18n::t!("help_back"),
                },
                #[cfg(target_os = "windows")]
                ActiveScreen::DefenderSubmenu => rust_i18n::t!("help_def_sel"),
                ActiveScreen::StrategySubmenu => rust_i18n::t!("help_strat_sel"),
                ActiveScreen::ZapretTagSelect => rust_i18n::t!("help_tag_sel"),
                ActiveScreen::StrategyTagSelect => rust_i18n::t!("help_tag_sel"),
                ActiveScreen::ServiceSubmenu => rust_i18n::t!("help_srv_sel"),
                ActiveScreen::ListsEditorSubmenu => rust_i18n::t!("help_lists"),
                ActiveScreen::AutotuneSubmenu => match app.autotune_menu {
                    AutotuneMenuState::PresetSelection => rust_i18n::t!("help_autotune_domains"),
                    AutotuneMenuState::NumRequests => rust_i18n::t!("help_autotune_req"),
                    AutotuneMenuState::Strategies => rust_i18n::t!("help_autotune_strat_sel"),
                    AutotuneMenuState::Protocols => rust_i18n::t!("help_autotune_proto"),
                    AutotuneMenuState::BlockChecks => rust_i18n::t!("help_autotune_blockchecks"),
                    AutotuneMenuState::EditCustom => rust_i18n::t!("help_autotune_domains"),
                    AutotuneMenuState::Results => rust_i18n::t!("help_autotune_results_sel"),
                    AutotuneMenuState::Run => rust_i18n::t!("help_autotune_run"),
                    AutotuneMenuState::Back => rust_i18n::t!("help_back"),
                },
                ActiveScreen::AutotuneProtocolsSubmenu => match app.autotune_protocols_menu {
                    AutotuneProtocolsState::Back => rust_i18n::t!("help_back"),
                    _ => rust_i18n::t!("help_autotune_toggle"),
                },
                ActiveScreen::AutotuneBlockChecksSubmenu => match app.autotune_block_checks_menu {
                    AutotuneBlockChecksState::Back => rust_i18n::t!("help_back"),
                    _ => rust_i18n::t!("help_autotune_toggle"),
                },
                ActiveScreen::AutotunePresetSelectionSubmenu => {
                    rust_i18n::t!("help_autotune_presets")
                }
                ActiveScreen::AutotuneStrategiesSubmenu => {
                    rust_i18n::t!("help_autotune_strat")
                }
                ActiveScreen::AutotuneResultsSubmenu => {
                    rust_i18n::t!("help_autotune_results")
                }
            };

            let help_text = if let Some(ref msg) = app.status_message {
                msg.clone()
            } else {
                dynamic_help.to_string()
            };

            let help_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Theme::dim_item());
                
            let help = Paragraph::new(Span::styled(
                help_text,
                Style::default().fg(if app.status_message.is_some() { Color::Cyan } else { Color::Gray }),
            ))
            .alignment(ratatui::layout::Alignment::Center)
            .block(help_block);
            
            f.render_widget(help, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.autotune_request_editing {
                        match key.code {
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                app.autotune_request_buf.push(c);
                            }
                            KeyCode::Backspace => {
                                app.autotune_request_buf.pop();
                            }
                            KeyCode::Enter => {
                                if let Ok(n) = app.autotune_request_buf.parse::<usize>() {
                                    app.autotune_config.num_requests = n.max(1);
                                }
                                app.autotune_request_editing = false;
                                app.autotune_request_buf.clear();
                            }
                            KeyCode::Esc => {
                                app.autotune_request_editing = false;
                                app.autotune_request_buf.clear();
                            }
                            _ => {}
                        }
                        continue;
                    }
                    match key.code {
                        KeyCode::Up => app.prev_menu(),
                        KeyCode::Down => app.next_menu(),
                        KeyCode::Left => {
                            app.cycle_current(false);
                        }
                        KeyCode::Right => app.cycle_current(true),
                        KeyCode::Enter | KeyCode::Char(' ') => app.cycle_current(true),
                        KeyCode::Char('q') | KeyCode::Esc => {
                            match app.active_screen {
                                ActiveScreen::AutotuneSubmenu => {
                                    app.active_screen = ActiveScreen::Main;
                                }
                                ActiveScreen::AutotuneProtocolsSubmenu
                                | ActiveScreen::AutotuneBlockChecksSubmenu
                                | ActiveScreen::AutotunePresetSelectionSubmenu
                                | ActiveScreen::AutotuneStrategiesSubmenu
                                | ActiveScreen::AutotuneResultsSubmenu => {
                                    app.active_screen = ActiveScreen::AutotuneSubmenu;
                                }
                                ActiveScreen::Main => {
                                    app.should_quit = true;
                                }
                                _ => {
                                    app.active_screen = ActiveScreen::Main;
                                }
                            }
                        }
                        _ => {}
                    }

                    match app.active_screen {
                        ActiveScreen::DownloadDepsSubmenu
                        | ActiveScreen::DownloadZapretSubmenu
                        | ActiveScreen::DownloadStrategiesSubmenu
                        | ActiveScreen::ZapretTagSelect
                        | ActiveScreen::StrategyTagSelect => {
                            app.refresh_dep_status();
                        }
                        ActiveScreen::ServiceSubmenu => {
                            app.refresh_service_status();
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.should_download_zapret {
            app.should_download_zapret = false;
            
            let nfqws_target_string;
            let nfqws_ver = match &app.nfqws_target {
                VersionTarget::Recommended => crate::download::ZAPRET_REC_VER,
                VersionTarget::Latest => "latest",
                VersionTarget::Tag(t) => {
                    nfqws_target_string = t.clone();
                    &nfqws_target_string
                }
            };
            
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            
            let res = crate::download::install_dependencies(nfqws_ver, "skip");
            
            if res.is_ok() {
                println!("{}", rust_i18n::t!("msg_dl_ok"));
            } else {
                let err_msg = res.as_ref().unwrap_err();
                println!("{}{}", rust_i18n::t!("msg_dl_fail"), err_msg);
                println!("{}", rust_i18n::t!("msg_dl_key"));
            }
            
            // Wait for keypress
            loop {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        break;
                    }
                }
            }
            
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            terminal.clear()?;
            
            if let Err(e) = res {
                app.show_error(e.to_string());
            } else {
                app.status_message = Some(rust_i18n::t!("msg_dl_zapret_ok").into_owned());
                app.active_screen = ActiveScreen::DownloadZapretSubmenu;
                app.refresh_dep_status();
            }
        }

        if app.should_download_strategies {
            app.should_download_strategies = false;
            
            let strat_target_string;
            let strat_ver = match &app.strat_target {
                VersionTarget::Recommended => "recommended",
                VersionTarget::Latest => "latest",
                VersionTarget::Tag(t) => {
                    strat_target_string = t.clone();
                    &strat_target_string
                }
            };
            
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            
            let res = crate::download::install_dependencies("skip", strat_ver);
            
            if res.is_ok() {
                println!("{}", rust_i18n::t!("msg_dl_ok"));
            } else {
                let err_msg = res.as_ref().unwrap_err();
                println!("{}{}", rust_i18n::t!("msg_dl_fail"), err_msg);
                println!("{}", rust_i18n::t!("msg_dl_key"));
            }
            
            // Wait for keypress
            loop {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        break;
                    }
                }
            }
            
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            terminal.clear()?;
            
            if let Err(e) = res {
                app.show_error(e.to_string());
            } else {
                app.status_message = Some(rust_i18n::t!("msg_dl_strat_ok").into_owned());
                app.strategies = crate::strategy::get_strategies();
                app.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
                app.refresh_dep_status();
            }
        }

        if app.should_download_defaults {
            app.should_download_defaults = false;
            
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;
            
            let res = crate::download::install_dependencies(crate::download::ZAPRET_REC_VER, "recommended");
            
            if res.is_ok() {
                println!("{}", rust_i18n::t!("msg_dl_ok"));
            } else {
                let err_msg = res.as_ref().unwrap_err();
                println!("{}{}", rust_i18n::t!("msg_dl_fail"), err_msg);
                println!("{}", rust_i18n::t!("msg_dl_key"));
            }
            
            // Wait for keypress
            loop {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        break;
                    }
                }
            }
            
            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            terminal.clear()?;
            
            if let Err(e) = res {
                app.show_error(e.to_string());
            } else {
                app.status_message = Some(rust_i18n::t!("msg_dl_all_ok").into_owned());
                app.strategies = crate::strategy::get_strategies();
                app.active_screen = ActiveScreen::DownloadDepsSubmenu;
            }
        }

        if let Some(file_path) = app.should_open_editor.take() {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;

            let _ = crate::utils::open_editor(&file_path);

            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            terminal.clear()?;

            app.status_message = Some(format!("{}{}", rust_i18n::t!("msg_closed_editor"), std::path::Path::new(&file_path).file_name().unwrap_or_default().to_string_lossy()));
            if app.active_screen != ActiveScreen::AutotuneSubmenu {
                app.active_screen = ActiveScreen::ListsEditorSubmenu;
                app.refresh_ipset_status();
            }
        }

        if app.should_run_autotune {
            app.should_run_autotune = false;
            app.autotune_running = true;
            app.autotune_results = None;

            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            terminal.show_cursor()?;

            println!("{}", rust_i18n::t!("autotune_running"));
            println!();
            let config = &app.autotune_config;
            let interface = app.interfaces.get(app.selected_interface)
                .map(|s| s.as_str())
                .unwrap_or("any");
            #[cfg(target_os = "linux")]
            let backend: &dyn crate::firewalls::FirewallBackend = &app.selected_backend;
            #[cfg(target_os = "windows")]
            let backend: &dyn crate::firewalls::FirewallBackend = &crate::firewalls::windivert::WinDivertBackend;
            let results = crate::autotune::run_all(config, &|done, total| {
                let pct = done * 100 / total.max(1);
                print!("\r  {} {}/{} ({}%)", rust_i18n::t!("autotune_progress"), done, total, pct);
                let _ = std::io::stdout().flush();
            }, backend, interface);
            // Save results to file for persistence across restarts
            crate::autotune::save_results_file(&results);
            println!();
            println!();
            println!("{}", rust_i18n::t!("autotune_done"));
            println!();
            println!("{}", rust_i18n::t!("autotune_how_to_read"));
            println!();
            println!("--- {} ---", rust_i18n::t!("menu_autotune_net_checks"));
            let check_labels = ["DNS", "TCP RST", "SNI", "SIBERIAN", "QUIC", "CIDR"];
            for (label, check) in check_labels.iter().zip(&results.block_results) {
                println!("  {}: {} - {}", label, status_str(&check.status), status_detail(&check.status));
            }
            println!();

            for pr in &results.preset_results {
                println!("--- {} [{}] ---", rust_i18n::t!("autotune_domain_results"), pr.preset_name);
                let req_count = pr.domain_checks.first().map(|_| app.autotune_config.num_requests).unwrap_or(3);
                for dc in &pr.domain_checks {
                    println!("  {}: alive={} HTTP:{}({}/{}) HTTPS:{}({}/{}) TLS1.2={} TLS1.3={} QUIC:{}({}/{}) baseline={}",
                        dc.domain,
                        status_str(&dc.alive),
                        status_str(&dc.http), dc.http_count, req_count,
                        status_str(&dc.https), dc.https_count, req_count,
                        status_str(&dc.tls12),
                        status_str(&dc.tls13),
                        status_str(&dc.quic), dc.quic_count, req_count,
                        status_str(if dc.baseline_pass { &CheckStatus::Pass } else { &CheckStatus::Fail }),
                    );
                }
                if !pr.strategy_results.is_empty() {
                    println!();
                    println!("  --- {} ---", rust_i18n::t!("autotune_strat_results"));
                    for sr in &pr.strategy_results {
                        let status = if sr.works { "✅ WORKS" } else { "❌ FAILS" };
                        let protos = if sr.protocols_working.is_empty() {
                            String::new()
                        } else {
                            format!(" [{}]", sr.protocols_working.join(", "))
                        };
                        println!("    {}: {} ({}/{} blocked domains unblocked){}", sr.strategy_name, status, sr.score(), sr.total(), protos);
                        for dc in &sr.domain_checks {
                            println!("      {} HTTP:{} HTTPS:{} T12:{} T13:{} Q:{}",
                                dc.domain,
                                if dc.http { "✅" } else { "❌" },
                                if dc.https { "✅" } else { "❌" },
                                if dc.tls12 { "✅" } else { "❌" },
                                if dc.tls13 { "✅" } else { "❌" },
                                if dc.quic { "✅" } else { "❌" },
                            );
                        }
                    }
                    let working: Vec<&StrategyCheckResult> = pr.strategy_results.iter().filter(|s| s.works).collect();
                    if working.is_empty() {
                        println!("    {}", rust_i18n::t!("autotune_strat_none_work"));
                    } else {
                        println!("    {} {} {}", rust_i18n::t!("autotune_strat_works_count"), working.len(), rust_i18n::t!("autotune_strat_of_total").replace("{}", &pr.strategy_results.len().to_string()));
                        for s in &working {
                            println!("      ✅ {} ({}/{})", s.strategy_name, s.score(), s.total());
                        }
                    }
                }
                println!();
            }

            if !results.common_strategies.is_empty() {
                println!("--- {} ({}) ---", rust_i18n::t!("autotune_common_strats"), results.common_strategies.len());
                for name in &results.common_strategies {
                    println!("  ✅ {}", name);
                }
                println!();
            }
            println!("{}", rust_i18n::t!("msg_dl_key"));
            println!("{}", rust_i18n::t!("msg_dl_key"));

            loop {
                if let Ok(Event::Key(key)) = event::read() {
                    if key.kind == KeyEventKind::Press {
                        break;
                    }
                }
            }

            enable_raw_mode()?;
            execute!(terminal.backend_mut(), EnterAlternateScreen)?;
            terminal.clear()?;

            app.autotune_results = Some(results);
            app.autotune_running = false;
            app.status_message = Some(rust_i18n::t!("autotune_done").into_owned());
        }

        if app.should_run || app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
