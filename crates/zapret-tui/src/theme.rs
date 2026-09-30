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
}
