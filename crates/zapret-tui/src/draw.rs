//! One frame of the UI.
//!
//! Pure rendering: it reads [`AppState`] and paints. The shape is the same on
//! every screen, so a menu, a picker and a report all sit in the same place and
//! the eye does not have to find them again:
//!
//! ```text
//! ┌ zapret-rust 2.1.0 ─────────────────────────────────────────────┐
//! │ Main ▸ Autotune                                                │  where you are
//! │ ┌ Blocking Checks ──────────────────────────────────────────┐ │
//! │ │ ▌ 🌐 Domains            < Discord >                        │ │
//! │ │   📋 Strategies         < 4 / 12 >                         │ │
//! │ └────────────────────────────────────────────────────────────┘ │
//! │ 💡 Submenu: pick the domain presets to test against.            │  what this row does
//! │ nfqws ✔  strategies ✔  Windows Service: active                 │  what is installed
//! ├────────────────────────────────────────────────────────────────┤
//! │ ↑↓ move  ←→ change  ⏎ open  esc back                           │  what the keys do
//! └────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Only the middle block changes. Everything around it is chrome, drawn here
//! once so no screen has to know about it.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, List, ListItem, ListState, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};
use ratatui::Frame;
use unicode_width::UnicodeWidthStr;

use crate::menus::{self, Menu};
use crate::state::{
    ActiveScreen, AppState, AutotuneBlockChecksState, AutotuneMenuState, AutotuneProtocolsState, DownloadDepsMenuState,
    DownloadSubmenuState, ExtendedMenuState, FakesMenuState, GamefilterMenuState, MainMenuState, TtlMenuState,
};
use crate::theme::Theme;
use crate::views;

/// The bar in the gutter on the row the cursor is on.
///
/// Two cells wide, and that width is the gutter every other row pads itself
/// out to, so a row's text never shifts sideways when the cursor moves onto it.
const MARKER: &str = "▌ ";

/// Screens that show what is installed under the menu, because on those screens
/// it changes what the rows mean.
fn shows_status_line(screen: ActiveScreen) -> bool {
    matches!(
        screen,
        ActiveScreen::Main
            | ActiveScreen::ServiceSubmenu
            | ActiveScreen::ExtendedSubmenu
            | ActiveScreen::DownloadDepsSubmenu
            | ActiveScreen::DownloadZapretSubmenu
            | ActiveScreen::DownloadStrategiesSubmenu
            | ActiveScreen::ZapretTagSelect
            | ActiveScreen::StrategyTagSelect
    )
}

// ------------------------------------------------------------------ chrome --

/// `Main ▸ Autotune ▸ Domains`, cut rather than wrapped if the terminal is narrow.
fn breadcrumb(app: &AppState) -> Line<'static> {
    let trail = app.history.trail(app.active_screen);
    let mut spans = vec![Span::styled(" ", Theme::crumb())];
    for (i, screen) in trail.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" ▸ ", Theme::frame()));
        }
        // The first crumb is where you started; the rest are where you went.
        let style = if i == 0 { Theme::crumb() } else { Theme::accent() };
        spans.push(Span::styled(screen.label().trim().to_string(), style));
    }
    Line::from(spans)
}

/// The key hints on the bottom border. The same on every screen, because the
/// keys do not change — only what they act on does, and that is the help line.
fn key_hints() -> Line<'static> {
    let keys = [("↑↓", "move"), ("←→", "change"), ("⏎", "open"), ("esc", "back")];
    let mut spans = vec![Span::raw(" ")];
    for (i, (key, what)) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("   ", Theme::frame()));
        }
        spans.push(Span::styled(*key, Theme::key()));
        spans.push(Span::styled(*what, Theme::crumb()));
    }
    Line::from(spans)
}

/// The name of the init system, spelled per platform.
fn service_kind() -> String {
    #[cfg(target_os = "linux")]
    {
        zapret_wrapper::service::detect_init_system()
            .map(|t| t.as_str().to_string())
            .unwrap_or_else(|| rust_i18n::t!("status_srv_unknown").into_owned())
    }
    #[cfg(target_os = "windows")]
    {
        rust_i18n::t!("status_srv_win").into_owned()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        rust_i18n::t!("status_srv_unknown").into_owned()
    }
}

/// What is installed and whether the service is up. One line, read at a glance.
fn service_line(app: &AppState) -> Line<'static> {
    let (mark, color, state) = if !app.service_installed {
        ("✖", Color::Red, rust_i18n::t!("status_srv_not_inst").into_owned())
    } else if app.service_active {
        ("✔", Color::Green, rust_i18n::t!("status_srv_active").into_owned())
    } else {
        ("●", Color::Yellow, rust_i18n::t!("status_srv_stopped").into_owned())
    };

    let chip = |name: &str, ok: bool| {
        vec![
            Span::styled(format!(" {name}"), Theme::muted()),
            Span::styled(
                if ok { " ✔" } else { " ✖" },
                Style::default().fg(if ok { Color::Green } else { Color::Red }),
            ),
        ]
    };

    let mut spans = vec![
        Span::styled(format!(" {mark} "), Style::default().fg(color)),
        Span::styled(service_kind(), Theme::muted()),
        Span::styled(format!(" {state}"), Style::default().fg(color)),
        Span::styled("   ", Theme::frame()),
    ];
    spans.extend(chip("nfqws", app.nfqws_installed));
    spans.extend(chip("strategies", app.strategies_installed));

    Line::from(spans)
}

/// Cut a styled line to `width` cells, keeping every span's colour.
///
/// The status line is built from spans that each carry their own colour, so it
/// cannot go through [`views::fit`], which would flatten it to plain text.
fn clip(line: Line<'static>, width: u16) -> Line<'static> {
    let mut used = 0usize;
    let mut spans = Vec::new();
    for span in line.spans {
        let w = span.content.width();
        if used + w <= width as usize {
            used += w;
            spans.push(span);
        } else if used < width as usize {
            let keep = width as usize - used;
            spans.push(Span::styled(views::fit(&span.content, keep as u16), span.style));
            used = width as usize;
        } else {
            break;
        }
    }
    Line::from(spans)
}

// -------------------------------------------------------------------- menus --

/// The menu the current screen is showing.
fn menu_for(app: &AppState) -> Menu {
    match app.active_screen {
        ActiveScreen::Main => menus::main_menu::render(app),
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => menus::defender_menu::render(app),
        ActiveScreen::StrategySubmenu => menus::strategy_menu::render(app),
        ActiveScreen::DownloadDepsSubmenu => menus::download_menu::render(app),
        ActiveScreen::DownloadZapretSubmenu => menus::download_submenu::render(app, true),
        ActiveScreen::DownloadStrategiesSubmenu => menus::download_submenu::render(app, false),
        ActiveScreen::GamefilterSubmenu => menus::gamefilter_menu::render(app),
        ActiveScreen::ExtendedSubmenu => menus::extended_menu::render(app),
        ActiveScreen::TtlSubmenu => menus::ttl_menu::render(app),
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
        ActiveScreen::AutotuneStrategiesSubmenu => menus::autotune_menu::render_strategies(app),
        // Never reached: the report owns the whole middle band before a menu is
        // ever asked for. Kept total so adding a screen cannot fall through.
        ActiveScreen::AutotuneResultsSubmenu => Menu::new("", Vec::new()),
    }
}

/// Turn a [`Menu`] into the rows ratatui draws.
///
/// The gutter is the mark, then [`MARKER`]'s two cells. On the selected row
/// ratatui draws the highlight symbol there instead, which is why the padding is
/// only added to the rows that are *not* selected: same width either way, so
/// the labels line up.
fn list_items(menu: &Menu) -> Vec<ListItem<'static>> {
    let gutter = MARKER.width();
    menu.rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            // A heading is scenery. It gets the gutter padding like any other
            // row so nothing shifts, but it keeps its own colour and never
            // carries a value, because there is nothing there to set.
            let style = if row.heading { Theme::heading() } else { Theme::label() };
            let mut spans = vec![Span::styled(if row.marked { "●" } else { " " }, Theme::accent())];
            if i != menu.index {
                spans.push(Span::raw(" ".repeat(gutter)));
            }
            spans.push(Span::styled(row.label.clone(), style));
            if let Some(value) = &row.value {
                spans.push(Span::styled(
                    format!("  {value}"),
                    row.value_style.unwrap_or_else(Theme::value),
                ));
            }
            ListItem::new(Line::from(spans))
        })
        .collect()
}

/// Paint the middle band: the report where there is one, a menu everywhere else.
fn draw_body(f: &mut Frame, app: &mut AppState, area: Rect) {
    // The report is tables, not rows: it scrolls by a line and has no cursor,
    // so it takes the whole band and never goes through the menu pipeline.
    if app.active_screen == ActiveScreen::AutotuneResultsSubmenu {
        if let Some(ref results) = app.autotune_results {
            views::report::render(
                f,
                area,
                results,
                app.autotune_report_tab,
                &mut app.autotune_results_index,
            );
            return;
        }
    }
    draw_menu(f, app, area);
}

fn draw_menu(f: &mut Frame, app: &AppState, area: Rect) {
    let menu = menu_for(app);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Theme::frame())
        .title(Line::from(Span::styled(
            format!(" {} ", menu.title.trim()),
            Theme::title(),
        )));
    let inner = block.inner(area);
    f.render_widget(block, area);

    // The autotune screen warns before it runs, and the warning belongs inside
    // the block it is warning about rather than in a frame of its own.
    let warning = app.active_screen == ActiveScreen::AutotuneSubmenu && !app.autotune_running;
    let (warning_area, list_area) = if warning {
        let [top, rest] = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
        (top, rest)
    } else {
        (Rect::ZERO, inner)
    };

    if warning {
        let text = rust_i18n::t!("autotune_warning_disable").into_owned();
        f.render_widget(
            Paragraph::new(Span::styled(
                views::fit(&text, inner.width),
                Theme::warn().add_modifier(Modifier::BOLD),
            )),
            warning_area,
        );
    }

    let list = List::new(list_items(&menu))
        .highlight_symbol(MARKER)
        .highlight_style(Theme::selection());
    let mut state = ListState::default().with_selected(Some(menu.index));
    f.render_stateful_widget(list, list_area, &mut state);

    // A scrollbar only where there is something to scroll: an always-on rail
    // beside a six-row menu claims there is more below, and there is not.
    let rows = menu.rows.len();
    if rows as u16 > list_area.height && list_area.height > 1 {
        let mut scroll = ScrollbarState::new(rows)
            .position(state.offset().min(rows.saturating_sub(1)))
            .viewport_content_length(list_area.height as usize);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("│"))
                .thumb_symbol("█")
                .style(Theme::frame()),
            list_area,
            &mut scroll,
        );
    }
}

// --------------------------------------------------------------------- help --

/// The locale key of the one sentence about the row under the cursor.
fn help_key(app: &AppState) -> &'static str {
    match app.active_screen {
        ActiveScreen::Main => match app.main_menu {
            #[cfg(target_os = "windows")]
            MainMenuState::DefenderSettings => "help_def",
            MainMenuState::DownloadDeps => "help_dl",
            #[cfg(target_os = "linux")]
            MainMenuState::Interface => "help_iface",
            MainMenuState::Strategy => "help_strat",
            MainMenuState::GamefilterSettings => "help_gf",
            #[cfg(target_os = "linux")]
            MainMenuState::BackendSettings => "help_backend",
            MainMenuState::IpsetMode => "help_ipset",
            MainMenuState::ListsEditor => "help_lists",
            MainMenuState::Autotune => "help_autotune",
            MainMenuState::Extended => "help_extended",
            MainMenuState::ServiceSettings => "help_srv",
            MainMenuState::Run => "help_run",
            MainMenuState::Quit => "help_quit",
        },
        ActiveScreen::ExtendedSubmenu => match app.extended_menu {
            ExtendedMenuState::Ttl => "help_ttl",
            ExtendedMenuState::Fakes => "help_fakes",
            ExtendedMenuState::Back => "help_back",
        },
        ActiveScreen::DownloadDepsSubmenu => match app.download_deps_menu {
            DownloadDepsMenuState::ZapretDownloader => "help_dl_zap",
            DownloadDepsMenuState::StrategiesDownloader => "help_dl_str",
            DownloadDepsMenuState::DownloadDefaults => "help_dl_def",
            DownloadDepsMenuState::Back => "help_back",
        },
        ActiveScreen::DownloadZapretSubmenu => match app.download_zapret_menu {
            DownloadSubmenuState::Version => "help_dl_ver",
            DownloadSubmenuState::SelectTag => "help_dl_tag",
            DownloadSubmenuState::Start => "help_dl_start",
            DownloadSubmenuState::Back => "help_back",
        },
        ActiveScreen::DownloadStrategiesSubmenu => match app.download_strategies_menu {
            DownloadSubmenuState::Version => "help_dl_ver",
            DownloadSubmenuState::SelectTag => "help_dl_tag",
            DownloadSubmenuState::Start => "help_dl_start",
            DownloadSubmenuState::Back => "help_back",
        },
        ActiveScreen::GamefilterSubmenu => match app.gamefilter_menu {
            GamefilterMenuState::Tcp => "help_gf_tcp",
            GamefilterMenuState::Udp => "help_gf_udp",
            GamefilterMenuState::Back => "help_back",
        },
        ActiveScreen::FakesSubmenu => match app.fakes_menu {
            FakesMenuState::DiscordUdp | FakesMenuState::GameUdp => "help_fakes_sel",
            FakesMenuState::Back => "help_back",
        },
        ActiveScreen::FakesSelectSubmenu => "help_fakes_select",
        #[cfg(target_os = "windows")]
        ActiveScreen::DefenderSubmenu => "help_def_sel",
        ActiveScreen::StrategySubmenu => "help_strat_sel",
        ActiveScreen::ZapretTagSelect | ActiveScreen::StrategyTagSelect => "help_tag_sel",
        ActiveScreen::ServiceSubmenu => "help_srv_sel",
        ActiveScreen::ListsEditorSubmenu => "help_lists",
        ActiveScreen::TtlSubmenu => match app.ttl_menu {
            TtlMenuState::SetValue => "help_ttl_value",
            TtlMenuState::Autopick => "help_ttl_autopick",
            _ => "help_ttl",
        },
        ActiveScreen::AutotuneSubmenu => match app.autotune_menu {
            AutotuneMenuState::PresetSelection => "help_autotune_domains",
            AutotuneMenuState::NumRequests => "help_autotune_req",
            AutotuneMenuState::Strategies => "help_autotune_strat_sel",
            AutotuneMenuState::Protocols => "help_autotune_proto",
            AutotuneMenuState::BlockChecks => "help_autotune_blockchecks",
            AutotuneMenuState::EditDomains => "help_autotune_edit_domains",
            AutotuneMenuState::Results => "help_autotune_results_sel",
            AutotuneMenuState::Run => "help_autotune_run",
            AutotuneMenuState::Back => "help_back",
        },
        ActiveScreen::AutotuneProtocolsSubmenu => match app.autotune_protocols_menu {
            AutotuneProtocolsState::Back => "help_back",
            _ => "help_autotune_toggle",
        },
        ActiveScreen::AutotuneBlockChecksSubmenu => match app.autotune_block_checks_menu {
            AutotuneBlockChecksState::Back => "help_back",
            _ => "help_autotune_toggle",
        },
        ActiveScreen::AutotuneEditDomainsSubmenu => "help_autotune_edit_domains",
        ActiveScreen::AutotunePresetSelectionSubmenu => "help_autotune_presets",
        ActiveScreen::AutotuneStrategiesSubmenu => "help_autotune_strat",
        ActiveScreen::AutotuneResultsSubmenu => "help_autotune_results",
    }
}

// -------------------------------------------------------------------- frame --

pub fn draw(f: &mut Frame, app: &mut AppState) {
    let frame = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Theme::frame())
        .title(Line::from(vec![
            Span::styled(" zapret-rust ", Theme::title()),
            Span::styled(env!("CARGO_PKG_VERSION"), Theme::crumb()),
        ]))
        .title_bottom(key_hints());

    let area = f.area();
    let inner = frame.inner(area);
    f.render_widget(frame, area);
    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // The status line is only reserved on the screens that show it, rather than
    // left empty on the rest.
    let with_status = shows_status_line(app.active_screen);
    let [crumbs, body, help, status] = Layout::vertical(if with_status {
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ]
    } else {
        [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(0),
        ]
    })
    .areas(inner);

    f.render_widget(Paragraph::new(breadcrumb(app)), crumbs);
    draw_body(f, app, body);

    // A message from the last key replaces the row's own explanation: it is the
    // answer to what the user just did, and the row's description has not
    // changed since the screen loaded.
    let (text, style) = match &app.status_message {
        Some(msg) => (msg.clone(), Theme::accent()),
        None => (rust_i18n::t!(help_key(app)).into_owned(), Theme::muted()),
    };
    f.render_widget(
        Paragraph::new(clip(Line::from(Span::styled(text, style)), help.width)),
        help,
    );

    if with_status {
        f.render_widget(Paragraph::new(clip(service_line(app), status.width)), status);
    }
}
