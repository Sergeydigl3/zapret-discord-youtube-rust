//! zapret-rust — DPI bypass with a TUI.
//!
//! This binary is only bootstrap: pick the locale, decide whether we are the
//! Windows service or the interactive program, prepare the terminal, parse the
//! command line, and hand over to [`app::run`]. Everything else lives in
//! `zapret-core` (domain) and `zapret-tui` (interface).

mod app;
mod cli;
#[cfg(target_os = "windows")]
mod daemon;

use clap::Parser;

use cli::Cli;

// The macro joins this path onto CARGO_MANIFEST_DIR, so `locales` next to this
// manifest is found. It must run at the crate root: `t!` reaches the generated
// table through `crate::_rust_i18n_t`, which only exists at the root.
rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    // One language for the whole process: every crate has its own translation
    // table, so they are all set to the locale detected here.
    let locale = zapret_core::i18n::detect_locale();
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

    zapret_tui::setup_console();
    zapret_core::platform::ensure_admin();

    let args = Cli::parse();

    if let Some(ref d) = args.cache_dir {
        std::env::set_var("ZAPRET_CACHE_DIR", d);
    }

    // Make sure the bundled custom strategies are present in the
    // `custom-strategies` folder so they can be picked from the strategy menu.
    if let Err(e) = zapret_core::strategy::ensure_custom_strategies() {
        println!("{}{}", rust_i18n::t!("err_custom_strategies"), e);
    }

    if args.help {
        cli::show_help();
        return;
    }

    app::run(args);
}
