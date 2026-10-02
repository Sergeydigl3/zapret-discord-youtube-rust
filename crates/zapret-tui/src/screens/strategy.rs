//! The strategy picker.
//!
//! Every installed strategy, then the way back. Choosing one is the point of
//! the screen, so Enter both picks and leaves; there is nothing to confirm
//! afterwards.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{Row, back};
use crate::model::Model;
use crate::msg::Msg;
use crate::state::AppState;

const HELP: &[&str] = &["help_strat_sel", "help_back"];

#[derive(Debug, PartialEq, Clone)]
pub enum StrategyMsg {
    Open,
}

#[derive(Component)]
pub struct StrategyMenu {
    component: MenuList,
}

impl StrategyMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let mut rows: Vec<Row> = state
            .strategies
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let row = Row::new(name.clone());
                if i == state.selected_strategy { row.mark() } else { row }
            })
            .collect();
        rows.push(back());
        self.component
            .set(&rust_i18n::t!("tui_title_strategy"), rows, HELP, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for StrategyMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |_| None,
            || Some(Msg::Strategy(StrategyMsg::Open)),
        )
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = StrategyMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Strategy, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Strategy, StrategyMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    // The picker opens on the strategy in use, not at the top of the list.
    let at = state.selected_strategy;
    if let Some(menu) = super::mounted!(app, Id::Strategy, StrategyMenu) {
        menu.component.at(at);
    }
}

pub fn update(model: &mut Model, _msg: StrategyMsg) -> Option<Msg> {
    let row = menu(model)?;
    let state = &mut model.state;
    if row < state.strategies.len() {
        state.selected_strategy = row;
        state.save_current_config();
        let name = state.strategies[row].clone();
        return Some(Msg::BackWith(format!("{}{}", rust_i18n::t!("msg_strat_sel"), name)));
    }
    Some(Msg::Back)
}

fn menu(model: &Model) -> Option<usize> {
    Some(
        model
            .app
            .get_component(&Id::Strategy)?
            .as_any()
            .downcast_ref::<StrategyMenu>()?
            .row(),
    )
}
