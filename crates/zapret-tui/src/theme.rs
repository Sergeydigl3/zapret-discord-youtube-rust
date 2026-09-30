use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn selected_item() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::LightYellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn selected_value() -> Style {
        Style::default()
            .fg(Color::Blue)
            .bg(Color::LightYellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn normal_item() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn normal_value() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }

    pub fn dim_item() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn active_value() -> Style {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    }

    pub fn inactive_value() -> Style {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    }

    pub fn header_style() -> Style {
        Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
    }

    pub fn block_title() -> Style {
        Style::default().fg(Color::LightGreen)
    }

    /// The colour of something the UI wants the eye to land on first.
    pub fn accent() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }

    /// The header row of a table: the one inverted band in the report.
    pub fn table_header() -> Style {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
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
}
