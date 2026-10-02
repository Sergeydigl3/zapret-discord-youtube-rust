//! One palette for the whole UI.
//!
//! Every colour in the app comes from here, which is what keeps a menu, a table
//! and a progress bar reading as one program rather than three. Nothing outside
//! this file names a colour. Two greys carry the structure — [`Theme::frame`]
//! for the borders and scenery, [`Theme::selection`] for the band behind the
//! cursor row — and everything else is reserved for meaning.

use ratatui::style::{Color, Modifier, Style};

/// The frame: borders, rules, the hint line. Barely there on purpose.
const FRAME: Color = Color::Indexed(238);

/// The selected row's band. One step back from the frame, so the row reads as
/// raised off the background without becoming a slab of colour.
const BAND: Color = Color::Indexed(240);
pub struct Theme;

impl Theme {
    /// The band behind the selected row: background only. Ratatui patches this
    /// over the whole row, so a foreground here would flatten a row's label and
    /// its value into one colour.
    pub fn selection() -> Style {
        Style::default().bg(BAND).add_modifier(Modifier::BOLD)
    }

    /// A row's name.
    pub fn label() -> Style {
        Style::default().fg(Color::White)
    }

    /// The name of a group of rows. Dimmer than a row and heavier, so a heading
    /// reads as structure rather than as one more thing to pick.
    pub fn heading() -> Style {
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)
    }

    /// The value a row is showing — the setting this row cycles through.
    pub fn value() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }

    /// A setting the user has turned on.
    pub fn on() -> Style {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    }

    /// A setting the user has turned off.
    pub fn off() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// Something that is a fact rather than a choice: the file currently in
    /// use, a count, a line of explanation.
    pub fn muted() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// Borders, rules and every other piece of scenery.
    pub fn frame() -> Style {
        Style::default().fg(FRAME)
    }

    /// The title on a frame.
    pub fn title() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }

    /// The breadcrumb under the top border: where in the app you are.
    pub fn crumb() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    /// The one thing the eye should land on first.
    pub fn accent() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }

    /// A key name in the hint line.
    pub fn key() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }

    pub fn ok() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn bad() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn warn() -> Style {
        Style::default().fg(Color::Yellow)
    }

    /// The header row of a table: the one inverted band in the report.
    pub fn table_header() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }
}
