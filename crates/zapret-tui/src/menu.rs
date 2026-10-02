//! A menu: a stdlib [`List`] plus everything a bare list cannot know.
//!
//! tui-realm's `List` draws rows and moves a cursor, and stops there. Three
//! things this app needs on top of that all live here, in one place, because
//! each of them used to be its own near-duplicated `match ActiveScreen`:
//!
//! - **Headings.** A row that names a group must never take the cursor, so
//!   `step` and `row_at` both skip the indices in `heading`.
//! - **The mouse.** A component is never handed its `Rect`, so `view` remembers
//!   it here; `row_at` is then arithmetic on the area rather than a hit map that
//!   a render pass has to keep in step with the keys.
//! - **The help line.** The sentence about the row under the cursor comes from
//!   `help[min(cursor, …)]`, read back through a query attribute by
//!   [`crate::model`].

use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::Frame;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::component::Component;
use tuirealm::props::{AttrValue, Attribute, BorderSides, BorderType, Borders, LineStatic, QueryResult};
use tuirealm::state::{State, StateValue};
use tui_realm_stdlib::components::List;

use crate::menus::Row;
use crate::theme::Theme;

/// The bar in the gutter on the row the cursor is on.
pub const MARKER: &str = "▌ ";

/// The attribute a component answers the help question through.
pub const HELP: Attribute = Attribute::Custom("help");

/// The attribute a component answers "do you need a warning banner" through.
///
/// The banner is drawn by the chrome, above the body, and not inside the menu's
/// own frame. A line painted *into* the frame would sit on top of the first row
/// — hiding the row, and hiding the cursor's band when the cursor is on it.
pub const WARN: Attribute = Attribute::Custom("warn");

/// One menu screen's worth of rows, and where the cursor is on them.
pub struct MenuList {
    list: List,
    rows: Vec<Row>,
    /// Which row is a heading, and so not selectable.
    heading: Vec<bool>,
    /// The area the last frame drew the menu in, borders included.
    area: Rect,
    /// A line of warning inside the top of the frame, above the first row.
    warning: Option<String>,
    cursor: usize,
    /// One locale key per row, headings included, so a screen builds it in the
    /// same loop that builds the rows rather than counting them twice.
    help: Vec<&'static str>,
}

impl Default for MenuList {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuList {
    pub fn new() -> Self {
        Self {
            list: List::default().scroll(true),
            rows: Vec::new(),
            heading: Vec::new(),
            area: Rect::default(),
            warning: None,
            cursor: 0,
            help: vec!["help_back"],
        }
    }

    /// Re-seed the menu from the state it is showing.
    ///
    /// The cursor is *not* touched: a screen the user left and came back to has
    /// to come back to the row they left it on, and only the screen's own
    /// `update` may move it.
    pub fn set(&mut self, title: &str, rows: Vec<Row>, help: &[&'static str], warning: Option<String>) {
        self.rows = rows;
        self.heading = self.rows.iter().map(|r| r.heading).collect();
        let spans = self.spans();
        self.list = List::default()
            .rows(spans)
            .scroll(true)
            .rewind(true)
            .borders(
                Borders::default()
                    .sides(BorderSides::ALL)
                    .modifiers(BorderType::Rounded)
                    .color(Theme::frame_color()),
            )
            .inactive(Theme::frame())
            .title(Line::from(Span::styled(format!(" {} ", title.trim()), Theme::title())))
            .highlight_str(LineStatic::from(Line::from(Span::styled(MARKER, Theme::accent()))))
            .highlight_style(Theme::selection());
        self.help = help.to_vec();
        self.warning = warning;
        self.clamp_cursor();
        self.list.states.list_index = self.cursor;
    }

    /// Turn the row model into what the widget draws.
    ///
    /// The gutter is the mark in the first cell and, on the selected row, the
    /// highlight symbol in the two before it. ratatui reserves that width for
    /// every row whether or not it is selected, so labels never shift sideways
    /// as the cursor moves.
    fn spans(&self) -> Vec<Line<'static>> {
        self.rows
            .iter()
            .map(|row| {
                // A heading is scenery: it keeps its own colour and never carries
                // a value, because there is nothing there to set.
                let style = if row.heading { Theme::heading() } else { Theme::label() };
                let mut spans = vec![Span::styled(if row.marked { "●" } else { " " }, Theme::accent())];
                spans.push(Span::styled(row.label.clone(), style));
                if let Some(value) = &row.value {
                    spans.push(Span::styled(
                        format!("  {value}"),
                        row.value_style.unwrap_or_else(Theme::value),
                    ));
                }
                Line::from(spans)
            })
            .collect()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Put the cursor on `index`, or as near it as the menu allows.
    pub fn at(&mut self, index: usize) {
        self.cursor = index.min(self.rows.len().saturating_sub(1));
        self.list.states.list_index = self.cursor;
    }

    /// Put the cursor on a row a pointer landed on. False when that row is
    /// scenery — a heading — so a click on one does nothing.
    pub fn select(&mut self, index: usize) -> bool {
        if index >= self.rows.len() || self.heading.get(index).copied().unwrap_or(false) {
            return false;
        }
        self.at(index);
        true
    }

    /// Move the cursor one selectable row up or down, wrapping at the ends.
    pub fn step(&mut self, forward: bool) {
        let n = self.rows.len();
        if n == 0 {
            return;
        }
        let mut i = self.cursor;
        for _ in 0..n {
            i = if forward { (i + 1) % n } else { (i + n - 1) % n };
            if !self.heading.get(i).copied().unwrap_or(false) {
                self.cursor = i;
                self.list.states.list_index = i;
                return;
            }
        }
    }

    /// The row under a point, headings included, so a click on one can be told
    /// apart from a click on a row the cursor may sit on.
    pub fn row_at(&self, x: u16, y: u16) -> Option<usize> {
        let inner = self.inner();
        let height = inner.height;
        if height == 0 || !inner.contains(ratatui::layout::Position::new(x, y)) {
            return None;
        }
        let index = self.offset(usize::from(height)) + usize::from(y - inner.y);
        (index < self.rows.len()).then_some(index)
    }

    /// The locale key for the sentence about the row under the cursor.
    pub fn help_key(&self) -> &'static str {
        let i = self.cursor.min(self.help.len().saturating_sub(1));
        self.help.get(i).copied().unwrap_or("help_back")
    }

    /// That sentence, in the user's language.
    ///
    /// The menu holds keys, not sentences: the same rows are drawn under two
    /// languages, and translating here rather than at build time is what lets the
    /// language change without a screen being rebuilt.
    pub fn help_text(&self) -> String {
        rust_i18n::t!(self.help_key()).into_owned()
    }

    /// The rows' area inside the frame.
    ///
    /// This has to be exactly the widget's own inner area and nothing else. It
    /// is what turns a click into a row number, and a list that thinks its rows
    /// start one line lower than the list actually draws them puts the selection
    /// one line above whatever the pointer is on.
    fn inner(&self) -> Rect {
        Rect {
            x: self.area.x + 1,
            y: self.area.y + 1,
            width: self.area.width.saturating_sub(2),
            height: self.area.height.saturating_sub(2),
        }
    }

    /// Which row is drawn at the top of the list.
    ///
    /// The widget is handed a fresh scroll offset every frame and asked to keep
    /// the cursor on screen, so the offset is fully determined by the cursor and
    /// the height — no guessing about what it kept from last frame.
    fn offset(&self, height: usize) -> usize {
        (self.cursor + 1).saturating_sub(height)
    }

    fn clamp_cursor(&mut self) {
        if self.rows.is_empty() {
            self.cursor = 0;
        } else if self.cursor >= self.rows.len() {
            self.cursor = self.rows.len() - 1;
        }
        // A screen that shrank under the cursor must not leave it on scenery.
        while self.heading.get(self.cursor).copied().unwrap_or(false) {
            self.cursor = (self.cursor + 1) % self.rows.len().max(1);
        }
    }
}

impl Component for MenuList {
    fn view(&mut self, f: &mut Frame, area: Rect) {
        self.area = area;
        self.list.states.list_index = self.cursor;
        self.list.view(f, area);

        let inner = self.inner();

        // A scrollbar only where there is something to scroll: an always-on rail
        // beside a six-row menu claims there is more below, and there is not.
        let rows = self.rows.len();
        let height = usize::from(inner.height);
        if rows > height && inner.height > 1 {
            let offset = self.offset(height);
            let mut scroll = ScrollbarState::new(rows)
                .position(offset)
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

    fn query<'a>(&'a self, attr: Attribute) -> Option<QueryResult<'a>> {
        if matches!(attr, HELP) {
            return Some(QueryResult::Owned(AttrValue::String(self.help_text())));
        }
        if matches!(attr, WARN) {
            return self
                .warning
                .clone()
                .map(|text| QueryResult::Owned(AttrValue::String(text)));
        }
        self.list.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.list.attr(attr, value);
    }

    fn state(&self) -> State {
        State::Single(StateValue::Usize(self.cursor))
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        match cmd {
            Cmd::Move(Direction::Down) | Cmd::Scroll(Direction::Down) => {
                self.step(true);
                CmdResult::Changed(self.state())
            }
            Cmd::Move(Direction::Up) | Cmd::Scroll(Direction::Up) => {
                self.step(false);
                CmdResult::Changed(self.state())
            }
            Cmd::GoTo(Position::Begin) => {
                self.at(0);
                CmdResult::Changed(self.state())
            }
            _ => CmdResult::NoChange,
        }
    }
}
