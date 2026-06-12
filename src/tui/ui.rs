use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, Paragraph},
    Terminal,
};
use std::io;

use crate::tui::state::{
    ActiveScreen, AppState, VersionTarget, MainMenuState,
    DownloadDepsMenuState, DownloadSubmenuState,
    GamefilterMenuState,
};
use crate::tui::menus;
use crate::tui::theme::Theme;

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
                    ("\u{F00D}", Color::Red, rust_i18n::t!("status_srv_not_inst"))
                } else if app.service_active {
                    ("\u{F00C}", Color::Green, rust_i18n::t!("status_srv_active"))
                } else {
                    ("\u{F04C}", Color::Yellow, rust_i18n::t!("status_srv_stopped"))
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
                let mut show_error = false;
                let mut error_msg = String::new();
                if let Some((ref msg, instant)) = app.dependency_error {
                    if instant.elapsed() < std::time::Duration::from_secs(3) {
                        show_error = true;
                        error_msg = msg.clone();
                    }
                }

                let status_text = if show_error {
                    Line::from(vec![
                        Span::styled(error_msg, Style::default().fg(Color::Red).add_modifier(ratatui::style::Modifier::BOLD)),
                    ])
                } else {
                    let nfqws_status = if app.nfqws_installed { "✅" } else { "❌" };
                    let strat_status = if app.strategies_installed { "✅" } else { "❌" };
                    Line::from(vec![
                        Span::styled(rust_i18n::t!("status_deps_title"), Style::default().fg(Color::Gray)),
                        Span::styled("nfqws ", Style::default().fg(Color::White)),
                        Span::raw(nfqws_status),
                        Span::styled(format!(" | {} ", rust_i18n::t!("status_deps_strat")), Style::default().fg(Color::White)),
                        Span::raw(strat_status),
                    ])
                };
                let status_paragraph = Paragraph::new(status_text)
                    .alignment(ratatui::layout::Alignment::Center);
                
                f.render_widget(status_paragraph, main_chunks[2]);
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
                    MainMenuState::Strategy => rust_i18n::t!("help_strat"),
                    MainMenuState::GamefilterSettings => rust_i18n::t!("help_gf"),
                    MainMenuState::ServiceSettings => rust_i18n::t!("help_srv"),
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
            };

            let help_text = if let Some(ref msg) = app.status_message {
                format!("INFO: {} | {}", msg, dynamic_help)
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
                    match key.code {
                        KeyCode::Up => app.prev_menu(),
                        KeyCode::Down => app.next_menu(),
                        KeyCode::Left => app.cycle_current(false),
                        KeyCode::Right => app.cycle_current(true),
                        KeyCode::Enter | KeyCode::Char(' ') => app.cycle_current(true),
                        KeyCode::Char('q') | KeyCode::Esc => {
                            if app.active_screen != ActiveScreen::Main {
                                app.active_screen = ActiveScreen::Main;
                            } else {
                                app.should_quit = true;
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
                app.status_message = Some(format!("Error: {}", e));
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
                app.status_message = Some(format!("Error: {}", e));
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
                app.status_message = Some(format!("Error: {}", e));
            } else {
                app.status_message = Some(rust_i18n::t!("msg_dl_all_ok").into_owned());
                app.strategies = crate::strategy::get_strategies();
                app.active_screen = ActiveScreen::DownloadDepsSubmenu;
            }
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
