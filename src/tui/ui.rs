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
                ActiveScreen::Main => " 🚀 Zapret-Rust TUI ",
                #[cfg(target_os = "windows")]
                ActiveScreen::DefenderSubmenu => " 🛡️ Windows Defender Settings ",
                ActiveScreen::StrategySubmenu => " 📜 Select Strategy ",
                ActiveScreen::DownloadDepsSubmenu => " 📥 Downloader Categories ",
                ActiveScreen::DownloadZapretSubmenu => " ⚙️ Zapret Downloader ",
                ActiveScreen::DownloadStrategiesSubmenu => " 📜 Strategies Downloader ",
                ActiveScreen::GamefilterSubmenu => " 🎮 Game Filter Settings ",
                ActiveScreen::ZapretTagSelect => " 🏷️ Select Zapret Tag ",
                ActiveScreen::StrategyTagSelect => " 🏷️ Select Strategies Tag ",
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
                ActiveScreen::ZapretTagSelect => menus::tag_menu::render(&app.available_nfqws_tags, app.nfqws_tag_index, " Select Zapret Tag "),
                ActiveScreen::StrategyTagSelect => menus::tag_menu::render(&app.available_strat_tags, app.strat_tag_index, " Select Strategy Tag "),
            };

            let list_block = Block::default()
                .title(Span::styled(block_title, Theme::block_title()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Theme::dim_item());

            if app.active_screen == ActiveScreen::Main {
                let inner_area = list_block.inner(chunks[1]);
                let main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ])
                    .split(inner_area);

                f.render_widget(list_block, chunks[1]);

                let list = List::new(items)
                    .highlight_style(Style::default().add_modifier(ratatui::style::Modifier::ITALIC));
                let mut list_state = ratatui::widgets::ListState::default();
                list_state.select(Some(selected_index));
                f.render_stateful_widget(list, main_chunks[0], &mut list_state);

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
                        Span::styled("📥 Dependencies Status: ", Style::default().fg(Color::Gray)),
                        Span::styled("nfqws ", Style::default().fg(Color::White)),
                        Span::raw(nfqws_status),
                        Span::styled(" | strategies ", Style::default().fg(Color::White)),
                        Span::raw(strat_status),
                    ])
                };
                let status_paragraph = Paragraph::new(status_text)
                    .alignment(ratatui::layout::Alignment::Center);
                
                f.render_widget(status_paragraph, main_chunks[1]);
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
                    MainMenuState::DefenderSettings => "💡 Windows Defender: Add or remove this folder from antivirus exclusions.",
                    MainMenuState::DownloadDeps => "💡 Downloader: Select and download/update Zapret components or strategies.",
                    MainMenuState::Interface => "💡 Toggle: Choose network interface. Press SPACE/ENTER to cycle.",
                    MainMenuState::Strategy => "💡 Select: Choose strategy folder or script to use.",
                    MainMenuState::GamefilterSettings => "💡 Submenu: Open TCP/UDP Game Filter ports configuration.",
                    MainMenuState::Run => "💡 Action: Run Zapret with selected configuration.",
                    MainMenuState::Quit => "💡 Action: Exit the TUI application.",
                },
                ActiveScreen::DownloadDepsSubmenu => match app.download_deps_menu {
                    DownloadDepsMenuState::ZapretDownloader => "💡 Select: Go to Zapret (nfqws) downloader settings.",
                    DownloadDepsMenuState::StrategiesDownloader => "💡 Select: Go to Strategies downloader settings.",
                    DownloadDepsMenuState::Back => "💡 Action: Return to the configuration menu.",
                },
                ActiveScreen::DownloadZapretSubmenu => match app.download_zapret_menu {
                    DownloadSubmenuState::Version => "💡 Toggle: Choose Zapret version. Press SPACE/ENTER to cycle (Recommended ➔ Latest).",
                    DownloadSubmenuState::SelectTag => "💡 Select: Fetch and choose a specific Git tag/release of Zapret.",
                    DownloadSubmenuState::Start => "💡 Action: Download and install the selected Zapret version.",
                    DownloadSubmenuState::Back => "💡 Action: Go back to Downloader categories.",
                },
                ActiveScreen::DownloadStrategiesSubmenu => match app.download_strategies_menu {
                    DownloadSubmenuState::Version => "💡 Toggle: Choose Strategies version. Press SPACE/ENTER to cycle (Recommended ➔ Latest).",
                    DownloadSubmenuState::SelectTag => "💡 Select: Fetch and choose a specific Git tag/release of strategies.",
                    DownloadSubmenuState::Start => "💡 Action: Download and install the selected strategies.",
                    DownloadSubmenuState::Back => "💡 Action: Go back to Downloader categories.",
                },
                ActiveScreen::GamefilterSubmenu => match app.gamefilter_menu {
                    GamefilterMenuState::Tcp => "💡 Toggle: Intercept TCP ports for game traffic. Press SPACE/ENTER or Left/Right.",
                    GamefilterMenuState::Udp => "💡 Toggle: Intercept UDP ports for game traffic. Press SPACE/ENTER or Left/Right.",
                    GamefilterMenuState::Back => "💡 Action: Return to the main configuration menu.",
                },
                #[cfg(target_os = "windows")]
                ActiveScreen::DefenderSubmenu => "💡 Defender: Select add/remove exclusion or back.",
                ActiveScreen::StrategySubmenu => "💡 Strategy Select: Choose a strategy from the downloaded strategies list.",
                ActiveScreen::ZapretTagSelect => "💡 Select: Use UP/DOWN to navigate, ENTER to select a specific Zapret tag.",
                ActiveScreen::StrategyTagSelect => "💡 Select: Use UP/DOWN to navigate, ENTER to select a specific Strategies tag.",
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
                println!("\n✅ Download completed successfully. Press any key to return to the menu...");
            } else {
                let err_msg = res.as_ref().unwrap_err();
                println!("\n❌ Download failed: {}", err_msg);
                println!("Press any key to return to the menu...");
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
                app.status_message = Some("Zapret successfully downloaded and installed.".to_string());
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
                println!("\n✅ Download completed successfully. Press any key to return to the menu...");
            } else {
                let err_msg = res.as_ref().unwrap_err();
                println!("\n❌ Download failed: {}", err_msg);
                println!("Press any key to return to the menu...");
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
                app.status_message = Some("Strategies successfully downloaded and installed.".to_string());
                app.strategies = crate::strategy::get_strategies();
                app.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
                app.refresh_dep_status();
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
