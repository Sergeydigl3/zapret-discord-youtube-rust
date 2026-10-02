//! The service screen: install, start, stop, restart, uninstall.
//!
//! The rows on this screen depend on what is installed and whether it is
//! running, which is why the component re-reads the service every frame rather
//! than only after an action — the list itself is what changed.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};
use zapret_wrapper::paths;
use zapret_wrapper::service;

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{Row, back};
use crate::model::Model;
use crate::msg::Msg;
use crate::state::AppState;

#[derive(Debug, PartialEq, Clone)]
pub enum ServiceMsg {
    Open,
}

/// One action the screen offers, in the order the rows are drawn.
#[derive(Debug, PartialEq, Clone, Copy)]
enum Action {
    Install,
    Start,
    Stop,
    Restart,
    Uninstall,
    Back,
}

#[derive(Component)]
pub struct ServiceMenu {
    component: MenuList,
    /// What each drawn row does. Built in the same pass as the rows so the two
    /// cannot drift, and read back through the cursor's own index.
    actions: Vec<Action>,
}

impl ServiceMenu {
    fn refresh(&mut self, state: &AppState) {
        let (rows, actions, help): (Vec<Row>, Vec<Action>, Vec<&str>) =
            if !state.service_installed {
                (
                    vec![Row::new(rust_i18n::t!("menu_srv_install")), back()],
                    vec![Action::Install, Action::Back],
                    vec!["help_srv_sel", "help_back"],
                )
            } else if state.service_active {
                (
                    vec![
                        Row::new(rust_i18n::t!("menu_srv_stop")),
                        Row::new(rust_i18n::t!("menu_srv_restart")),
                        Row::new(rust_i18n::t!("menu_srv_uninstall")),
                        back(),
                    ],
                    vec![Action::Stop, Action::Restart, Action::Uninstall, Action::Back],
                    vec!["help_srv_sel", "help_srv_sel", "help_srv_sel", "help_back"],
                )
            } else {
                (
                    vec![
                        Row::new(rust_i18n::t!("menu_srv_start")),
                        Row::new(rust_i18n::t!("menu_srv_uninstall")),
                        back(),
                    ],
                    vec![Action::Start, Action::Uninstall, Action::Back],
                    vec!["help_srv_sel", "help_srv_sel", "help_back"],
                )
            };
        self.actions = actions;
        self.component
            .set(&rust_i18n::t!("menu_srv_title"), rows, &help, None);
    }

    fn action(&self) -> Action {
        self.actions
            .get(self.component.cursor())
            .copied()
            .unwrap_or(Action::Back)
    }
}

impl AppComponent<Msg, NoUserEvent> for ServiceMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(ev, &mut self.component, |forward| {
            forward.then_some(Msg::Service(ServiceMsg::Open))
        }, || Some(Msg::Service(ServiceMsg::Open)))
    }
}

pub fn mount(app: &mut App, state: &AppState) {
    let mut menu = ServiceMenu {
        component: MenuList::new(),
        actions: Vec::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::Service, Box::new(menu), vec![]);
}

pub fn sync(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::Service, ServiceMenu) {
        menu.refresh(state);
    }
}

pub fn enter(app: &mut App, state: &mut AppState) {
    // What is installed and whether it is running is what the rows *are*, so it
    // has to be re-read before they are drawn.
    state.refresh_service_status();
    if let Some(menu) = super::mounted!(app, Id::Service, ServiceMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update(model: &mut Model, _msg: ServiceMsg) -> Option<Msg> {
    let action = model
        .app
        .get_component(&Id::Service)?
        .as_any()
        .downcast_ref::<ServiceMenu>()?
        .action();
    if action == Action::Back {
        return Some(Msg::Back);
    }

    let Some(mgr) = service::get_detected_manager() else {
        model.state.status_message = Some(rust_i18n::t!("msg_err_init").into_owned());
        return None;
    };

    let state = &mut model.state;
    let result = match action {
        Action::Install => {
            if !state.check_dependencies() {
                return None;
            }
            match std::env::current_exe() {
                Ok(exe) => mgr
                    .install(&exe, &paths::config_path(), &paths::cache_dir())
                    .and_then(|_| mgr.start()),
                Err(e) => Err(e.to_string()),
            }
        }
        Action::Start => mgr.start(),
        Action::Stop => mgr.stop(),
        Action::Restart => {
            if !state.check_dependencies() {
                return None;
            }
            mgr.restart()
        }
        Action::Uninstall => mgr.uninstall(),
        Action::Back => Ok(()),
    };

    match result {
        Ok(()) => {
            state.refresh_service_status();
            state.status_message = Some(rust_i18n::t!("msg_op_ok").into_owned());
        }
        Err(e) => {
            state.refresh_service_status();
            state.show_error(e);
        }
    }
    None
}
