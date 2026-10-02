//! The game-filter toggles.
//!
//! Two switches and the way back. Both arrows toggle, because a switch has no
//! "forward" — turning it on and turning it off are the same gesture.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{back, toggles};
use crate::model::Model;
use crate::msg::Msg;
use crate::state::AppState;

const HELP: &[&str] = &["help_gf_tcp", "help_gf_udp", "help_back"];

#[derive(Debug, PartialEq, Clone)]
pub enum GamefilterMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct GamefilterMenu {
    component: MenuList,
}

impl GamefilterMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let mut rows = toggles([
            (rust_i18n::t!("menu_gf_tcp").to_string(), state.tcp_gamefilter),
            (rust_i18n::t!("menu_gf_udp").to_string(), state.udp_gamefilter),
        ]);
        rows.push(back());
        self.component
            .set(&rust_i18n::t!("menu_gf_title"), rows, HELP, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for GamefilterMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Gamefilter(GamefilterMsg::Port(forward))),
            || Some(Msg::Gamefilter(GamefilterMsg::Open)),
        )
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = GamefilterMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Gamefilter, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Gamefilter, GamefilterMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Gamefilter, GamefilterMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, msg: GamefilterMsg) -> Option<Msg> {
    let row = menu(model)?;
    match (msg, row) {
        (GamefilterMsg::Open, 0 | 1) => toggle(model, row),
        (GamefilterMsg::Port(_), 0 | 1) => toggle(model, row),
        (GamefilterMsg::Open, _) => return Some(Msg::Back),
        (GamefilterMsg::Port(_), _) => {}
    }
    Some(Msg::Redraw)
}

/// Turn one switch, or do nothing on the way-back row.
fn toggle(model: &mut Model, row: usize) {
    let state = &mut model.state;
    match row {
        0 => {
            state.tcp_gamefilter = !state.tcp_gamefilter;
            state.save_current_config();
        }
        1 => {
            state.udp_gamefilter = !state.udp_gamefilter;
            state.save_current_config();
        }
        _ => {}
    }
}

fn menu(model: &Model) -> Option<usize> {
    Some(
        model
            .app
            .get_component(&Id::Gamefilter)?
            .as_any()
            .downcast_ref::<GamefilterMenu>()?
            .row(),
    )
}
