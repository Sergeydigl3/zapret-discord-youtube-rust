//! The main menu.
//!
//! The rows come from [`MainMenuState::GROUPS`] and the cursor walks that same
//! list, so a row cannot exist in the menu without being reachable by the
//! arrows. The headings are drawn from the same pass and marked as headings, so
//! the cursor steps over them rather than landing on them — which is the one
//! thing that used to need two mutually inverse functions in two files.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::Row;
use crate::model::Model;
use crate::msg::{Msg, Pending};
use crate::state::screens::ActiveScreen;
use crate::state::AppState;

#[cfg(target_os = "linux")]
use zapret_core::firewall::LinuxBackend;

/// What a main-menu row is.
///
/// `Interface` and `Backend` are Linux-only: nftables and iptables can bind the
/// rules to one output device and WinDivert cannot, so on Windows those rows do
/// not exist rather than exist and do nothing.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MainMenuState {
    #[cfg(target_os = "windows")]
    DefenderSettings,
    DownloadDeps,
    #[cfg(target_os = "linux")]
    Interface,
    Strategy,
    GamefilterSettings,
    #[cfg(target_os = "linux")]
    BackendSettings,
    IpsetMode,
    ListsEditor,
    Autotune,
    Extended,
    ServiceSettings,
    Run,
    Quit,
}

/// A named group of main-menu rows.
pub struct Group {
    /// The locale key of the heading.
    pub title: &'static str,
    pub rows: &'static [MainMenuState],
}

impl MainMenuState {
    /// The single source of truth: the menu is rendered from this list and the
    /// cursor walks it. Headings are drawn from it but never land the cursor.
    pub const GROUPS: &'static [Group] = &[
        Group {
            title: "menu_group_setup",
            rows: &[Self::DownloadDeps, Self::ListsEditor],
        },
        Group {
            title: "menu_group_network",
            rows: &[
                #[cfg(target_os = "linux")]
                Self::Interface,
                Self::GamefilterSettings,
                #[cfg(target_os = "linux")]
                Self::BackendSettings,
                Self::IpsetMode,
                Self::Extended,
            ],
        },
        Group {
            title: "menu_group_system",
            rows: &[
                #[cfg(target_os = "windows")]
                Self::DefenderSettings,
                Self::ServiceSettings,
            ],
        },
        Group {
            title: "menu_group_strategy",
            rows: &[Self::Strategy, Self::Autotune],
        },
        // No heading: a heading above "Run" would name a group of one decision.
        Group {
            title: "",
            rows: &[Self::Run, Self::Quit],
        },
    ];
}

/// What the main menu can ask for.
#[derive(Debug, PartialEq, Clone)]
pub enum MainMsg {
    /// Left / Right: change the setting on this row.
    Port(bool),
    /// Enter: do what this row is.
    Open,
}

#[derive(Component)]
pub struct MainMenu {
    component: MenuList,
    /// What each drawn row does, or `None` for a heading. Built in the same pass
    /// as the rows so the two cannot drift.
    actions: Vec<Option<MainMenuState>>,
}

impl MainMenu {
    /// The action under the cursor, or `None` if the cursor is on a heading.
    pub fn action(&self) -> Option<MainMenuState> {
        self.actions.get(self.component.cursor()).copied().flatten()
    }

    pub fn refresh(&mut self, state: &AppState) {
        let mut rows = Vec::new();
        let mut actions = Vec::new();
        let mut help = Vec::new();
        for group in MainMenuState::GROUPS {
            if !group.title.is_empty() {
                rows.push(Row::heading(rust_i18n::t!(group.title).into_owned()));
                actions.push(None);
                // Never asked: the cursor steps over headings and `select`
                // refuses them, so a click cannot land here either.
                help.push("help_quit");
            }
            for action in group.rows {
                let label = rust_i18n::t!(label_key(*action)).into_owned();
                rows.push(match value_of(state, *action) {
                    Some(value) => Row::value(label, format!("< {value} >")),
                    None => Row::new(label),
                });
                actions.push(Some(*action));
                help.push(help_key(*action));
            }
        }
        self.actions = actions;
        self.component
            .set(&rust_i18n::t!("menu_main_title"), rows, &help, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for MainMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Main(MainMsg::Port(forward))),
            || Some(Msg::Main(MainMsg::Open)),
        )
    }
}

/// The locale key of a row's name.
fn label_key(state: MainMenuState) -> &'static str {
    match state {
        #[cfg(target_os = "windows")]
        MainMenuState::DefenderSettings => "menu_main_defender",
        MainMenuState::DownloadDeps => "menu_main_downloader",
        #[cfg(target_os = "linux")]
        MainMenuState::Interface => "menu_main_interface",
        MainMenuState::Strategy => "menu_main_strategy",
        MainMenuState::GamefilterSettings => "menu_main_gamefilter",
        #[cfg(target_os = "linux")]
        MainMenuState::BackendSettings => "menu_main_backend",
        MainMenuState::IpsetMode => "menu_main_ipset",
        MainMenuState::ListsEditor => "menu_main_lists",
        MainMenuState::Autotune => "menu_main_autotune",
        MainMenuState::Extended => "menu_main_extended",
        MainMenuState::ServiceSettings => "menu_main_service",
        MainMenuState::Run => "menu_main_run",
        MainMenuState::Quit => "menu_main_quit",
    }
}

fn help_key(state: MainMenuState) -> &'static str {
    match state {
        #[cfg(target_os = "windows")]
        MainMenuState::DefenderSettings => "help_def",
        MainMenuState::DownloadDeps => "help_dl",
        #[cfg(target_os = "linux")]
        MainMenuState::Interface => "help_iface",
        MainMenuState::Strategy => "help_strat",
        MainMenuState::GamefilterSettings => "help_gf",
        #[cfg(target_os = "linux")]
        MainMenuState::BackendSettings => "help_backend",
        MainMenuState::IpsetMode => "help_ipset",
        MainMenuState::ListsEditor => "help_lists",
        MainMenuState::Autotune => "help_autotune",
        MainMenuState::Extended => "help_extended",
        MainMenuState::ServiceSettings => "help_srv",
        MainMenuState::Run => "help_run",
        MainMenuState::Quit => "help_quit",
    }
}

/// What a row is showing, or `None` for a row that only does something.
fn value_of(state: &AppState, action: MainMenuState) -> Option<String> {
    match action {
        #[cfg(target_os = "windows")]
        MainMenuState::DefenderSettings => None,
        MainMenuState::DownloadDeps => None,
        #[cfg(target_os = "linux")]
        MainMenuState::Interface => Some(state.interface().to_string()),
        MainMenuState::Strategy => Some(
            state
                .strategies
                .get(state.selected_strategy)
                .cloned()
                .unwrap_or_default(),
        ),
        MainMenuState::GamefilterSettings => {
            let parts: Vec<&str> = [
                state.tcp_gamefilter.then_some("TCP"),
                state.udp_gamefilter.then_some("UDP"),
            ]
            .into_iter()
            .flatten()
            .collect();
            Some(if parts.is_empty() {
                rust_i18n::t!("val_off").into_owned()
            } else {
                parts.join("+")
            })
        }
        #[cfg(target_os = "linux")]
        MainMenuState::BackendSettings => Some(state.selected_backend.to_string()),
        MainMenuState::IpsetMode => Some(
            state
                .available_ipset_modes
                .get(state.selected_ipset_mode)
                .map(|m| m.to_string())
                .unwrap_or_else(|| rust_i18n::t!("val_none").into_owned()),
        ),
        MainMenuState::ListsEditor
        | MainMenuState::Autotune
        | MainMenuState::Extended
        | MainMenuState::ServiceSettings
        | MainMenuState::Run
        | MainMenuState::Quit => None,
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = MainMenu {
        component: MenuList::new(),
        actions: Vec::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Main, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Main, MainMenu) {
        menu.refresh(state);
    }
}

/// The main menu has nowhere to go from, so there is nothing to prepare.
pub fn enter(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Main, MainMenu) {
        menu.refresh(state);
    }
}

/// Apply what the main menu asked for.
pub fn update(model: &mut Model, msg: MainMsg) -> Option<Msg> {
    let action = menu(model)?;

    match msg {
        MainMsg::Port(forward) => match action {
            #[cfg(target_os = "linux")]
            MainMenuState::Interface => {
                cycle_interface(&mut model.state, forward);
                Some(Msg::Redraw)
            }
            #[cfg(target_os = "linux")]
            MainMenuState::BackendSettings => {
                cycle_backend(&mut model.state, forward);
                Some(Msg::Redraw)
            }
            MainMenuState::IpsetMode => {
                cycle_ipset(&mut model.state, forward);
                Some(Msg::Redraw)
            }
            // A row with nothing to cycle opens on the forward arrow, which is
            // what a single Right has always done.
            _ if forward => update(model, MainMsg::Open),
            _ => Some(Msg::Redraw),
        },
        MainMsg::Open => {
            let state = &mut model.state;
            // Running and quitting are the only two rows the session ends on, so
            // they go through `Pending` rather than through a message of their
            // own.
            let mut leave = None;
            let next = match action {
                MainMenuState::DownloadDeps => Some(Msg::Open(ActiveScreen::DownloadDepsSubmenu)),
                MainMenuState::Strategy => Some(Msg::Open(ActiveScreen::StrategySubmenu)),
                MainMenuState::GamefilterSettings => Some(Msg::Open(ActiveScreen::GamefilterSubmenu)),
                MainMenuState::IpsetMode => {
                    cycle_ipset(state, true);
                    Some(Msg::Redraw)
                }
                MainMenuState::Extended => Some(Msg::Open(ActiveScreen::ExtendedSubmenu)),
                MainMenuState::ServiceSettings => Some(Msg::Open(ActiveScreen::ServiceSubmenu)),
                MainMenuState::ListsEditor => {
                    if zapret_wrapper::paths::strategies_installed() {
                        state.lists_files = zapret_wrapper::lists::get_lists_files();
                        Some(Msg::Open(ActiveScreen::ListsEditorSubmenu))
                    } else {
                        state.show_error(rust_i18n::t!("err_no_strats").into_owned());
                        None
                    }
                }
                MainMenuState::Autotune => Some(Msg::Open(ActiveScreen::AutotuneSubmenu)),
                #[cfg(target_os = "linux")]
                MainMenuState::Interface => {
                    cycle_interface(state, true);
                    Some(Msg::Redraw)
                }
                #[cfg(target_os = "linux")]
                MainMenuState::BackendSettings => {
                    cycle_backend(state, true);
                    Some(Msg::Redraw)
                }
                #[cfg(target_os = "windows")]
                MainMenuState::DefenderSettings => Some(Msg::Open(ActiveScreen::DefenderSubmenu)),
                MainMenuState::Run => {
                    if !state.check_dependencies() {
                        None
                    } else {
                        leave = Some(Pending::Run);
                        None
                    }
                }
                MainMenuState::Quit => {
                    leave = Some(Pending::Quit);
                    None
                }
            };
            if let Some(pending) = leave {
                model.pending = Some(pending);
            }
            next
        }
    }
}

/// The action the main menu's cursor is on.
fn menu(model: &Model) -> Option<MainMenuState> {
    model
        .app
        .get_component(&Id::Main)?
        .as_any()
        .downcast_ref::<MainMenu>()?
        .action()
}

#[cfg(target_os = "linux")]
fn cycle_interface(state: &mut AppState, forward: bool) {
    let len = state.interfaces.len();
    if len == 0 {
        return;
    }
    state.selected_interface = if forward {
        (state.selected_interface + 1) % len
    } else {
        (state.selected_interface + len - 1) % len
    };
    state.save_current_config();
}

#[cfg(target_os = "linux")]
fn cycle_backend(state: &mut AppState, forward: bool) {
    let backends = LinuxBackend::variants();
    if backends.is_empty() {
        return;
    }
    let len = backends.len();
    let at = backends
        .iter()
        .position(|b| *b == state.selected_backend)
        .unwrap_or(0);
    state.selected_backend = if forward {
        backends[(at + 1) % len]
    } else {
        backends[(at + len - 1) % len]
    };
    state.save_current_config();
}

fn cycle_ipset(state: &mut AppState, forward: bool) {
    use zapret_wrapper::lists;
    let len = state.available_ipset_modes.len();
    if len == 0 {
        return;
    }
    let old_mode = state.available_ipset_modes[state.selected_ipset_mode];
    state.selected_ipset_mode = if forward {
        (state.selected_ipset_mode + 1) % len
    } else {
        (state.selected_ipset_mode + len - 1) % len
    };
    let new_mode = state.available_ipset_modes[state.selected_ipset_mode];
    lists::apply_ipset_mode(old_mode, new_mode);
    state.available_ipset_modes = lists::get_available_modes();
    state.selected_ipset_mode = state
        .available_ipset_modes
        .iter()
        .position(|m| m == &new_mode)
        .unwrap_or(0);
}
