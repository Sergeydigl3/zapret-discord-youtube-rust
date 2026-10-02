//! The frame every screen is drawn inside.
//!
//! Only chrome lives here: the border, the breadcrumb, the key hints and the
//! two lines under the body. The middle band belongs to the screen's own
//! component, which is why there is no table of menus, no help table and no
//! mouse table left in this file — the screen that is on owns all three.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};
use ratatui::Frame;

use crate::state::AppState;
use crate::theme::Theme;
use crate::views::fit_line;

/// The bands the frame is split into.
pub struct Chrome {
    pub body: Rect,
    pub help: Rect,
    /// `None` on the screens that do not show it, rather than an empty line.
    pub status: Option<Rect>,
}

/// Draw the border, the breadcrumb and the key hints, and hand back the bands.
///
/// `warning` is the focused screen's own banner. It gets a band of its own
/// above the body instead of being painted inside the body's frame: a line drawn
/// into the frame would land on top of the screen's first row, and the first row
/// is exactly where the cursor starts.
pub fn chrome(f: &mut Frame, state: &AppState, warning: Option<&str>) -> Chrome {
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
        return Chrome {
            body: Rect::ZERO,
            help: Rect::ZERO,
            status: None,
        };
    }

    // The status line is only reserved on the screens that show it, rather than
    // left empty on the rest; the warning banner only on the ones that have one.
    let with_status = state.active_screen.shows_status_line();
    let [crumbs, warn, body, help, status] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(u16::from(warning.is_some())),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(u16::from(with_status)),
    ])
    .areas(inner);

    f.render_widget(Paragraph::new(breadcrumb(state)), crumbs);
    if let Some(text) = warning {
        f.render_widget(
            Paragraph::new(fit_line(
                Line::from(Span::styled(
                    text.to_string(),
                    Theme::warn().add_modifier(Modifier::BOLD),
                )),
                warn.width,
            )),
            warn,
        );
    }

    Chrome {
        body,
        help,
        status: with_status.then_some(status),
    }
}

/// `Main ▸ Autotune ▸ Domains`, cut rather than wrapped if the terminal is narrow.
fn breadcrumb(state: &AppState) -> Line<'static> {
    let mut spans = vec![Span::styled(" ", Theme::crumb())];
    for (i, screen) in state.history.trail(state.active_screen).iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" ▸ ", Theme::frame()));
        }
        // The first crumb is where you started; the rest are where you went.
        let style = if i == 0 { Theme::crumb() } else { Theme::accent() };
        spans.push(Span::styled(screen.label().trim().to_string(), style));
    }
    Line::from(spans)
}

/// The key hints on the bottom border. The same on every screen, because the keys
/// do not change — only what they act on does, and that is the help line.
fn key_hints() -> Line<'static> {
    let mut spans = vec![Span::raw(" ")];
    for (i, (key, what)) in [("↑↓", "move"), ("←→", "change"), ("⏎", "open"), ("esc", "back")]
        .iter()
        .enumerate()
    {
        if i > 0 {
            spans.push(Span::styled("   ", Theme::frame()));
        }
        spans.push(Span::styled(*key, Theme::key()));
        spans.push(Span::styled(*what, Theme::crumb()));
    }
    Line::from(spans)
}

/// The one line under the body: what the row under the cursor does, or the
/// answer to what the user just did.
pub fn help(f: &mut Frame, area: Rect, text: String, state: &AppState) {
    let style = if state.status_message.is_some() {
        Theme::accent()
    } else {
        Theme::muted()
    };
    f.render_widget(
        Paragraph::new(fit_line(Line::from(Span::styled(text, style)), area.width)),
        area,
    );
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
pub fn status(f: &mut Frame, area: Rect, state: &AppState) {
    let (mark, color, text) = if !state.service_installed {
        ("✖", Color::Red, rust_i18n::t!("status_srv_not_inst").into_owned())
    } else if state.service_active {
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
        Span::styled(format!(" {text}"), Style::default().fg(color)),
        Span::styled("   ", Theme::frame()),
    ];
    spans.extend(chip("nfqws", state.nfqws_installed));
    spans.extend(chip("strategies", state.strategies_installed));

    f.render_widget(
        Paragraph::new(fit_line(Line::from(spans), area.width)),
        area,
    );
}
