//! The downloader screens: the category, the two version pickers.
//!
//! Both pickers are the same four rows and the same component; which one a
//! mounted instance is depends on the screen it was mounted for, so the version
//! a cycle changes is read back from the state rather than stored twice.

use tuirealm::component::{AppComponent, Component};
use tuirealm::event::{Event, NoUserEvent};

use super::{menu_event, App, Id};
use crate::menu::MenuList;
use crate::menus::{Row, back};
use crate::model::Model;
use crate::msg::{Msg, Pending};
use crate::state::screens::{ActiveScreen, DownloadTarget, VersionTarget};
use crate::state::AppState;

const HELP: &[&str] = &["help_dl_ver", "help_dl_tag", "help_dl_start", "help_back"];

// ---------------------------------------------------------------- category --

#[derive(Debug, PartialEq, Clone)]
pub enum DownloadDepsMsg {
    Open,
}

#[derive(Component)]
pub struct DownloadDepsMenu {
    component: MenuList,
}

impl DownloadDepsMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, _state: &AppState) {
        let rows = vec![
            Row::new(rust_i18n::t!("menu_dl_zapret")),
            Row::new(rust_i18n::t!("menu_dl_strat")),
            Row::new(rust_i18n::t!("menu_dl_defaults")),
            back(),
        ];
        self.component
            .set(&rust_i18n::t!("menu_dl_title"), rows, HELP_CATEGORY, None);
    }
}

const HELP_CATEGORY: &[&str] = &["help_dl_zap", "help_dl_str", "help_dl_def", "help_back"];

impl AppComponent<Msg, NoUserEvent> for DownloadDepsMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| forward.then_some(Msg::DownloadDeps(DownloadDepsMsg::Open)),
            || Some(Msg::DownloadDeps(DownloadDepsMsg::Open)),
        )
    }
}

pub fn mount_deps(app: &mut App, state: &AppState) {
    let mut menu = DownloadDepsMenu {
        component: MenuList::new(),
    };
    menu.refresh(state);
    let _ = app.mount(Id::DownloadDeps, Box::new(menu), vec![]);
}

pub fn sync_deps(app: &mut App, state: &AppState) {
    if let Some(menu) = super::mounted!(app, Id::DownloadDeps, DownloadDepsMenu) {
        menu.refresh(state);
    }
}

pub fn enter_deps(app: &mut App, state: &mut AppState) {
    if let Some(menu) = super::mounted!(app, Id::DownloadDeps, DownloadDepsMenu) {
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update_deps(model: &mut Model, _msg: DownloadDepsMsg) -> Option<Msg> {
    let row = model
        .app
        .get_component(&Id::DownloadDeps)?
        .as_any()
        .downcast_ref::<DownloadDepsMenu>()?
        .row();
    match row {
        0 => Some(Msg::Open(ActiveScreen::DownloadZapretSubmenu)),
        1 => Some(Msg::Open(ActiveScreen::DownloadStrategiesSubmenu)),
        2 => {
            model.pending = Some(Pending::DownloadDefaults);
            None
        }
        _ => Some(Msg::Back),
    }
}

// ------------------------------------------------------------------ picker --

#[derive(Debug, PartialEq, Clone)]
pub enum DownloadMsg {
    /// Left / Right, on the version row only.
    Port(bool),
    Open,
}

#[derive(Component)]
pub struct DownloadMenu {
    component: MenuList,
    /// Which downloader this mounted instance is for.
    target: DownloadTarget,
}

impl DownloadMenu {
    fn row(&self) -> usize {
        self.component.cursor()
    }

    fn refresh(&mut self, state: &AppState) {
        let target = match self.target {
            DownloadTarget::Zapret => state.nfqws_target.clone(),
            DownloadTarget::Strategies => state.strat_target.clone(),
        };

        // The three versions are not a cycleable value on a row: the tag has to
        // be named, and a `Recommended → Latest → Recommended` loop cannot carry
        // a tag. So all three sit on the row at once and the one in use is the
        // one marked.
        let rec_ver = if self.target == DownloadTarget::Zapret {
            zapret_fetch::ZAPRET_REC_VER.to_string()
        } else {
            zapret_fetch::STRAT_REC_VER[..7].to_string()
        };
        let tag_label = match &target {
            VersionTarget::Tag(t) => rust_i18n::t!("val_tag_fmt").replace("{}", t),
            _ => rust_i18n::t!("val_tag").into_owned(),
        };
        let options = [
            (
                matches!(target, VersionTarget::Recommended),
                format!("{} ({})", rust_i18n::t!("val_rec"), rec_ver),
            ),
            (
                matches!(target, VersionTarget::Latest),
                rust_i18n::t!("val_latest").into_owned(),
            ),
            (matches!(target, VersionTarget::Tag(_)), tag_label),
        ];
        let version_value = options
            .iter()
            .map(|(current, label)| {
                if *current {
                    format!("[→ {label}]")
                } else {
                    format!("[{label}]")
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        let title = if self.target == DownloadTarget::Zapret {
            rust_i18n::t!("tui_title_download_zapret")
        } else {
            rust_i18n::t!("tui_title_download_strat")
        };
        let label = if self.target == DownloadTarget::Zapret {
            rust_i18n::t!("menu_subdl_title_zapret")
        } else {
            rust_i18n::t!("menu_subdl_title_strat")
        };

        let rows = vec![
            Row::value(label, version_value),
            Row::new(rust_i18n::t!("menu_subdl_tag")),
            Row::new(rust_i18n::t!("menu_subdl_start")),
            back(),
        ];
        self.component.set(&title, rows, HELP, None);
    }
}

impl AppComponent<Msg, NoUserEvent> for DownloadMenu {
    fn on(&mut self, ev: &Event<NoUserEvent>) -> Option<Msg> {
        menu_event(
            ev,
            &mut self.component,
            |forward| Some(Msg::Download(DownloadMsg::Port(forward))),
            || Some(Msg::Download(DownloadMsg::Open)),
        )
    }
}

pub fn mount_sub(app: &mut App, state: &AppState) {
    // Whichever of the two pickers is not on screen yet has to be mounted here,
    // because both share this component and this function.
    for (id, target) in [
        (Id::DownloadZapret, DownloadTarget::Zapret),
        (Id::DownloadStrategies, DownloadTarget::Strategies),
    ] {
        if app.mounted(&id) {
            continue;
        }
        let mut menu = DownloadMenu {
            component: MenuList::new(),
            target,
        };
        menu.refresh(state);
        let _ = app.mount(id, Box::new(menu), vec![]);
    }
}

pub fn sync_sub(app: &mut App, state: &AppState) {
    for id in [Id::DownloadZapret, Id::DownloadStrategies] {
        if let Some(menu) = app
            .get_component_mut(&id)
            .and_then(|c| c.as_any_mut().downcast_mut::<DownloadMenu>())
        {
            menu.refresh(state);
        }
    }
}

pub fn enter_sub(app: &mut App, state: &mut AppState) {
    let id = super::id_of(state.active_screen);
    if let Some(menu) = app
        .get_component_mut(&id)
        .and_then(|c| c.as_any_mut().downcast_mut::<DownloadMenu>())
    {
        // A picker opens on its version row.
        menu.component.at(0);
        menu.refresh(state);
    }
}

pub fn update_sub(model: &mut Model, msg: DownloadMsg) -> Option<Msg> {
    let id = super::id_of(model.state.active_screen);
    let menu = model
        .app
        .get_component(&id)?
        .as_any()
        .downcast_ref::<DownloadMenu>()?;
    let row = menu.row();
    let target = menu.target;

    match msg {
        // The version row always advances, on either arrow: it is a row of three
        // choices, not a pair of ends.
        DownloadMsg::Port(_) if row == 0 => {
            let version = version_of(model, target);
            let next = version.cycle(true);
            *version_slot(model, target) = next;
            Some(Msg::Redraw)
        }
        DownloadMsg::Port(forward) if forward => update_sub(model, DownloadMsg::Open),
        DownloadMsg::Port(_) => Some(Msg::Redraw),
        DownloadMsg::Open => match row {
            0 => {
                let version = version_of(model, target);
                let next = version.cycle(true);
                *version_slot(model, target) = next;
                Some(Msg::Redraw)
            }
            1 => {
                let (repo, key) = match target {
                    DownloadTarget::Zapret => ("bol-van/zapret", "msg_fetch_zapret_tags"),
                    DownloadTarget::Strategies => (
                        "Flowseal/zapret-discord-youtube",
                        "msg_fetch_strat_tags",
                    ),
                };
                let state = &mut model.state;
                state.status_message = Some(rust_i18n::t!(key).into_owned());
                match zapret_fetch::fetch_repo_tags(repo) {
                    Ok(tags) => {
                        *tags_slot(model, target) = tags;
                        Some(Msg::Open(match target {
                            DownloadTarget::Zapret => ActiveScreen::ZapretTagSelect,
                            DownloadTarget::Strategies => ActiveScreen::StrategyTagSelect,
                        }))
                    }
                    Err(e) => {
                        state.show_error(format!("{}{}", rust_i18n::t!("msg_err_fetch_tags"), e));
                        None
                    }
                }
            }
            2 => {
                model.pending = Some(match target {
                    DownloadTarget::Zapret => Pending::DownloadZapret,
                    DownloadTarget::Strategies => Pending::DownloadStrategies,
                });
                None
            }
            _ => Some(Msg::Back),
        },
    }
}

fn version_of(model: &Model, target: DownloadTarget) -> VersionTarget {
    match target {
        DownloadTarget::Zapret => model.state.nfqws_target.clone(),
        DownloadTarget::Strategies => model.state.strat_target.clone(),
    }
}

fn version_slot(model: &mut Model, target: DownloadTarget) -> &mut VersionTarget {
    match target {
        DownloadTarget::Zapret => &mut model.state.nfqws_target,
        DownloadTarget::Strategies => &mut model.state.strat_target,
    }
}

fn tags_slot(model: &mut Model, target: DownloadTarget) -> &mut Vec<String> {
    match target {
        DownloadTarget::Zapret => &mut model.state.available_nfqws_tags,
        DownloadTarget::Strategies => &mut model.state.available_strat_tags,
    }
}
