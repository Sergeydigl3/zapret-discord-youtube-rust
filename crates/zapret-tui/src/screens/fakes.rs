//! The fake-payload screens: pick a target, then pick a `.bin` file for it.
//!
//! The second screen opens with a header naming what is in use. That header is
//! a row like any other, marked as scenery in the same pass that draws it, so
//! the cursor starts past it and a click cannot land on it — which is what a
//! hand-maintained `FIRST_ACTION` constant used to be for.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};
use zapret_wrapper::fakes::{self, FakeTarget};

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{Row, back};
use crate::model::Model;
use crate::msg::Msg;
use crate::state::screens::{ActiveScreen, FakesSelectTarget};
use crate::state::AppState;
use crate::theme::Theme;

const HELP: &[&str] = &["help_fakes_sel", "help_fakes_sel", "help_back"];
/// The first entry is the header, which the cursor never sits on.
const HELP_SELECT: &[&str] = &["help_back", "help_fakes_select", "help_back"];

#[derive(Debug, PartialEq, Clone)]
pub enum FakesMsg {
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct FakesMenu {
    component: MenuList,
}

impl FakesMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let none = rust_i18n::t!("menu_fakes_none").into_owned();
        let rows = vec![
            Row::value(
                rust_i18n::t!("menu_fakes_discord"),
                state
                    .fakes_state
                    .discord_active
                    .as_deref()
                    .unwrap_or(&none)
                    .to_string(),
            )
            .styled_value(Theme::value()),
            Row::value(
                rust_i18n::t!("menu_fakes_game"),
                state
                    .fakes_state
                    .game_active
                    .as_deref()
                    .unwrap_or(&none)
                    .to_string(),
            )
            .styled_value(Theme::value()),
            back(),
        ];
        self.component
            .set(&rust_i18n::t!("menu_fakes_title"), rows, HELP, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for FakesMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Fakes(FakesMsg::Port(forward))),
            || Some(Msg::Fakes(FakesMsg::Open)),
        )
    }
}

#[derive(Component)]
pub struct FakesSelectMenu {
    component: MenuList,
    /// Which payload the file list is being chosen for.
    target: FakesSelectTarget,
}

impl FakesSelectMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn target(&self) -> FakesSelectTarget {
        self.target
    }

    fn refresh(&mut self, state: &AppState) {
        let none = rust_i18n::t!("menu_fakes_none").into_owned();
        let current = match self.target {
            FakesSelectTarget::DiscordUdp => state.fakes_state.discord_active.as_deref(),
            FakesSelectTarget::GameUdp => state.fakes_state.game_active.as_deref(),
        }
        .unwrap_or(&none);

        let mut rows = vec![Row::heading(format!(
            "{}: {}",
            rust_i18n::t!("menu_fakes_current"),
            current
        ))];
        rows.extend(
            state
                .fakes_state
                .available
                .iter()
                .map(|fake| Row::new(fake.filename.clone())),
        );
        rows.push(back());
        self.component
            .set(&rust_i18n::t!("menu_fakes_select_title"), rows, HELP_SELECT, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for FakesSelectMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        // The file list acts on both arrows, exactly as it always has.
        menu_event(
            ev,
            &mut self.component,
            |_forward| Some(Msg::Fakes(FakesMsg::Open)),
            || Some(Msg::Fakes(FakesMsg::Open)),
        )
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = FakesMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Fakes, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Fakes, FakesMenu) {
        menu.refresh(state);
    }
}

pub fn mount_select(app: &mut App, state: &AppState) {
    let mut menu = FakesSelectMenu {
        component: MenuList::new(),
        target: FakesSelectTarget::DiscordUdp,
    };
    menu.refresh(state);
    let _ = app.mount(Id::FakesSelect, Box::new(menu), vec![]);
}

pub fn sync_select(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::FakesSelect, FakesSelectMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    // What is in use may have changed while this screen was shut.
    state.fakes_state = fakes::load_fakes_state();    if let Some(menu) = super::mounted!(app, Id::Fakes, FakesMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn enter_select(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::FakesSelect, FakesSelectMenu) {
        // Land on the first file rather than on the header above it, and past
        // the end when there is nothing to choose.
        menu.component
            .at(usize::from(state.fakes_state.available.is_empty()));
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, msg: FakesMsg) -> Option<Msg> {
    match model.state.active_screen {
        ActiveScreen::FakesSubmenu => update_target(model, msg),
        _ => update_select(model),
    }
}

fn update_target(model: &mut Model, msg: FakesMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::Fakes)?
        .as_any()
        .downcast_ref::<FakesMenu>()?
        .row();
    match (msg, row) {
        (FakesMsg::Open, 2) | (FakesMsg::Port(true), 2) => Some(Msg::Back),
        (FakesMsg::Port(false), 2) => Some(Msg::Redraw),
        // Rows 0 and 1 are doors, so the forward arrow opens them too.
        (_, 0 | 1) => {
            let target = if row == 0 {
                FakesSelectTarget::DiscordUdp
            } else {
                FakesSelectTarget::GameUdp
            };
            if let Some(menu) = super::mounted!(&mut model.app, Id::FakesSelect, FakesSelectMenu) {
                menu.target = target;
            }
            Some(Msg::Open(ActiveScreen::FakesSelectSubmenu))
        }
        _ => None,
    }
}

fn update_select(model: &mut Model) -> Option<Msg> {
    let (row, target) = {
        let menu = model
            .app
            .get_component(&Id::FakesSelect)?
            .as_any()
            .downcast_ref::<FakesSelectMenu>()?;
        (menu.row(), menu.target())
    };
    // Row 0 is the header, so the files are rows 1..=n and the way back is n+1.
    let Some(source) = row
        .checked_sub(1)
        .and_then(|i| model.state.fakes_state.available.get(i))
        .cloned()
    else {
        return Some(Msg::Back);
    };
    let fake_target = match target {
        FakesSelectTarget::DiscordUdp => FakeTarget::DiscordUdp,
        FakesSelectTarget::GameUdp => FakeTarget::GameUdp,
    };
    match fakes::replace_active_fake(&model.state.fakes_state, &fake_target, &source) {
        Ok(()) => {
            model.state.fakes_state = fakes::load_fakes_state();
            Some(Msg::BackWith(rust_i18n::t!("msg_fakes_replaced").into_owned()))
        }
        Err(e) => {
            model.state.show_error(e);
            None
        }
    }
}
