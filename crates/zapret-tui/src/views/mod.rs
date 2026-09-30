//! The screens that own their whole area instead of being a list of rows.
//!
//! [`crate::draw`] paints a title, a list and a help line, which is all a menu
//! needs. A running sweep and its report are not menus: one is a bar over a
//! log, the other is a set of tables, and both have to be read as a whole.

pub mod progress;
pub mod report;

use ratatui::style::{Color, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// The rounded, dim frame that every screen in this app is drawn inside.
pub(crate) fn frame(title: Option<Span<'static>>) -> Block<'static> {
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));
    if let Some(title) = title {
        block = block.title(title);
    }
    block
}

/// Cut a value down to its column, so a long step name or detail string cannot
/// push a row out of shape.
///
/// Width, not code points: a Cyrillic heading is twice as wide as its length
/// says, and a table that measures it wrong is ragged in a way that is obvious.
pub(crate) fn fit(text: &str, width: u16) -> String {
    if text.width() as u16 <= width {
        return text.to_string();
    }
    if width == 0 {
        return String::new();
    }
    // One column of the budget goes on the ellipsis, so the result always fits.
    let budget = width.saturating_sub(1) as usize;
    let mut out = String::new();
    let mut used = 0usize;
    for c in text.chars() {
        let w = c.width().unwrap_or(0);
        if used + w > budget {
            break;
        }
        out.push(c);
        used += w;
    }
    out.push('…');
    out
}
