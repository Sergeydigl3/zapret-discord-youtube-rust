//! Where the drawn rows are, so the mouse can point at them.
//!
//! Rendering knows the geometry and the key handler knows what a row means, and
//! the mouse needs both at once. Rather than have [`crate::draw`] call back into
//! the event loop, every frame records where it put things here and the mouse
//! handler reads the map back. The map is written once per frame and read once
//! per click, so it cannot go stale.

use ratatui::layout::{Position, Rect};

/// How many lines one notch of the wheel moves the report.
pub const WHEEL_LINES: usize = 3;

/// The parts of the current frame the mouse can act on.
#[derive(Default)]
pub struct HitMap {
    /// A menu, and the rect of every row on screen, in the order the menu drew
    /// them.
    menu: Vec<Rect>,
    /// The list area the rows above were drawn in. A click outside it is not a
    /// click on a row, even if the coordinates match where a row used to be.
    area: Rect,
    /// Set instead of a menu on the report screen, which scrolls but has nothing
    /// to click.
    report: Option<Rect>,
}

impl HitMap {
    /// Forget the last frame. Called at the top of every draw, so a row that has
    /// scrolled off or been replaced by a different screen cannot be clicked.
    pub fn clear(&mut self) {
        self.menu.clear();
        self.area = Rect::default();
        self.report = None;
    }

    /// Record a menu that was just drawn. `rows` holds the rect of each drawn
    /// row, indexed by the row's number in the menu, and only the rows that are
    /// actually on screen.
    pub fn set_menu(&mut self, area: Rect, rows: Vec<Rect>) {
        self.menu = rows;
        self.area = area;
        self.report = None;
    }

    /// Record a report that was just drawn.
    pub fn set_report(&mut self, area: Rect) {
        self.menu.clear();
        self.report = Some(area);
    }

    /// The menu row at a point, if the point is on one.
    pub fn row_at(&self, column: u16, row: u16) -> Option<usize> {
        if self.menu.is_empty() {
            return None;
        }
        let at = Position::new(column, row);
        if !self.area.contains(at) {
            return None;
        }
        self.menu.iter().position(|rect| rect.contains(at))
    }

    /// Whether the point is over the part of the screen the wheel scrolls.
    ///
    /// Only the report scrolls a long way; a menu scrolls by moving the cursor,
    /// which the wheel already does everywhere.
    pub fn scrolls_at(&self, column: u16, row: u16) -> bool {
        self.report
            .is_some_and(|area| area.contains(Position::new(column, row)))
    }
}
