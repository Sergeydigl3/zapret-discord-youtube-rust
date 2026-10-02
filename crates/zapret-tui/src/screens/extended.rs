//! The Extended submenu: the settings that are real but not part of the
//! everyday run.
//!
//! TTL and the fake payloads go one level deeper; the router switch is the one
//! row that changes a value where it stands.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{Row, back};
#[cfg(target_os = "linux")]
use crate::menus::toggles;
use crate::model::Model;
use crate::msg::Msg;
use crate::state::screens::ActiveScreen;
use crate::state::AppState;

#[derive(Debug, PartialEq, Clone)]
pub enum ExtendedMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct ExtendedMenu {
    component: MenuList,
}

impl ExtendedMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let mut rows = vec![
            Row::value(rust_i18n::t!("menu_main_ttl"), ttl_label(state)),
            Row::new(rust_i18n::t!("menu_main_fakes")),
        ];
        #[cfg(target_os = "linux")]
        rows.extend(toggles(vec![(
            rust_i18n::t!("menu_extended_router").into_owned(),
            state.router_mode,
        )]));
        rows.push(back());

        let mut help = vec!["help_ttl", "help_fakes"];
        #[cfg(target_os = "linux")]
        help.push("help_router");
        help.push("help_back");

        self.component
            .set(&rust_i18n::t!("menu_extended_title"), rows, &help, None);
    }
}

/// A short label for the current TTL: the hop count, or "leave it alone".
pub fn ttl_label(state: &AppState) -> String {
    match state.dpi_desync_ttl {
        Some(n) => n.to_string(),
        None => rust_i18n::t!("ttl_dont_touch_short").into_owned(),
    }
}

impl AppComponent<Msg, NoUserEvent> for ExtendedMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Extended(ExtendedMsg::Port(forward))),
            || Some(Msg::Extended(ExtendedMsg::Open)),
        )
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = ExtendedMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Extended, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Extended, ExtendedMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Extended, ExtendedMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, msg: ExtendedMsg) -> Option<Msg> {
    let row = menu(model)?;
    match msg {
        ExtendedMsg::Open => open(model, row),
        // Only the router switch has a value to nudge; the other two rows are
        // doors, and a door opens on the forward arrow as it always has.
        ExtendedMsg::Port(forward) if forward => open(model, row),
        ExtendedMsg::Port(_) => Some(Msg::Redraw),
    }
}

fn open(model: &mut Model, row: usize) -> Option<Msg> {
    // Only the router row needs the model, and that row is Linux-only.
    let _ = model;
    match row {
        0 => Some(Msg::Open(ActiveScreen::TtlSubmenu)),
        1 => Some(Msg::Open(ActiveScreen::FakesSubmenu)),
        #[cfg(target_os = "linux")]
        2 => {
            let state = &mut model.state;
            state.router_mode = !state.router_mode;
            if let Err(e) = zapret_wrapper::config::save_router(state.router_mode) {
                state.show_error(e);
            }
            Some(Msg::Redraw)
        }
        _ => Some(Msg::Back),
    }
}

fn menu(model: &Model) -> Option<usize> {
    Some(
        model
            .app
            .get_component(&Id::Extended)?
            .as_any()
            .downcast_ref::<ExtendedMenu>()?
            .row(),
    )
}
