//! The application loop.
//!
//! One iteration is: ask the TUI for a configuration (unless we were given one
//! on the command line), start zapret in the foreground, wait until the user
//! stops it, then either exit or hand control back to the TUI.

use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[cfg(not(target_os = "windows"))]
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, Signal};

use zapret_wrapper::config;
use zapret_wrapper::paths;
use zapret_wrapper::run;
use zapret_wrapper::strategy;

use zapret_tui::{spawn_event_reader, AppState};

use crate::cli::Cli;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "linux")]
use zapret_core::firewall::LinuxBackend;

#[cfg(target_os = "windows")]
use zapret_core::firewall::windivert::WinDivertBackend;

use zapret_core::firewall::FirewallBackend;

/// What the run loop remembers about the firewall between passes.
///
/// On Linux this is the backend the user picked; on Windows there is nothing to
/// pick, so the type is empty and the two helpers below are what actually hold
/// the platform difference. The run loop itself never branches.
#[cfg(target_os = "linux")]
type ChosenBackend = LinuxBackend;

#[cfg(target_os = "windows")]
type ChosenBackend = ();

/// The firewall this run is put behind.
#[cfg(target_os = "linux")]
fn firewall_backend(chosen: ChosenBackend) -> Box<dyn FirewallBackend> {
    Box::new(chosen)
}

#[cfg(target_os = "windows")]
fn firewall_backend(_chosen: ChosenBackend) -> Box<dyn FirewallBackend> {
    Box::new(WinDivertBackend)
}

/// The backend as it is printed in the run parameters, where Windows has none.
#[cfg(target_os = "linux")]
fn backend_info(chosen: ChosenBackend) -> String {
    format!(", backend={}", chosen.to_config())
}

#[cfg(target_os = "windows")]
fn backend_info(_chosen: ChosenBackend) -> String {
    String::new()
}

pub fn run(args: Cli) {
    #[cfg(target_os = "linux")]
    let mut use_interface = args.interface.clone();
    let mut use_strategy = args.strategy.clone();
    let mut use_gamefilter_tcp = args.gamefiltertcp;
    let mut use_gamefilter_udp = args.gamefilterudp;
    #[cfg(target_os = "linux")]
    let mut use_backend: ChosenBackend = LinuxBackend::Nftables;
    // Windows has no choice to remember, so nothing writes to it.
    #[cfg(target_os = "windows")]
    #[allow(unused_mut)]
    let mut use_backend: ChosenBackend = ();
    let mut is_interactive = true;

    if let Some(config_file) = &args.config {
        println!("{}{}", rust_i18n::t!("msg_load_cfg"), config_file);
        match config::load_config(config_file) {
            Ok(cfg) => {
                #[cfg(target_os = "linux")]
                {
                    use_interface = cfg.interface;
                }
                use_strategy = Some(cfg.strategy);
                use_gamefilter_tcp = cfg.gamefilter_tcp;
                use_gamefilter_udp = cfg.gamefilter_udp;
                #[cfg(target_os = "linux")]
                {
                    use_backend = LinuxBackend::from_config(&cfg.backend);
                }
                is_interactive = false;
            }
            Err(e) => {
                println!("{}{}", rust_i18n::t!("err_load_cfg"), e);
                exit(1);
            }
        }
    } else if use_strategy.is_some() {
        is_interactive = false;
    }

    install_signal_handlers();

    // Single event reader for the whole process lifetime. Re-entering the TUI
    // after each "Run" must reuse this reader: spawning a new one per session
    // leaks threads that stay blocked on the console input handle and starve
    // the live reader of events.
    let reader = spawn_event_reader();

    loop {
        if is_interactive {
            let strategies = strategy::get_strategies();
            let mut app = AppState::new(strategies);

            let res = zapret_tui::run_tui(&mut app, &reader);
            if let Err(e) = res {
                println!("{}{}", rust_i18n::t!("err_tui"), e);
                exit(1);
            }

            #[cfg(target_os = "linux")]
            {
                use_interface = app.interface().to_string();
            }
            use_strategy = app.strategies.get(app.selected_strategy).cloned();
            use_gamefilter_tcp = app.tcp_gamefilter;
            use_gamefilter_udp = app.udp_gamefilter;
            #[cfg(target_os = "linux")]
            {
                use_backend = app.selected_backend;
            }

            if app.should_quit {
                println!("{}", rust_i18n::t!("msg_exited"));
                return;
            }
        }

        let strategy_file = match use_strategy {
            Some(ref s) => s.clone(),
            None => {
                println!("{}", rust_i18n::t!("msg_no_strat"));
                if is_interactive {
                    continue;
                } else {
                    exit(1);
                }
            }
        };

        let nfqws_ok = paths::nfqws_installed();
        let strat_ok = paths::strategies_installed();
        if !nfqws_ok || !strat_ok {
            if !nfqws_ok && !strat_ok {
                eprintln!("{}", rust_i18n::t!("msg_err_both_missing"));
            } else if !nfqws_ok {
                eprintln!("{}", rust_i18n::t!("msg_err_nfqws_missing"));
            } else {
                eprintln!("{}", rust_i18n::t!("msg_err_strat_missing"));
            }
            if is_interactive {
                thread::sleep(Duration::from_secs(2));
                continue;
            } else {
                exit(1);
            }
        }

        let backend = firewall_backend(use_backend);
        let backend_info = backend_info(use_backend);

        // The interface is part of the run only where there is one to choose.
        let scope = {
            #[cfg(target_os = "linux")]
            {
                format!(", interface={use_interface}")
            }
            #[cfg(not(target_os = "linux"))]
            {
                String::new()
            }
        };

        println!(
            "{}{}, {}gamefiltertcp={}, gamefilterudp={}{}",
            rust_i18n::t!("msg_run_params"),
            strategy_file,
            scope,
            use_gamefilter_tcp,
            use_gamefilter_udp,
            backend_info
        );

        let req = zapret_wrapper::plan::RunRequest::new(&strategy_file, use_gamefilter_tcp, use_gamefilter_udp);
        #[cfg(target_os = "linux")]
        let req = req.with_interface(&use_interface);
        run::run_foreground(&req, backend.as_ref());

        thread::sleep(Duration::from_millis(100));
        println!("{}", rust_i18n::t!("msg_zapret_started"));

        RUNNING.store(true, Ordering::SeqCst);

        while RUNNING.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        run::stop(backend.as_ref());

        if !is_interactive {
            break;
        }
    }
}

/// Stop waiting on `RUNNING` when the process is asked to terminate.
fn install_signal_handlers() {
    #[cfg(not(target_os = "windows"))]
    {
        extern "C" fn handle_signal(_: i32) {
            RUNNING.store(false, Ordering::SeqCst);
        }
        let handler = SigHandler::Handler(handle_signal);
        let sig_action = SigAction::new(handler, SaFlags::empty(), signal::SigSet::empty());
        unsafe {
            let _ = signal::sigaction(Signal::SIGTERM, &sig_action);
            let _ = signal::sigaction(Signal::SIGINT, &sig_action);
        }
    }
    #[cfg(target_os = "windows")]
    {
        ctrlc::set_handler(move || {
            RUNNING.store(false, Ordering::SeqCst);
        })
        .unwrap_or_else(|e| eprintln!("{}{}", rust_i18n::t!("err_ctrl_c"), e));
    }
}
