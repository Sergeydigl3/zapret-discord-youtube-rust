use crate::menus::{Menu, Row};
use crate::state::screens::FakesSelectTarget;
use crate::state::AppState;
use crate::theme::Theme;
use zapret_wrapper::fakes::FakesState;

pub fn render(app: &AppState) -> Menu {
    let none_label = rust_i18n::t!("menu_fakes_none").into_owned();

    let rows = vec![
        Row::value(
            rust_i18n::t!("menu_fakes_discord"),
            app.fakes_state
                .discord_active
                .as_deref()
                .unwrap_or(&none_label)
                .to_string(),
        )
        .styled_value(Theme::value()),
        Row::value(
            rust_i18n::t!("menu_fakes_game"),
            app.fakes_state
                .game_active
                .as_deref()
                .unwrap_or(&none_label)
                .to_string(),
        )
        .styled_value(Theme::value()),
        Row::new(rust_i18n::t!("menu_fakes_back")),
    ];

    let index = app.fakes_menu.index();

    Menu::new(rust_i18n::t!("menu_fakes_title"), rows).at(index)
}

pub fn render_select(state: &FakesState, target: &FakesSelectTarget, selected_index: usize) -> Menu {
    let none_label = rust_i18n::t!("menu_fakes_none").into_owned();
    let current = match target {
        FakesSelectTarget::DiscordUdp => state.discord_active.as_deref(),
        FakesSelectTarget::GameUdp => state.game_active.as_deref(),
    }
    .unwrap_or(&none_label);

    // Row 0 is what is in use right now, not a file, so it is scenery above the
    // list rather than something the cursor can land on. The cursor starts past
    // it, on the first file.
    let mut rows =
        vec![Row::value(rust_i18n::t!("menu_fakes_current"), current.to_string()).styled_value(Theme::muted())];

    rows.extend(state.available.iter().map(|fake| Row::new(fake.filename.clone())));
    rows.push(Row::new(rust_i18n::t!("menu_fakes_back")));

    Menu::new(rust_i18n::t!("menu_fakes_select_title"), rows).at(selected_index)
}
