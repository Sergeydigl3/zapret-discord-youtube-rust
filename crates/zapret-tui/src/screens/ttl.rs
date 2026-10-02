//! The TTL submenu: leave it alone, pin a number, or sweep for one.
//!
//! The screen walks the same range the sweep walks, so a value picked by hand
//! and a value found by the sweep are comparable.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};
use zapret_wrapper::config;
use zapret_wrapper::domains::ttl;

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::Row;
use crate::menus::back;
use crate::model::Model;
use crate::msg::{Msg, Pending};
use crate::state::AppState;

const HELP: &[&str] = &["help_ttl", "help_ttl_value", "help_ttl_autopick", "help_back"];

#[derive(Debug, PartialEq, Clone)]
pub enum TtlMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct TtlMenu {
    component: MenuList,
}

impl TtlMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let value = crate::screens::extended::ttl_label(state);
        let rows = vec![
            Row::value(rust_i18n::t!("menu_ttl_dont_touch"), value.clone()),
            // Only the row that actually takes a number is wrapped in arrows, so
            // a value is never mistaken for something you can nudge here.
            Row::value(rust_i18n::t!("menu_ttl_set_value"), format!("< {value} >")),
            Row::new(rust_i18n::t!("menu_ttl_autopick")),
            back(),
        ];
        self.component
            .set(&rust_i18n::t!("tui_title_ttl"), rows, HELP, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for TtlMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Ttl(TtlMsg::Port(forward))),
            || Some(Msg::Ttl(TtlMsg::Open)),
        )
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = TtlMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Ttl, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Ttl, TtlMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Ttl, TtlMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, msg: TtlMsg) -> Option<Msg> {
    let row = menu(model)?;
    match msg {
        TtlMsg::Port(forward) => match row {
            // Only the "set a number" row has something to cycle.
            1 => cycle(model, forward),
            _ if forward => update(model, TtlMsg::Open),
            _ => Some(Msg::Redraw),
        },
        TtlMsg::Open => match row {
            0 => {
                set_ttl(&mut model.state, None);
                model.state.status_message = Some(rust_i18n::t!("ttl_set_off").into_owned());
                Some(Msg::Redraw)
            }
            // The arrows already set it; there is nothing to confirm, so the row
            // just gives the way back.
            1 => Some(Msg::Back),
            2 => {
                if model.state.strategies.is_empty() {
                    model
                        .state
                        .show_error(rust_i18n::t!("err_no_strats").into_owned());
                    None
                } else if model.state.check_dependencies() {
                    model.pending = Some(Pending::StartTtl);
                    None
                } else {
                    None
                }
            }
            _ => Some(Msg::Back),
        },
    }
}

/// Move the pinned hop count one step inside the sweep's own range.
fn cycle(model: &mut Model, forward: bool) -> Option<Msg> {
    let min = ttl::TTL_MIN as u32;
    let max = ttl::TTL_MAX as u32;
    let state = &mut model.state;
    // Starting from "off" the first step lands on the bottom of the range, and a
    // value saved before the range moved is snapped into it.
    let current = state
        .dpi_desync_ttl
        .map_or(min.saturating_sub(1), |v| (v as u32).max(min.saturating_sub(1)));
    let next = if forward {
        if current >= max { min } else { current + 1 }
    } else if current <= min {
        min
    } else {
        current - 1
    };
    set_ttl(state, Some(next as u8));
    Some(Msg::Redraw)
}

/// Write the setting through to the config, so it survives a restart.
fn set_ttl(state: &mut AppState, value: Option<u8>) {
    state.dpi_desync_ttl = value;
    let _ = config::save_ttl(value);
}

fn menu(model: &Model) -> Option<usize> {
    Some(
        model
            .app
            .get_component(&Id::Ttl)?
            .as_any()
            .downcast_ref::<TtlMenu>()?
            .row(),
    )
}
