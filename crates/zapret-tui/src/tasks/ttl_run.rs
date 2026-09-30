//! The fixed-TTL autopick sweep.

use crossterm::event::Event;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::mpsc::Receiver;
use zapret_wrapper::config;
use zapret_wrapper::domains::ttl;

use crate::event::wait_for_key;
use crate::screen::{begin_external_output, end_external_output};
use crate::state::AppState;

pub fn run_ttl_autopick(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
) -> Result<(), io::Error> {
    begin_external_output(terminal)?;

    println!("{}", rust_i18n::t!("ttl_running"));
    println!();

    let strategy = app.strategies.get(app.selected_strategy).cloned().unwrap_or_default();
    let interface = app
        .interfaces
        .get(app.selected_interface)
        .map(|s| s.as_str())
        .unwrap_or("any");
    let backend = app.firewall_backend();

    let result = if strategy.is_empty() {
        Err(rust_i18n::t!("msg_no_strat").into_owned())
    } else {
        ttl::autopick_ttl(&strategy, interface, backend)
    };

    println!();
    match &result {
        Ok(ttl) => {
            let _ = config::save_ttl(Some(*ttl));
            println!("{} {}", rust_i18n::t!("ttl_found"), ttl);
        }
        Err(e) => {
            println!("{}{}", rust_i18n::t!("msg_err"), e);
        }
    }
    println!();
    println!("{}", rust_i18n::t!("msg_dl_key"));

    wait_for_key(rx)?;
    end_external_output(terminal, rx)?;

    match result {
        Ok(ttl) => {
            app.dpi_desync_ttl = Some(ttl);
            app.status_message = Some(format!("{} {}", rust_i18n::t!("ttl_found"), ttl));
        }
        Err(e) => app.show_error(e),
    }
    Ok(())
}
