//! The autotune report: a scrolling view, not a menu.
//!
//! The tables are drawn by [`crate::views::report`], which is unchanged — a set
//! of rules and coloured cells is not what a stdlib table can say. What this
//! component owns is the part that *is* state: which tab is open, how far it is
//! scrolled, and its own area so the wheel can tell a scroll from a click.

use ratatui::layout::{Position, Rect};
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, MouseEventKind, NoUserEvent};
use tuirealm::props::{AttrValue, Attribute, QueryResult};
use tuirealm::state::State;
use tuirealm::ratatui::Frame;
use zapret_wrapper::autotune::AutotuneResults;

use super::{App, Id};
use crate::menu::HELP;
use crate::model::Model;
use crate::msg::Msg;
use crate::state::screens::AutotuneReportTab;
use crate::state::AppState;
use crate::theme::Theme;

/// How far PageUp / PageDown move the report.
const PAGE: usize = 10;

/// How many lines one notch of the wheel moves it.
const WHEEL: usize = 3;

#[derive(Debug, PartialEq, Clone)]
pub enum ReportMsg {
    /// Move by `lines`, never above the top.
    Scroll(bool, usize),
    Tab(bool),
}

/// Its `Component` impl is written out rather than derived: the report is not a
/// wrapper around a stdlib component, it is one of its own.
pub struct ReportScreen {
    /// The results being reported on, copied in when the screen is opened. They
    /// do not change while the report is on screen, so they are copied once
    /// rather than once a frame.
    results: Option<AutotuneResults>,
    /// The area the last frame drew in, so the wheel knows what it is over.
    area: Rect,
    tab: AutotuneReportTab,
    scroll: usize,
}

impl ReportScreen {
    fn view_report(&mut self, f: &mut Frame, area: Rect) {
        self.area = area;
        let Some(results) = &self.results else {
            return;
        };
        let lines = crate::views::report::build(results, self.tab, area.width.saturating_sub(2));
        self.scroll = crate::views::report::render(f, area, results, self.tab, &mut self.scroll);

        // A scrollbar only where there is something to scroll: an always-on rail
        // beside a six-line report claims there is more below, and there is not.
        let height = usize::from(area.height.saturating_sub(2));
        if lines.len() > height && height > 0 {
            let mut scroll = ScrollbarState::new(lines.len())
                .position(self.scroll)
                .viewport_content_length(height);
            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("│"))
                    .thumb_symbol("█")
                    .style(Theme::frame()),
                area,
                &mut scroll,
            );
        }
    }
}

impl Component for ReportScreen {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        self.view_report(f, area);
    }

    fn query<'a>(&'a self, attr: Attribute) -> Option<QueryResult<'a>> {
        if matches!(attr, HELP) {
            return Some(QueryResult::Owned(AttrValue::String(
                rust_i18n::t!("help_autotune_results").into_owned(),
            )));
        }
        // Anything else — the warning banner included — is not this screen's to
        // answer, and it has nothing to say.
        None
    }

    fn attr(&mut self, _attr: Attribute, _value: AttrValue) {}

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::NoChange
    }
}

impl AppComponent<Msg, NoUserEvent> for ReportScreen {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(k) => match k.code {
                Key::Esc | Key::Char('q') => Some(Msg::Back),
                Key::Up | Key::Char('k') => Some(Msg::Report(ReportMsg::Scroll(false, 1))),
                Key::Down | Key::Char('j') => Some(Msg::Report(ReportMsg::Scroll(true, 1))),
                Key::PageUp => Some(Msg::Report(ReportMsg::Scroll(false, PAGE))),
                Key::PageDown => Some(Msg::Report(ReportMsg::Scroll(true, PAGE))),
                Key::Tab => Some(Msg::Report(ReportMsg::Tab(true))),
                Key::BackTab => Some(Msg::Report(ReportMsg::Tab(false))),
                _ => None,
            },
            Event::Mouse(m) => {
                if !self.area.contains(Position::new(m.column, m.row)) {
                    return None;
                }
                match m.kind {
                    MouseEventKind::ScrollDown => Some(Msg::Report(ReportMsg::Scroll(true, WHEEL))),
                    MouseEventKind::ScrollUp => Some(Msg::Report(ReportMsg::Scroll(false, WHEEL))),
                    // The report has no cursor to move, so a click does nothing.
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

pub fn mount(app: &mut App, _state: &AppState) {
    let screen = ReportScreen {
        results: None,
        area: Rect::default(),
        tab: AutotuneReportTab::Summary,
        scroll: 0,
    };
    let _ = app.mount(Id::Report, Box::new(screen), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(screen) = super::mounted!(app, Id::Report, ReportScreen) {
        if screen.results.is_none() {
            screen.results = state.autotune_results.clone();
        }
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    if let Some(screen) = super::mounted!(app, Id::Report, ReportScreen) {
        // A fresh report opens at the top of its first tab rather than wherever
        // the last one was scrolled to.
        screen.tab = AutotuneReportTab::Summary;
        screen.scroll = 0;
        screen.results = state.autotune_results.clone();
    }
}

pub fn update(model: &mut Model, msg: ReportMsg) -> Option<Msg> {
    let screen = model.app.get_component_mut(&Id::Report)?;
    let screen = screen.as_any_mut().downcast_mut::<ReportScreen>()?;
    match msg {
        ReportMsg::Scroll(forward, lines) => {
            screen.scroll = if forward {
                screen.scroll.saturating_add(lines)
            } else {
                screen.scroll.saturating_sub(lines)
            };
        }
        ReportMsg::Tab(forward) => {
            screen.tab = if forward { screen.tab.next() } else { screen.tab.prev() };
            // The two tabs have nothing in common vertically, so landing halfway
            // down a freshly opened one would just look broken.
            screen.scroll = 0;
        }
    }
    Some(Msg::Redraw)
}
