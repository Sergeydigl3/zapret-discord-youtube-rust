//! The Windows Defender exclusion screen.
//!
//! The status line and the gap under it are scenery above the actions, drawn in
//! the same pass that draws the actions and marked as headings in it — so the
//! cursor starts below them and a click cannot land on one.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};
use zapret_wrapper::defender;

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{Row, back};
use crate::model::Model;
use crate::msg::Msg;
use crate::state::AppState;
use crate::theme::Theme;

const HELP: &[&str] = &["help_back", "help_back", "help_def_sel", "help_def_sel", "help_back"];

#[derive(Debug, PartialEq, Clone)]
pub enum DefenderMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct DefenderMenu {
    component: MenuList,
}

impl DefenderMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let status = match state.defender_status_cache {
            Some(true) => rust_i18n::t!("status_def_active"),
            Some(false) => rust_i18n::t!("status_def_inactive"),
            None => rust_i18n::t!("status_def_unknown"),
        };

        let rows = vec![
            Row::value(rust_i18n::t!("status_def_curr"), status).styled_value(Theme::accent()),
            Row::new(""),
            Row::new(rust_i18n::t!("menu_def_add")),
            Row::new(rust_i18n::t!("menu_def_remove")),
            back(),
        ];
        self.component
            .set(&rust_i18n::t!("menu_def_title"), rows, HELP, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for DefenderMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Defender(DefenderMsg::Port(forward))),
            || Some(Msg::Defender(DefenderMsg::Open)),
        )
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = DefenderMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Defender, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Defender, DefenderMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    state.refresh_defender_status();
    if let Some(menu) = super::mounted!(app, Id::Defender, DefenderMenu) {
        // The first two rows are scenery, so the cursor starts on the first
        // action below them.
        menu.component.at(2);
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, msg: DefenderMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::Defender)?
        .as_any()
        .downcast_ref::<DefenderMenu>()?
        .row();
    match msg {
        DefenderMsg::Port(forward) if forward => update(model, DefenderMsg::Open),
        DefenderMsg::Port(_) => Some(Msg::Redraw),
        DefenderMsg::Open => match row {
            2 => apply(model, defender::add_defender_exclusion(), "msg_def_add_ok"),
            3 => apply(model, defender::remove_defender_exclusion(), "msg_def_rm_ok"),
            _ => Some(Msg::Back),
        },
    }
}

fn apply(
    model: &mut Model,
    result: Result<(), String>,
    ok_message: &'static str,
) -> Option<Msg> {
    let state = &mut model.state;
    match result {
        Ok(()) => {
            state.status_message = Some(rust_i18n::t!(ok_message).into_owned());
            state.refresh_defender_status();
        }
        Err(e) => {
            state.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), e));
            state.refresh_defender_status();
        }
    }
    Some(Msg::Redraw)
}
