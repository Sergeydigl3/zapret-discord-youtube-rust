//! The row model every menu is drawn from.
//!
//! A menu is data — a label per row, an optional value, and where the cursor is.
//! [`crate::draw`] says what a selected row looks like, so the two cannot drift.
//! Nothing here knows about the terminal.

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
    /// What the setting on this row currently reads. `None` for a row that has
    /// nothing to show — a command, or a way back.
    pub value: Option<String>,
    /// The value's colour, when the value needs to say more than "here is the
    /// current setting". `None` uses [`Theme::value`](crate::theme::Theme::value).
    pub value_style: Option<Style>,
    /// Whether this row is part of the current choice. The mark goes in the
    /// gutter so it is visible without the cursor being anywhere near the row.
    pub marked: bool,
    /// Whether this row names a group of the rows below it rather than being
    /// something the user can pick. A heading is drawn but never selected.
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
            value: Some(value.into()),
            ..Self::new(label)
        }
    }

    /// The name of the group of rows below, which cannot itself be picked.
    pub fn heading(label: impl Into<String>) -> Self {
        Self {
            heading: true,
            ..Self::new(label)
        }
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
    /// the ones".
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
    /// The row the cursor is on, clamped to the rows when the menu is drawn so a
    /// screen that shrinks under a stale index still draws a cursor.
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
