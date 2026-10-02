//! The two tag pickers: one component, mounted once per downloader.
//!
//! Rows are the fetched tags and then the way back. Which list a mounted
//! instance shows is read from the screen it was mounted for, so the two never
//! have to be told apart in two places.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{Row, back};
use crate::model::Model;
use crate::msg::Msg;
use crate::state::screens::{TagTarget, VersionTarget};
use crate::state::AppState;

const HELP: &[&str] = &["help_tag_sel", "help_back"];

#[derive(Debug, PartialEq, Clone)]
pub enum TagMsg {
    Open,
}

#[derive(Component)]
pub struct TagMenu {
    component: MenuList,
    target: TagTarget,
}

impl TagMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let tags = self.tags(state);
        let mut rows: Vec<Row> = tags.iter().map(Row::new).collect();
        rows.push(back());
        let title = if self.target == TagTarget::Zapret {
            rust_i18n::t!("menu_tag_title_zapret")
        } else {
            rust_i18n::t!("menu_tag_title_strat")
        };
        self.component.set(&title, rows, HELP, None);
    }

    fn tags<'a>(&self, state: &'a AppState) -> &'a [String] {
        match self.target {
            TagTarget::Zapret => &state.available_nfqws_tags,
            TagTarget::Strategies => &state.available_strat_tags,
        }
    }
}

impl AppComponent<Msg, NoUserEvent> for TagMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(ev, &mut self.component, |_| None, || {
            Some(Msg::Tag(TagMsg::Open))
        })
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    for (id, target) in [
        (Id::ZapretTag, TagTarget::Zapret),
        (Id::StrategyTag, TagTarget::Strategies),
    ] {
        if app.mounted(&id) {
            continue;
        }
        let mut menu = TagMenu {
            component: MenuList::new(),
            target,
        };
        menu.refresh(state);
        let _ = app.mount(id, Box::new(menu), vec![]);
    }
}

pub fn sync(app: &mut App, state: &AppState) {
    for id in [Id::ZapretTag, Id::StrategyTag] {
        if let Some(menu) = app
            .get_component_mut(&id)
            .and_then(|c| c.as_any_mut().downcast_mut::<TagMenu>())
        {
            menu.refresh(state);
        }
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    let id = super::id_of(state.active_screen);
    if let Some(menu) = app
        .get_component_mut(&id)
        .and_then(|c| c.as_any_mut().downcast_mut::<TagMenu>())
    {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, _msg: TagMsg) -> Option<Msg> {
    let id = super::id_of(model.state.active_screen);
    let menu = model.app.get_component(&id)?.as_any().downcast_ref::<TagMenu>()?;
    let row = menu.row();
    let target = menu.target;
    let tags = match target {
        TagTarget::Zapret => &model.state.available_nfqws_tags,
        TagTarget::Strategies => &model.state.available_strat_tags,
    };
    // Row past the tags is the way back, not a tag that does not exist.
    let Some(selected) = tags.get(row).cloned() else {
        return Some(Msg::Back);
    };
    let message = match target {
        TagTarget::Zapret => {
            model.state.nfqws_target = VersionTarget::Tag(selected);
            "msg_zapret_tag_sel"
        }
        TagTarget::Strategies => {
            model.state.strat_target = VersionTarget::Tag(selected);
            "msg_strat_tag_sel"
        }
    };
    // The picker was opened from the downloader screen, so `Back` pops exactly
    // one level and lands there.
    Some(Msg::BackWith(rust_i18n::t!(message).into_owned()))
}
