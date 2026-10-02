//! The list-file editor.
//!
//! One row per file, then the way back. Opening a file hands the terminal to
//! the user's editor, which is a [`Pending::OpenEditor`] rather than anything
//! done here — the screen only says which file.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{Row, back};
use crate::model::Model;
use crate::msg::{Msg, Pending};
use crate::state::AppState;

const HELP: &[&str] = &["help_lists", "help_back"];

#[derive(Debug, PartialEq, Clone)]
pub enum ListsMsg {
    Open,
}

#[derive(Component)]
pub struct ListsMenu {
    component: MenuList,
}

impl ListsMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let mut rows: Vec<Row> = state
            .lists_files
            .iter()
            .map(|file| {
                Row::new(
                    std::path::Path::new(file)
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy(),
                )
            })
            .collect();
        rows.push(back());
        self.component
            .set(&rust_i18n::t!("tui_title_lists"), rows, HELP, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for ListsMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(ev, &mut self.component, |forward| {
            forward.then_some(Msg::Lists(ListsMsg::Open))
        }, || Some(Msg::Lists(ListsMsg::Open)))
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = ListsMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Lists, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Lists, ListsMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::Lists, ListsMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, _msg: ListsMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::Lists)?
        .as_any()
        .downcast_ref::<ListsMenu>()?
        .row();
    match model.state.lists_files.get(row) {
        Some(file) => {
            model.pending = Some(Pending::OpenEditor(file.clone()));
            None
        }
        // Row past the last file is the way back.
        None => Some(Msg::Back),
    }
}
