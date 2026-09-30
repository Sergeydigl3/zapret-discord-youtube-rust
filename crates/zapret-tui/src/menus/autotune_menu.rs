use ratatui::style::Style;
use zapret_wrapper::autotune::BlockCheckType;
use zapret_wrapper::domains::PRESETS;

use crate::menus::{toggles, Menu, Row};
use crate::state::screens::{AutotuneBlockChecksState, AutotuneProtocolsState};
use crate::state::AppState;
use crate::theme::Theme;

/// `ON`/`OFF` for a protocol, in the one order every transport is written in.
fn protocol_status(app: &AppState) -> String {
    let mark = |on: bool| if on { "on" } else { "off" };
    format!(
        "HTTP:{} T1.2:{} T1.3:{} QUIC:{}",
        mark(app.autotune_config.check_http),
        mark(app.autotune_config.check_tls12),
        mark(app.autotune_config.check_tls13),
        mark(app.autotune_config.check_quic)
    )
}

/// How many of the six block checks are on, and whether that is all or none of
/// them — a count alone makes "all of them" look like a subset.
fn block_check_status(app: &AppState) -> (String, Style) {
    let enabled = app.autotune_config.block_checks.count_enabled();
    let total = BlockCheckType::all().len();
    match (enabled, total) {
        (0, _) => (rust_i18n::t!("val_off").into_owned(), Theme::off()),
        (n, t) if n == t => (rust_i18n::t!("val_on").into_owned(), Theme::on()),
        (n, t) => (format!("{n}/{t}"), Theme::value()),
    }
}

pub fn render_config(app: &AppState) -> Menu {
    let preset_names: Vec<&str> = app
        .autotune_config
        .preset_indices
        .iter()
        .filter_map(|&i| PRESETS.get(i).map(|p| p.name))
        .collect();
    let presets = if preset_names.is_empty() {
        rust_i18n::t!("menu_autotune_preset_none").into_owned()
    } else {
        format!("[ {} ]", preset_names.join(", "))
    };

    let requests = if app.autotune_request_editing {
        // A blinking cursor, so it is obvious the arrows are not what is taking
        // this number now — the keyboard is.
        let caret = if (app.autotune_request_buf.len() as u64).is_multiple_of(2) {
            "_"
        } else {
            " "
        };
        format!("< {}{caret} >", app.autotune_request_buf)
    } else {
        format!("< {} >", app.autotune_config.num_requests)
    };

    let strategies = if app.autotune_config.strategy_indices.is_empty() && !app.strategies.is_empty() {
        rust_i18n::t!("menu_autotune_strat_none").into_owned()
    } else {
        format!(
            "{} / {}",
            app.autotune_config.strategy_indices.len(),
            app.strategies.len()
        )
    };

    let (block_checks, block_style) = block_check_status(app);
    let has_results = app.has_autotune_results_file;
    let results = if has_results {
        rust_i18n::t!("menu_autotune_view").to_string()
    } else {
        rust_i18n::t!("menu_autotune_no_results").to_string()
    };

    let rows = vec![
        Row::value(rust_i18n::t!("menu_autotune_domains"), format!("< {presets} >")),
        Row::value(rust_i18n::t!("menu_autotune_requests"), requests),
        Row::value(rust_i18n::t!("menu_autotune_strategies"), format!("< {strategies} >")),
        Row::value(
            rust_i18n::t!("menu_autotune_protocols"),
            format!("< {} >", protocol_status(app)),
        ),
        Row::value(
            rust_i18n::t!("menu_autotune_blockchecks"),
            format!("< {block_checks} >"),
        )
        .styled_value(block_style),
        Row::new(rust_i18n::t!("menu_autotune_edit_domains")),
        Row::value(rust_i18n::t!("menu_autotune_results"), results).styled_value(if has_results {
            Theme::value()
        } else {
            Theme::muted()
        }),
        Row::new(rust_i18n::t!("menu_autotune_run")),
        Row::new(rust_i18n::t!("menu_autotune_back")),
    ];

    Menu::new(rust_i18n::t!("menu_autotune_title"), rows).at(app.autotune_menu.index())
}

/// The one row the running sweep owns. It is not a menu, so it gets its own
/// render rather than a menu with the rows taken away.
pub fn render_header() -> Menu {
    Menu::new(
        rust_i18n::t!("menu_autotune_title"),
        vec![Row::new(rust_i18n::t!("autotune_running"))],
    )
}

pub fn render_domain_files(app: &AppState) -> Menu {
    let mut rows: Vec<Row> = app
        .domain_files
        .iter()
        .map(|(label, _)| Row::new(label.clone()))
        .collect();
    rows.push(Row::new(rust_i18n::t!("menu_autotune_back")));

    Menu::new(rust_i18n::t!("tui_title_autotune_edit_domains"), rows).at(app.domain_files_index)
}

pub fn render_protocols(app: &AppState, cursor: AutotuneProtocolsState) -> Menu {
    let mut rows = toggles([
        (
            rust_i18n::t!("menu_autotune_http").to_string(),
            app.autotune_config.check_http,
        ),
        (
            rust_i18n::t!("menu_autotune_tls12").to_string(),
            app.autotune_config.check_tls12,
        ),
        (
            rust_i18n::t!("menu_autotune_tls13").to_string(),
            app.autotune_config.check_tls13,
        ),
        (
            rust_i18n::t!("menu_autotune_quic").to_string(),
            app.autotune_config.check_quic,
        ),
    ]);
    rows.push(Row::new(rust_i18n::t!("menu_autotune_back")));

    let index = match cursor {
        AutotuneProtocolsState::Http => 0,
        AutotuneProtocolsState::Tls12 => 1,
        AutotuneProtocolsState::Tls13 => 2,
        AutotuneProtocolsState::Quic => 3,
        AutotuneProtocolsState::Back => 4,
    };

    Menu::new(rust_i18n::t!("tui_title_autotune_proto"), rows).at(index)
}

pub fn render_blockchecks(app: &AppState, cursor: AutotuneBlockChecksState) -> Menu {
    let all = BlockCheckType::all();
    let mut rows = toggles(
        all.iter()
            .enumerate()
            .map(|(i, ty)| (ty.name().to_string(), app.autotune_config.block_checks.get(i))),
    );
    rows.push(Row::new(rust_i18n::t!("menu_autotune_back")));

    let index = match cursor {
        AutotuneBlockChecksState::Back => BlockCheckType::all().len(),
        other => other.index(),
    };

    Menu::new(rust_i18n::t!("tui_title_autotune_bc"), rows).at(index)
}

pub fn render_presets(app: &AppState) -> Menu {
    let mut rows: Vec<Row> = PRESETS
        .iter()
        .enumerate()
        .map(|(i, preset)| Row::checked(preset.name, app.autotune_config.preset_indices.contains(&i)))
        .collect();
    rows.push(Row::new(rust_i18n::t!("menu_autotune_back")));

    Menu::new(rust_i18n::t!("tui_title_autotune_presets"), rows).at(app.autotune_preset_index)
}

pub fn render_strategies(app: &AppState) -> Menu {
    let mut rows: Vec<Row> = app
        .strategies
        .iter()
        .enumerate()
        .map(|(i, name)| Row::checked(name.clone(), app.autotune_config.strategy_indices.contains(&i)))
        .collect();
    rows.push(Row::new(rust_i18n::t!("menu_autotune_back")));

    Menu::new(rust_i18n::t!("tui_title_autotune_strat"), rows).at(app.autotune_strat_index)
}
