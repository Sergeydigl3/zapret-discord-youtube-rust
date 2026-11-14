//! zapret-rust — DPI bypass with a TUI.
//!
//! This binary is only bootstrap: pick the locale, decide whether we are the
//! Windows service or the interactive program, prepare the terminal, parse the
//! command line, and hand over to [`app::run`] or [`commands`]. Everything else
//! lives in `zapret-wrapper` (the framing), `zapret-core` (the launch kernel)
//! and `zapret-tui` (the interface).

mod app;
mod cli;
mod commands;
#[cfg(target_os = "windows")]
mod daemon;
mod locale;

use clap::Parser;

use cli::Cli;

// The macro joins this path onto CARGO_MANIFEST_DIR, so `locales` next to this
// manifest is found. It must run at the crate root: `t!` reaches the generated
// table through `crate::_rust_i18n_t`, which only exists at the root.
rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    // One language for the whole process: every crate has its own translation
    // table, so they are all set to the locale detected here.
    let locale = locale::detect();
    rust_i18n::set_locale(locale);
    zapret_tui::i18n::init(locale);

    #[cfg(target_os = "windows")]
    {
        if std::env::args().any(|arg| arg == "--service") {
            if let Err(e) = daemon::run_service() {
                eprintln!("{}{}", rust_i18n::t!("err_srv"), e);
                std::process::exit(1);
            }
            return;
        }
    }

    let args = Cli::parse();

    if let Some(ref d) = args.common.cache_dir {
        std::env::set_var("ZAPRET_CACHE_DIR", d);
    }

    if args.help {
        cli::show_help();
        return;
    }

    zapret_tui::setup_console();

    // Read-only commands (--status, --list-*, --show-config, editors) work as a
    // normal user; everything that touches the firewall, service, downloads or
    // the daemon is re-executed through pkexec/sudo as before.
    let needs_admin = args.needs_admin();
    if needs_admin {
        zapret_wrapper::platform::ensure_admin();
    }

    // Make sure the bundled custom strategies are present in the
    // `custom-strategies` folder so they can be picked from the strategy menu.
    // Read-only commands skip this: it writes to the cache directory, and
    // `--list-strategies` asks `get_strategies` to create them anyway.
    if needs_admin {
        if let Err(e) = zapret_wrapper::strategy::ensure_custom_strategies() {
            println!("{}{}", rust_i18n::t!("err_custom_strategies"), e);
        }
    }

    if args.has_non_tui_request() {
        if let Err(e) = commands::run(&args) {
            eprintln!("{}{}", rust_i18n::t!("err_cli_action"), e);
            std::process::exit(1);
        }
        return;
    }

    app::run(&args);
}
