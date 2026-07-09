use ratatui::{
    text::{Line, Span},
    widgets::ListItem,
    style::{Color, Style},
};
use crate::autotune::PRESETS;
use crate::tui::state::{AppState, AutotuneMenuState, AutotuneProtocolsState, AutotuneBlockChecksState};
use crate::tui::theme::Theme;

fn on_off(v: bool) -> &'static str {
    if v { "ON" } else { "OFF" }
}

pub fn render_config(app: &AppState) -> (Vec<ListItem<'static>>, String, usize) {
    let mut items: Vec<ListItem<'static>> = Vec::new();

    let is_sel = app.autotune_menu == AutotuneMenuState::PresetSelection;
    let preset_names: Vec<&str> = app.autotune_config.preset_indices.iter()
        .filter_map(|&i| if i < PRESETS.len() { Some(PRESETS[i].name) } else { None })
        .collect();
    let preset_label = if preset_names.is_empty() {
        rust_i18n::t!("menu_autotune_preset_none").into_owned()
    } else {
        format!("[ {} ]", preset_names.join(", "))
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!(" {}: ", rust_i18n::t!("menu_autotune_domains")),
            if is_sel { Theme::selected_item() } else { Theme::normal_item() },
        ),
        Span::styled(
            format!("< {} >", preset_label),
            if is_sel { Theme::selected_value() } else { Theme::normal_value() },
        ),
    ])));

    let is_sel = app.autotune_menu == AutotuneMenuState::NumRequests;
    items.push(ListItem::new({
        let value = if app.autotune_request_editing {
            let cursor = if (app.autotune_request_buf.len() as u64) % 2 == 0 { "_" } else { " " };
            format!("< {} >", app.autotune_request_buf.clone() + cursor)
        } else {
            format!("< {} >", app.autotune_config.num_requests)
        };
        Line::from(vec![
            Span::styled(
                format!(" {}: ", rust_i18n::t!("menu_autotune_requests")),
                if is_sel { Theme::selected_item() } else { Theme::normal_item() },
            ),
            Span::styled(
                value,
                if is_sel { Theme::selected_value() } else { Theme::normal_value() },
            ),
        ])
    }));

    let is_sel = app.autotune_menu == AutotuneMenuState::Strategies;
    let strat_count = app.autotune_config.strategy_indices.len();
    let strat_label: String = if strat_count == 0 && !app.strategies.is_empty() {
        rust_i18n::t!("menu_autotune_strat_none").into_owned()
    } else {
        format!("{} / {}", strat_count, app.strategies.len())
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!(" {}: ", rust_i18n::t!("menu_autotune_strategies")),
            if is_sel { Theme::selected_item() } else { Theme::normal_item() },
        ),
        Span::styled(
            format!("< {} >", strat_label),
            if is_sel { Theme::selected_value() } else { Theme::normal_value() },
        ),
    ])));

    let is_sel = app.autotune_menu == AutotuneMenuState::Protocols;
    let proto_status = format!(
        "HTTP:{} HTTPS:{} TLS1.2:{} TLS1.3:{} QUIC:{}",
        on_off(app.autotune_config.check_http),
        on_off(app.autotune_config.check_https),
        on_off(app.autotune_config.check_tls12),
        on_off(app.autotune_config.check_tls13),
        on_off(app.autotune_config.check_quic),
    );
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!(" {}: ", rust_i18n::t!("menu_autotune_protocols")),
            if is_sel { Theme::selected_item() } else { Theme::normal_item() },
        ),
        Span::styled(
            format!("< {} >", proto_status),
            if is_sel { Theme::selected_value() } else { Theme::normal_value() },
        ),
    ])));

    let is_sel = app.autotune_menu == AutotuneMenuState::BlockChecks;
    let enabled_count = app.autotune_config.block_checks.count_enabled();
    let bc_status = if enabled_count == 0 {
        rust_i18n::t!("val_off").into_owned()
    } else if enabled_count == 6 {
        rust_i18n::t!("val_on").into_owned()
    } else {
        format!("{}/6", enabled_count)
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!(" {}: ", rust_i18n::t!("menu_autotune_blockchecks")),
            if is_sel { Theme::selected_item() } else { Theme::normal_item() },
        ),
        Span::styled(
            format!("< {} >", bc_status),
            if is_sel { Theme::selected_value() } else { Theme::normal_value() },
        ),
    ])));

    let is_sel = app.autotune_menu == AutotuneMenuState::EditCustom;
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!(" {}: ", rust_i18n::t!("menu_autotune_edit_custom")),
            if is_sel { Theme::selected_item() } else { Theme::normal_item() },
        ),
    ])));

    let is_sel = app.autotune_menu == AutotuneMenuState::Results;
    let has_file = crate::autotune::load_results_file().is_some();
    let results_label = if has_file {
        rust_i18n::t!("menu_autotune_view")
    } else {
        rust_i18n::t!("menu_autotune_no_results")
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!(" {}: ", rust_i18n::t!("menu_autotune_results")),
            if is_sel { Theme::selected_item() } else { Theme::normal_item() },
        ),
        Span::styled(
            if has_file { format!("< {} >", results_label) } else { results_label.to_string() },
            if is_sel { Theme::selected_value() } else { Theme::normal_value() },
        ),
    ])));

    let is_sel = app.autotune_menu == AutotuneMenuState::Run;
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!(" {}", rust_i18n::t!("menu_autotune_run")),
            if is_sel { Theme::selected_item() } else { Theme::normal_item() },
        ),
    ])));

    let is_sel = app.autotune_menu == AutotuneMenuState::Back;
    items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_autotune_back")))
        .style(if is_sel { Theme::selected_item() } else { Theme::normal_item() }));

    let selected_index = if app.autotune_menu_index < items.len() { app.autotune_menu_index } else { 0 };
    (items, rust_i18n::t!("menu_autotune_title").into_owned(), selected_index)
}

pub fn render_protocols(
    app: &AppState,
    proto_menu: AutotuneProtocolsState,
) -> (Vec<ListItem<'static>>, String, usize) {
    let mut items: Vec<ListItem<'static>> = Vec::new();
    let mut selected_index = 0;

    let checks = [
        (AutotuneProtocolsState::Http, rust_i18n::t!("menu_autotune_http"), app.autotune_config.check_http),
        (AutotuneProtocolsState::Https, rust_i18n::t!("menu_autotune_https"), app.autotune_config.check_https),
        (AutotuneProtocolsState::Tls12, rust_i18n::t!("menu_autotune_tls12"), app.autotune_config.check_tls12),
        (AutotuneProtocolsState::Tls13, rust_i18n::t!("menu_autotune_tls13"), app.autotune_config.check_tls13),
        (AutotuneProtocolsState::Quic, rust_i18n::t!("menu_autotune_quic"), app.autotune_config.check_quic),
    ];

    for (idx, (state, label, enabled)) in checks.iter().enumerate() {
        let sel = *state == proto_menu;
        if sel { selected_index = idx; }
        let toggle = if *enabled {
            rust_i18n::t!("val_on")
        } else {
            rust_i18n::t!("val_off")
        };
        let toggle_style = if *enabled {
            Theme::active_value()
        } else {
            Theme::inactive_value()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!(" {}: ", label),
                if sel { Theme::selected_item() } else { Theme::normal_item() },
            ),
            Span::styled(
                format!("[ {} ]", toggle),
                if sel { Theme::selected_value() } else { toggle_style },
            ),
        ])));
    }

    let sel_back = AutotuneProtocolsState::Back == proto_menu;
    if sel_back { selected_index = 5; }
    items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_autotune_back")))
        .style(if sel_back { Theme::selected_item() } else { Theme::normal_item() }));

    (items, rust_i18n::t!("tui_title_autotune_proto").into_owned(), selected_index)
}

pub fn render_blockchecks(
    app: &AppState,
    bc_menu: AutotuneBlockChecksState,
) -> (Vec<ListItem<'static>>, String, usize) {
    use crate::autotune::BlockCheckType;
    let mut items: Vec<ListItem<'static>> = Vec::new();
    let mut selected_index = 0;

    let all_types = BlockCheckType::all();
    for (idx, ty) in all_types.iter().enumerate() {
        let state = match *ty {
            BlockCheckType::DnsSpoof => AutotuneBlockChecksState::DnsSpoof,
            BlockCheckType::TcpRst => AutotuneBlockChecksState::TcpRst,
            BlockCheckType::SniBlock => AutotuneBlockChecksState::SniBlock,
            BlockCheckType::SiberianBlock => AutotuneBlockChecksState::SiberianBlock,
            BlockCheckType::QuicBlock => AutotuneBlockChecksState::QuicBlock,
            BlockCheckType::CidrWhitelist => AutotuneBlockChecksState::CidrWhitelist,
        };
        let sel = state == bc_menu;
        if sel { selected_index = idx; }
        let enabled = app.autotune_config.block_checks.get(idx);
        let toggle = if enabled {
            rust_i18n::t!("val_on")
        } else {
            rust_i18n::t!("val_off")
        };
        let toggle_style = if enabled {
            Theme::active_value()
        } else {
            Theme::inactive_value()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!(" {}: ", ty.name()),
                if sel { Theme::selected_item() } else { Theme::normal_item() },
            ),
            Span::styled(
                format!("[ {} ]", toggle),
                if sel { Theme::selected_value() } else { toggle_style },
            ),
        ])));
    }

    let sel_back = AutotuneBlockChecksState::Back == bc_menu;
    if sel_back { selected_index = all_types.len(); }
    items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_autotune_back")))
        .style(if sel_back { Theme::selected_item() } else { Theme::normal_item() }));

    (items, rust_i18n::t!("tui_title_autotune_bc").into_owned(), selected_index)
}

pub fn render_presets(app: &AppState) -> (Vec<ListItem<'static>>, String, usize) {
    let mut items: Vec<ListItem<'static>> = Vec::new();
    let mut selected_index = 0;

    for (idx, preset) in PRESETS.iter().enumerate() {
        let sel = idx == app.autotune_preset_index;
        if sel { selected_index = idx; }
        let is_selected = app.autotune_config.preset_indices.contains(&idx);
        let toggle = if is_selected {
            rust_i18n::t!("val_on")
        } else {
            rust_i18n::t!("val_off")
        };
        let toggle_style = if is_selected {
            Theme::active_value()
        } else {
            Theme::inactive_value()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!(" {}: ", preset.name),
                if sel { Theme::selected_item() } else { Theme::normal_item() },
            ),
            Span::styled(
                format!("[ {} ]", toggle),
                if sel { Theme::selected_value() } else { toggle_style },
            ),
        ])));
    }

    let sel_back = app.autotune_preset_index >= PRESETS.len();
    if sel_back { selected_index = PRESETS.len(); }
    items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_autotune_back")))
        .style(if sel_back { Theme::selected_item() } else { Theme::normal_item() }));

    (items, rust_i18n::t!("tui_title_autotune_presets").into_owned(), selected_index)
}

pub fn render_strategies(
    app: &AppState,
    selected: usize,
) -> (Vec<ListItem<'static>>, String, usize) {
    let mut items: Vec<ListItem<'static>> = Vec::new();
    let mut selected_index = selected.min(app.strategies.len());

    for (idx, name) in app.strategies.iter().enumerate() {
        let sel = idx == selected;
        let checked = app.autotune_config.strategy_indices.contains(&idx);
        let toggle = if checked {
            rust_i18n::t!("val_on")
        } else {
            rust_i18n::t!("val_off")
        };
        let toggle_style = if checked {
            Theme::active_value()
        } else {
            Theme::inactive_value()
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(
                format!(" {}: ", name),
                if sel { Theme::selected_item() } else { Theme::normal_item() },
            ),
            Span::styled(
                format!("[ {} ]", toggle),
                if sel { Theme::selected_value() } else { toggle_style },
            ),
        ])));
    }

    let sel_back = selected >= app.strategies.len();
    if sel_back { selected_index = app.strategies.len(); }
    items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_autotune_back")))
        .style(if sel_back { Theme::selected_item() } else { Theme::normal_item() }));

    (items, rust_i18n::t!("tui_title_autotune_strat").into_owned(), selected_index)
}

pub fn render_results(_app: &AppState, scroll: usize) -> (Vec<ListItem<'static>>, String, usize) {
    let mut items: Vec<ListItem<'static>> = Vec::new();

    if let Some(cached) = crate::autotune::load_results_file() {
        for line in cached.lines() {
            items.push(ListItem::new(Line::from(Span::raw(format!(" {}", line)))));
        }
    } else {
        items.push(ListItem::new(Line::from(Span::raw(
            format!(" {}", rust_i18n::t!("autotune_no_results")),
        ))));
    }

    let back_idx = items.len();
    items.push(ListItem::new(format!(" {}", rust_i18n::t!("menu_autotune_back")))
        .style(if scroll == back_idx { Theme::selected_item() } else { Theme::normal_item() }));

    let selected = scroll.min(back_idx);
    (items, rust_i18n::t!("tui_title_autotune_results").into_owned(), selected)
}

pub fn render_header() -> (Vec<ListItem<'static>>, String, usize) {
    let items: Vec<ListItem<'static>> = vec![
        ListItem::new(Line::from(vec![
            Span::styled(
                rust_i18n::t!("autotune_running"),
                Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD),
            ),
        ])),
    ];
    (items, rust_i18n::t!("menu_autotune_title").into_owned(), 0)
}
