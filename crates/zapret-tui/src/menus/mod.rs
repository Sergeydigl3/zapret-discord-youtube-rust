//! The row model every menu is drawn from.
//!
//! A menu used to be a hand-built `Vec<ListItem>` plus a hand-computed selected
//! index, which meant fourteen files each repeating the same "am I the selected
//! row, then pick my style" question, and any menu that got it subtly wrong
//! showed a cursor in the wrong place. Now a menu says *what* it contains and
//! [`crate::draw`] says what a selected row looks like, so the two cannot drift.
//!
//! A menu is data: a label per row, an optional value the row cycles through,
//! and where the cursor is. Nothing here knows about the terminal.

pub mod autotune_menu;
#[cfg(target_os = "windows")]
pub mod defender_menu;
pub mod download_menu;
pub mod download_submenu;
pub mod extended_menu;
pub mod fakes_menu;
pub mod gamefilter_menu;
pub mod lists_menu;
pub mod main_menu;
pub mod service_menu;
pub mod strategy_menu;
pub mod tag_menu;
pub mod ttl_menu;

use ratatui::style::Style;

/// One row of a menu.
pub struct Row {
    /// What this row is called.
    pub label: String,
    /// What the setting on this row currently reads, drawn to the right of the
    /// label. `None` for a row that has nothing to show — a command, or a way
    /// back.
    pub value: Option<String>,
    /// The value's colour, when the value needs to say more than "here is the
    /// current setting": an enabled toggle, for instance. `None` uses
    /// [`Theme::value`](crate::theme::Theme::value).
    pub value_style: Option<Style>,
    /// Whether this row is part of the current choice: the active strategy, a
    /// selected domain preset, a ticked protocol. The mark goes in the gutter
    /// so it is visible without the cursor being anywhere near the row.
    pub marked: bool,
    /// Whether this row names a group of the rows below it rather than being
    /// something the user can pick.
    ///
    /// A heading is drawn but never selected: a menu that highlights its own
    /// section title makes the cursor look like it is on something.
    pub heading: bool,
}

impl Row {
    /// A row with no value: a command, or a separator.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: None,
            value_style: None,
            marked: false,
            heading: false,
        }
    }

    /// A row that shows the setting it cycles through.
    pub fn value(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: Some(value.into()),
            value_style: None,
            marked: false,
            heading: false,
        }
    }

    /// The name of the group of rows below, which cannot itself be picked.
    pub fn heading(label: impl Into<String>) -> Self {
        let mut row = Self::new(label);
        row.heading = true;
        row
    }

    /// Give this row's value a colour of its own.
    pub fn styled_value(mut self, style: Style) -> Self {
        self.value_style = Some(style);
        self
    }

    /// This row is part of the current choice.
    pub fn mark(mut self) -> Self {
        self.marked = true;
        self
    }

    /// A row in a multi-select, with a checkbox showing whether it is ticked.
    ///
    /// A checkbox rather than [`Row::mark`] because the two mean different
    /// things: the mark says "this is the one", the box says "this is one of
    /// the ones", and a multi-select that uses the first reads as though
    /// picking a second option would move the first.
    pub fn checked(label: impl Into<String>, checked: bool) -> Self {
        let (tick, style) = if checked {
            ("[x]", crate::theme::Theme::on())
        } else {
            ("[ ]", crate::theme::Theme::muted())
        };
        Row::value(label, tick).styled_value(style)
    }
}

/// A whole menu: its rows, the title on its frame, and where the cursor is.
pub struct Menu {
    pub rows: Vec<Row>,
    /// Sits on the frame's top border.
    pub title: String,
    /// The row the cursor is on. Clamped to the rows when the menu is drawn, so
    /// a screen that shrinks under a stale index still draws a cursor.
    pub index: usize,
}

impl Menu {
    pub fn new(title: impl Into<String>, rows: Vec<Row>) -> Self {
        Self {
            rows,
            title: title.into(),
            index: 0,
        }
    }

    /// Put the cursor on a row, clamped to what the menu actually holds.
    pub fn at(mut self, index: usize) -> Self {
        self.index = index.min(self.rows.len().saturating_sub(1));
        self
    }

    /// The row the cursor is on, if the menu has any rows at all.
    pub fn selected(&self) -> Option<&Row> {
        self.rows.get(self.index)
    }

    /// Whether the last row is the way out of this screen.
    ///
    /// Screens that end in a "Back" row navigate with `len + 1` as their bound,
    /// which means the cursor is only ever on a real row for `0..len` and the
    /// back row is `len`. This is that one line of arithmetic.
    pub fn back_selected(&self) -> bool {
        self.index >= self.rows.len()
    }
}

/// A menu of toggles, which is three screens in three different files.
///
/// The value is bracketed rather than a bare `ON`/`OFF` so a row that is not
/// selected still reads as a control, and coloured so the state is visible
/// before the cursor is anywhere near it.
pub fn toggles(entries: impl IntoIterator<Item = (String, bool)>) -> Vec<Row> {
    entries
        .into_iter()
        .map(|(label, enabled)| {
            let (text, style) = if enabled {
                (rust_i18n::t!("val_on"), crate::theme::Theme::on())
            } else {
                (rust_i18n::t!("val_off"), crate::theme::Theme::off())
            };
            Row::value(label, format!("[ {} ]", text)).styled_value(style)
        })
        .collect()
}
