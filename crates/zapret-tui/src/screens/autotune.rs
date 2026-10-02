//! The autotune screens: the configuration menu, the request-count input, the
//! domain-file editor, the protocol and block-check toggles, the preset and
//! strategy multi-selects.
//!
//! Eight screens that share one data structure ([`AutotuneConfig`]) and one
//! running-state flag, so they share a file. The report is a screen too but has
//! nothing to configure, so it has one of its own.

use tuirealm::command::{Cmd, CmdResult};
use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, Key, NoUserEvent};
use tuirealm::props::{AttrValue, Attribute, BorderSides, BorderType, Borders, InputType, QueryResult};
use tuirealm::state::State;
use tui_realm_stdlib::components::Input;
use zapret_wrapper::autotune::BlockCheckType;
use zapret_wrapper::domains::PRESETS;
use zapret_wrapper::run::queue_in_use;

use super::{menu_event, App, Id};
use crate::menu::{HELP as HELP_ATTR, MenuList};
use crate::menus::{Row, back, toggles};
use crate::model::Model;
use crate::msg::{Msg, Pending};
use crate::state::screens::ActiveScreen;
use crate::state::AppState;
use crate::theme::Theme;

/// The rows of the autotune menu, in the order they are drawn.
const HELP: &[&str] = &[
    "help_autotune_domains",
    "help_autotune_req",
    "help_autotune_strat_sel",
    "help_autotune_proto",
    "help_autotune_blockchecks",
    "help_autotune_edit_domains",
    "help_autotune_results_sel",
    "help_autotune_run",
    "help_back",
];

/// `ON`/`OFF` for a protocol, in the one order every transport is written in.
fn protocol_status(state: &AppState) -> String {
    let mark = |on: bool| if on { "on" } else { "off" };
    let c = &state.autotune_config;
    format!(
        "HTTP:{} T1.2:{} T1.3:{} QUIC:{}",
        mark(c.check_http),
        mark(c.check_tls12),
        mark(c.check_tls13),
        mark(c.check_quic)
    )
}

/// How many of the six block checks are on, and whether that is all or none of
/// them — a count alone makes "all of them" look like a subset.
fn block_check_status(state: &AppState) -> (String, ratatui::style::Style) {
    let enabled = state.autotune_config.block_checks.count_enabled();
    let total = BlockCheckType::all().len();
    match (enabled, total) {
        (0, _) => (rust_i18n::t!("val_off").into_owned(), Theme::off()),
        (n, t) if n == t => (rust_i18n::t!("val_on").into_owned(), Theme::on()),
        (n, t) => (format!("{n}/{t}"), Theme::value()),
    }
}

// ------------------------------------------------------------------- config --

#[derive(Debug, PartialEq, Clone)]
pub enum AutotuneMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct AutotuneMenu {
    component: MenuList,
}

impl AutotuneMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        // A running sweep owns the screen: the menu it was started from has
        // nothing left to say, so it steps aside rather than lying about what is
        // about to happen.
        if state.autotune_running {
            self.component.set(
                &rust_i18n::t!("menu_autotune_title"),
                vec![Row::new(rust_i18n::t!("autotune_running"))],
                &["help_back"],
                None,
            );
            return;
        }

        let c = &state.autotune_config;
        let preset_names: Vec<&str> = c
            .preset_indices
            .iter()
            .filter_map(|&i| PRESETS.get(i).map(|p| p.name))
            .collect();
        let presets = if preset_names.is_empty() {
            rust_i18n::t!("menu_autotune_preset_none").into_owned()
        } else {
            format!("[ {} ]", preset_names.join(", "))
        };

        // An empty selection really does sweep nothing, so it is worth saying so
        // rather than showing a count that reads like "0 out of 0 is fine".
        let strategies = if c.strategy_indices.is_empty() {
            rust_i18n::t!("menu_autotune_strat_none").into_owned()
        } else {
            format!("{} / {}", c.strategy_indices.len(), state.strategies.len())
        };

        let (block_checks, block_style) = block_check_status(state);
        let has_results = state.has_autotune_results_file;
        let results = if has_results {
            rust_i18n::t!("menu_autotune_view").to_string()
        } else {
            rust_i18n::t!("menu_autotune_no_results").to_string()
        };

        let rows = vec![
            Row::value(rust_i18n::t!("menu_autotune_domains"), format!("< {presets} >")),
            Row::value(
                rust_i18n::t!("menu_autotune_requests"),
                format!("< {} >", c.num_requests),
            ),
            Row::value(
                rust_i18n::t!("menu_autotune_strategies"),
                format!("< {strategies} >"),
            ),
            Row::value(
                rust_i18n::t!("menu_autotune_protocols"),
                format!("< {} >", protocol_status(state)),
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
            back(),
        ];

        // The sweep warns before it runs, and the warning belongs inside the
        // block it is warning about rather than in a frame of its own.
        let warning = Some(rust_i18n::t!("autotune_warning_disable").into_owned());
        self.component
            .set(&rust_i18n::t!("menu_autotune_title"), rows, HELP, warning);
    }
}

impl AppComponent<Msg, NoUserEvent> for AutotuneMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Autotune(AutotuneMsg::Port(forward))),
            || Some(Msg::Autotune(AutotuneMsg::Open)),
        )
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = AutotuneMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Autotune, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Autotune, AutotuneMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Autotune, AutotuneMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, msg: AutotuneMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::Autotune)?
        .as_any()
        .downcast_ref::<AutotuneMenu>()?
        .row();
    match msg {
        AutotuneMsg::Port(forward) if forward => update(model, AutotuneMsg::Open),
        AutotuneMsg::Port(_) => Some(Msg::Redraw),
        AutotuneMsg::Open => match row {
            // Presets and the request count are values the arrows set; Enter has
            // nothing to add to either.
            0 => Some(Msg::Open(ActiveScreen::AutotunePresetSelectionSubmenu)),
            1 => Some(Msg::Open(ActiveScreen::AutotuneNumRequests)),
            2 => {
                if model.state.strategies.is_empty() {
                    model
                        .state
                        .show_error(rust_i18n::t!("err_no_strats").into_owned());
                    None
                } else {
                    Some(Msg::Open(ActiveScreen::AutotuneStrategiesSubmenu))
                }
            }
            3 => Some(Msg::Open(ActiveScreen::AutotuneProtocolsSubmenu)),
            4 => Some(Msg::Open(ActiveScreen::AutotuneBlockChecksSubmenu)),
            5 => Some(Msg::Open(ActiveScreen::AutotuneEditDomainsSubmenu)),
            6 => Some(Msg::Open(ActiveScreen::AutotuneResultsSubmenu)),
            7 => {
                if queue_in_use() {
                    model.state.status_message =
                        Some(rust_i18n::t!("autotune_err_nfqws_running").into_owned());
                    None
                } else {
                    model.state.autotune_running = true;
                    model.pending = Some(Pending::StartAutotune);
                    None
                }
            }
            _ => Some(Msg::Back),
        },
    }
}

// ------------------------------------------------------------ num requests --

#[derive(Debug, PartialEq, Clone)]
pub enum NumRequestsMsg {
    /// Enter: take what was typed.
    Submit,
}

/// The request count, typed rather than nudged: a two-digit number is faster to
/// type than to walk, and this used to be a hand-rolled buffer whose caret
/// blinked on the buffer length instead of on a clock.
///
/// The `Component` impl is written out rather than derived so the screen can
/// answer the help question too — a field with no row under it would otherwise
/// have nothing to say about itself.
pub struct NumRequests {
    component: Input,
}

impl NumRequests {
    fn value(&self) -> String {
        self.component.states.get_value()
    }
}

impl Component for NumRequests {
    fn view(&mut self, f: &mut tuirealm::ratatui::Frame, area: ratatui::layout::Rect) {
        self.component.view(f, area);
    }

    fn query<'a>(&'a self, attr: Attribute) -> Option<QueryResult<'a>> {
        if matches!(attr, HELP_ATTR) {
            return Some(QueryResult::Owned(AttrValue::String(
                rust_i18n::t!("help_autotune_req").into_owned(),
            )));
        }
        // Everything else belongs to the field. The warning banner falls through
        // with the rest, and a plain input has no banner to give.
        self.component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: AttrValue) {
        self.component.attr(attr, value);
    }

    fn state(&self) -> State {
        self.component.state()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl AppComponent<Msg, NoUserEvent> for NumRequests {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(k) => match k.code {
                Key::Esc => Some(Msg::Back),
                Key::Enter => Some(Msg::NumRequests(NumRequestsMsg::Submit)),
                // Everything else belongs to the field: digits, backspace, and
                // the arrows that move a caret inside it.
                code => {
                    use tuirealm::command::{Cmd, Direction};
                    let cmd = match code {
                        Key::Up | Key::Char('k') => Cmd::Move(Direction::Up),
                        Key::Down | Key::Char('j') => Cmd::Move(Direction::Down),
                        Key::Left | Key::Char('h') => Cmd::Move(Direction::Left),
                        Key::Right | Key::Char('l') => Cmd::Move(Direction::Right),
                        Key::Char(c) => Cmd::Type(c),
                        Key::Backspace => Cmd::Delete,
                        Key::Delete => Cmd::Delete,
                        Key::Home => Cmd::GoTo(tuirealm::command::Position::Begin),
                        Key::End => Cmd::GoTo(tuirealm::command::Position::End),
                        _ => return None,
                    };
                    let _ = self.component.perform(cmd);
                    Some(Msg::Redraw)
                }
            },
            _ => None,
        }
    }
}

pub fn mount_requests(app: &mut App, _state: &AppState) {
    let input = Input::default()
        .borders(
            Borders::default()
                .sides(BorderSides::ALL)
                .modifiers(BorderType::Rounded)
                .color(Theme::frame_color()),
        )
        .inactive(Theme::frame())
        .input_type(InputType::UnsignedInteger)
        .input_len(2)
        .title(rust_i18n::t!("menu_autotune_requests").into_owned())
        .invalid_style(Theme::bad());
    let _ = app.mount(Id::NumRequests, Box::new(NumRequests { component: input }), vec![]);
}

/// Nothing to re-read: the field holds what was typed, and the caret is where
/// the terminal left it.
pub fn sync_requests(_app: &mut App, _state: &AppState) {}

pub fn enter_requests(app: &mut App, state: &mut AppState) {
    if let Some(field) = super::mounted!(app, Id::NumRequests, NumRequests) {
        field
            .component
            .attr(Attribute::Value, AttrValue::String(state.autotune_config.num_requests.to_string()));
    }
}

pub fn update_requests(model: &mut Model, _msg: NumRequestsMsg) -> Option<Msg> {
    let value = model
        .app
        .get_component(&Id::NumRequests)?
        .as_any()
        .downcast_ref::<NumRequests>()?
        .value();
    if let Ok(n) = value.parse::<usize>() {
        model.state.autotune_config.num_requests = n.max(1);
    }
    Some(Msg::Back)
}

// ----------------------------------------------------------------- domains --

#[derive(Debug, PartialEq, Clone)]
pub enum DomainsMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct DomainsMenu {
    component: MenuList,
}

impl DomainsMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let mut rows: Vec<Row> = state.domain_files.iter().map(|(label, _)| Row::new(label.clone())).collect();
        rows.push(back());
        self.component.set(
            &rust_i18n::t!("tui_title_autotune_edit_domains"),
            rows,
            &["help_autotune_edit_domains", "help_back"],
            None,
        );
    }
}

impl AppComponent<Msg, NoUserEvent> for DomainsMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Domains(DomainsMsg::Port(forward))),
            || Some(Msg::Domains(DomainsMsg::Open)),
        )
    }
}

pub fn mount_domains(app: &mut App, state: &AppState) {
    let mut menu = DomainsMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Domains, Box::new(menu), vec![]);
}

pub fn sync_domains(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Domains, DomainsMenu) {
        menu.refresh(state);
    }
}

pub fn enter_domains(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Domains, DomainsMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update_domains(model: &mut Model, msg: DomainsMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::Domains)?
        .as_any()
        .downcast_ref::<DomainsMenu>()?
        .row();
    match msg {
        DomainsMsg::Port(forward) if forward => update_domains(model, DomainsMsg::Open),
        DomainsMsg::Port(_) => Some(Msg::Redraw),
        DomainsMsg::Open => match model.state.domain_files.get(row) {
            Some((_, path)) => {
                model.pending = Some(Pending::OpenEditor(path.clone()));
                None
            }
            None => Some(Msg::Back),
        },
    }
}

// --------------------------------------------------------------- protocols --

#[derive(Debug, PartialEq, Clone)]
pub enum ProtocolsMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct ProtocolsMenu {
    component: MenuList,
}

impl ProtocolsMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let c = &state.autotune_config;
        let mut rows = toggles([
            (rust_i18n::t!("menu_autotune_http").to_string(), c.check_http),
            (rust_i18n::t!("menu_autotune_tls12").to_string(), c.check_tls12),
            (rust_i18n::t!("menu_autotune_tls13").to_string(), c.check_tls13),
            (rust_i18n::t!("menu_autotune_quic").to_string(), c.check_quic),
        ]);
        rows.push(back());
        self.component.set(
            &rust_i18n::t!("tui_title_autotune_proto"),
            rows,
            &[
                "help_autotune_toggle",
                "help_autotune_toggle",
                "help_autotune_toggle",
                "help_autotune_toggle",
                "help_back",
            ],
            None,
        );
    }
}

impl AppComponent<Msg, NoUserEvent> for ProtocolsMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |_forward| Some(Msg::Protocols(ProtocolsMsg::Open)),
            || Some(Msg::Protocols(ProtocolsMsg::Open)),
        )
    }
}

pub fn mount_protocols(app: &mut App, state: &AppState) {
    let mut menu = ProtocolsMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Protocols, Box::new(menu), vec![]);
}

pub fn sync_protocols(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Protocols, ProtocolsMenu) {
        menu.refresh(state);
    }
}

pub fn enter_protocols(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Protocols, ProtocolsMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update_protocols(model: &mut Model, _msg: ProtocolsMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::Protocols)?
        .as_any()
        .downcast_ref::<ProtocolsMenu>()?
        .row();
    let c = &mut model.state.autotune_config;
    match row {
        0 => c.check_http = !c.check_http,
        1 => c.check_tls12 = !c.check_tls12,
        2 => c.check_tls13 = !c.check_tls13,
        3 => c.check_quic = !c.check_quic,
        _ => return Some(Msg::Back),
    }
    Some(Msg::Redraw)
}

// ------------------------------------------------------------ block checks --

#[derive(Debug, PartialEq, Clone)]
pub enum BlockChecksMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct BlockChecksMenu {
    component: MenuList,
}

impl BlockChecksMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let mut rows = toggles(
            BlockCheckType::all()
                .iter()
                .enumerate()
                .map(|(i, ty)| (ty.name().to_string(), state.autotune_config.block_checks.get(i))),
        );
        rows.push(back());
        let mut help = vec!["help_autotune_toggle"; BlockCheckType::all().len()];
        help.push("help_back");
        self.component
            .set(&rust_i18n::t!("tui_title_autotune_bc"), rows, &help, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for BlockChecksMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |_forward| Some(Msg::BlockChecks(BlockChecksMsg::Open)),
            || Some(Msg::BlockChecks(BlockChecksMsg::Open)),
        )
    }
}

pub fn mount_block_checks(app: &mut App, state: &AppState) {
    let mut menu = BlockChecksMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::BlockChecks, Box::new(menu), vec![]);
}

pub fn sync_block_checks(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::BlockChecks, BlockChecksMenu) {
        menu.refresh(state);
    }
}

pub fn enter_block_checks(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::BlockChecks, BlockChecksMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update_block_checks(model: &mut Model, _msg: BlockChecksMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::BlockChecks)?
        .as_any()
        .downcast_ref::<BlockChecksMenu>()?
        .row();
    // The checks come from one ordered list and the rows are drawn from that
    // same list, so row `n` is check `n` by construction rather than by two
    // enums that have to stay in step.
    if row < BlockCheckType::all().len() {
        model.state.toggle_block_check(row);
        Some(Msg::Redraw)
    } else {
        Some(Msg::Back)
    }
}

// ------------------------------------------------- multi-selects: presets --

#[derive(Debug, PartialEq, Clone)]
pub enum PresetsMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct PresetsMenu {
    component: MenuList,
}

impl PresetsMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let mut rows: Vec<Row> = PRESETS
            .iter()
            .enumerate()
            .map(|(i, preset)| Row::checked(preset.name, state.autotune_config.preset_indices.contains(&i)))
            .collect();
        rows.push(back());
        let mut help = vec!["help_autotune_presets"; PRESETS.len()];
        help.push("help_back");
        self.component.set(
            &rust_i18n::t!("tui_title_autotune_presets"),
            rows,
            &help,
            None,
        );
    }
}

impl AppComponent<Msg, NoUserEvent> for PresetsMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        // A multi-select acts on both arrows, exactly as it always has.
        menu_event(
            ev,
            &mut self.component,
            |_forward| Some(Msg::Presets(PresetsMsg::Open)),
            || Some(Msg::Presets(PresetsMsg::Open)),
        )
    }
}

pub fn mount_presets(app: &mut App, state: &AppState) {
    let mut menu = PresetsMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Presets, Box::new(menu), vec![]);
}

pub fn sync_presets(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Presets, PresetsMenu) {
        menu.refresh(state);
    }
}

pub fn enter_presets(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Presets, PresetsMenu) {
        // Never open past the last preset: the row below them is the way back.
        menu.component
            .at(menu.component.cursor().min(PRESETS.len().saturating_sub(1)));
        menu.refresh(state);
    }
}

pub fn update_presets(model: &mut Model, _msg: PresetsMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::Presets)?
        .as_any()
        .downcast_ref::<PresetsMenu>()?
        .row();
    if row >= PRESETS.len() {
        return Some(Msg::Back);
    }
    toggle(&mut model.state.autotune_config.preset_indices, row);
    let selected: Vec<&str> = model
        .state
        .autotune_config
        .preset_indices
        .iter()
        .filter_map(|i| PRESETS.get(*i).map(|p| p.name))
        .collect();
    let names = if selected.is_empty() {
        rust_i18n::t!("menu_autotune_preset_none").into_owned()
    } else {
        selected.join(", ")
    };
    model.state.status_message = Some(format!(
        "{}: {}",
        rust_i18n::t!("autotune_preset_sel"),
        names
    ));
    Some(Msg::Redraw)
}

// ------------------------------------------------ multi-selects: strategies --

#[derive(Debug, PartialEq, Clone)]
pub enum AutotuneStrategiesMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct AutotuneStrategiesMenu {
    component: MenuList,
}

impl AutotuneStrategiesMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let mut rows: Vec<Row> = state
            .strategies
            .iter()
            .enumerate()
            .map(|(i, name)| Row::checked(name.clone(), state.autotune_config.strategy_indices.contains(&i)))
            .collect();
        rows.push(back());
        let mut help = vec!["help_autotune_strat"; state.strategies.len()];
        help.push("help_back");
        self.component.set(
            &rust_i18n::t!("tui_title_autotune_strat"),
            rows,
            &help,
            None,
        );
    }
}

impl AppComponent<Msg, NoUserEvent> for AutotuneStrategiesMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |_forward| Some(Msg::AutotuneStrategies(AutotuneStrategiesMsg::Open)),
            || Some(Msg::AutotuneStrategies(AutotuneStrategiesMsg::Open)),
        )
    }
}

pub fn mount_strategies(app: &mut App, state: &AppState) {
    let mut menu = AutotuneStrategiesMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::AutotuneStrategies, Box::new(menu), vec![]);
}

pub fn sync_strategies(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::AutotuneStrategies, AutotuneStrategiesMenu) {
        menu.refresh(state);
    }
}

pub fn enter_strategies(app: &mut App, state: &mut AppState) {
    let _ = state;
    if let Some(menu) = super::mounted!(app, Id::AutotuneStrategies, AutotuneStrategiesMenu) {
        menu.component.at(0);
    }
}

pub fn update_strategies(model: &mut Model, _msg: AutotuneStrategiesMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::AutotuneStrategies)?
        .as_any()
        .downcast_ref::<AutotuneStrategiesMenu>()?
        .row();
    // The last row is the way out, so a cursor past the strategies is not a
    // strategy that does not exist.
    if row < model.state.strategies.len() {
        toggle(&mut model.state.autotune_config.strategy_indices, row);
        Some(Msg::Redraw)
    } else {
        Some(Msg::Back)
    }
}

/// Add or remove one index, then put the list back in a predictable order.
fn toggle(indices: &mut Vec<usize>, index: usize) {
    match indices.iter().position(|i| *i == index) {
        Some(pos) => {
            indices.remove(pos);
        }
        None => indices.push(index),
    }
    indices.sort_unstable();
    indices.dedup();
}
