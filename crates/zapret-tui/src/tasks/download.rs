//! The three download flows: zapret, strategies and both at once.

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::Event;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc::Receiver;
use zapret_wrapper::strategy;

use crate::screen::run_download;
use crate::state::{ActiveScreen, AppState, VersionTarget};

/// Resolve the version selector chosen in the submenu into the string
/// `install_dependencies` expects.
///
/// `nfqws` maps the recommended entry to a pinned tag, while strategies use the
/// symbolic `recommended`; the two flows deliberately do not share one mapping.
pub fn nfqws_version(app: &AppState) -> String {
    let nfqws_target_string;
    match &app.nfqws_target {
        VersionTarget::Recommended => zapret_fetch::ZAPRET_REC_VER.to_string(),
        VersionTarget::Latest => "latest".to_string(),
        VersionTarget::Tag(t) => {
            nfqws_target_string = t.clone();
            nfqws_target_string
        }
    }
}

pub fn strategies_version(app: &AppState) -> String {
    let strat_target_string;
    match &app.strat_target {
        VersionTarget::Recommended => "recommended".to_string(),
        VersionTarget::Latest => "latest".to_string(),
        VersionTarget::Tag(t) => {
            strat_target_string = t.clone();
            strat_target_string
        }
    }
}

pub fn download_zapret(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
) -> Result<(), io::Error> {
    let nfqws_ver = nfqws_version(app);

    let res = run_download(terminal, rx, || {
        zapret_fetch::install_dependencies(&zapret_wrapper::install_targets(), &nfqws_ver, "skip")
    })?;

    if let Err(e) = res {
        app.show_error(e.to_string());
    } else {
        app.status_message = Some(rust_i18n::t!("msg_dl_zapret_ok").into_owned());
        app.active_screen = ActiveScreen::DownloadZapretSubmenu;
        app.refresh_dep_status();
    }
    Ok(())
}

pub fn download_strategies(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
) -> Result<(), io::Error> {
    let strat_ver = strategies_version(app);

    let res = run_download(terminal, rx, || {
        zapret_fetch::install_dependencies(&zapret_wrapper::install_targets(), "skip", &strat_ver)
    })?;

    if let Err(e) = res {
        app.show_error(e.to_string());
    } else {
        app.status_message = Some(rust_i18n::t!("msg_dl_strat_ok").into_owned());
        app.strategies = strategy::get_strategies();
        app.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
        app.refresh_dep_status();
    }
    Ok(())
}

pub fn download_defaults(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
) -> Result<(), io::Error> {
    let res = run_download(terminal, rx, || {
        zapret_fetch::install_dependencies(
            &zapret_wrapper::install_targets(),
            zapret_fetch::ZAPRET_REC_VER,
            "recommended",
        )
    })?;

    if let Err(e) = res {
        app.show_error(e.to_string());
    } else {
        app.status_message = Some(rust_i18n::t!("msg_dl_all_ok").into_owned());
        app.strategies = strategy::get_strategies();
        app.active_screen = ActiveScreen::DownloadDepsSubmenu;
        app.refresh_dep_status();
    }
    Ok(())
}
