//! The application loop: get a configuration (from the CLI or the TUI), run
//! zapret in the foreground until the user stops it, then loop or exit.

use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

#[cfg(not(target_os = "windows"))]
use nix::sys::signal::{self, SaFlags, SigAction, SigHandler, Signal};

use zapret_tui::{spawn_event_reader, AppState};
use zapret_wrapper::config;
use zapret_wrapper::paths;
use zapret_wrapper::plan::RunRequest;
use zapret_wrapper::run;
use zapret_wrapper::strategy;

use crate::cli::Cli;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
use zapret_core::firewall::windivert::WinDivertBackend;
use zapret_core::firewall::FirewallBackend;
#[cfg(target_os = "linux")]
use zapret_core::firewall::LinuxBackend;

/// What the run loop remembers about the firewall between passes: the backend the
/// user picked on Linux, nothing to remember on Windows.
#[cfg(target_os = "linux")]
type ChosenBackend = LinuxBackend;
#[cfg(target_os = "windows")]
type ChosenBackend = ();

#[cfg(target_os = "linux")]
fn firewall_backend(chosen: ChosenBackend) -> Box<dyn FirewallBackend> {
    Box::new(chosen)
}

#[cfg(target_os = "windows")]
fn firewall_backend(_chosen: ChosenBackend) -> Box<dyn FirewallBackend> {
    Box::new(WinDivertBackend)
}

/// The backend as it appears in the run parameters, which Windows has no room for.
#[cfg(target_os = "linux")]
fn backend_info(chosen: ChosenBackend) -> String {
    format!(", backend={}", chosen.to_config())
}

#[cfg(target_os = "windows")]
fn backend_info(_chosen: ChosenBackend) -> String {
    String::new()
}

/// Stop waiting on `RUNNING` when the process is asked to terminate.
fn install_signal_handlers() {
    #[cfg(not(target_os = "windows"))]
    {
        extern "C" fn handle_signal(_: i32) {
            RUNNING.store(false, Ordering::SeqCst);
        }
        let sig_action = SigAction::new(
            SigHandler::Handler(handle_signal),
            SaFlags::empty(),
            signal::SigSet::empty(),
        );
        unsafe {
            let _ = signal::sigaction(Signal::SIGTERM, &sig_action);
            let _ = signal::sigaction(Signal::SIGINT, &sig_action);
        }
    }
    #[cfg(target_os = "windows")]
    ctrlc::set_handler(|| RUNNING.store(false, Ordering::SeqCst))
        .unwrap_or_else(|e| eprintln!("{}{}", rust_i18n::t!("err_ctrl_c"), e));
}

pub fn run(args: Cli) {
    #[cfg(target_os = "linux")]
    let mut use_interface = args.interface.clone();
    let mut use_strategy = args.strategy.clone();
    let mut use_gamefilter_tcp = args.gamefiltertcp;
    let mut use_gamefilter_udp = args.gamefilterudp;
    #[cfg(target_os = "linux")]
    let mut use_backend: ChosenBackend = LinuxBackend::Nftables;
    #[cfg(target_os = "windows")]
    let use_backend: ChosenBackend = ();
    let mut is_interactive = true;

    if let Some(config_file) = &args.config {
        println!("{}{}", rust_i18n::t!("msg_load_cfg"), config_file);
        let cfg = match config::load_config(config_file) {
            Ok(cfg) => cfg,
            Err(e) => {
                println!("{}{}", rust_i18n::t!("err_load_cfg"), e);
                exit(1);
            }
        };
        #[cfg(target_os = "linux")]
        {
            use_interface = cfg.interface;
            use_backend = LinuxBackend::from_config(&cfg.backend);
        }
        use_strategy = Some(cfg.strategy);
        use_gamefilter_tcp = cfg.gamefilter_tcp;
        use_gamefilter_udp = cfg.gamefilter_udp;
        is_interactive = false;
    } else if use_strategy.is_some() {
        is_interactive = false;
    }

    install_signal_handlers();

    // One reader for the whole process lifetime: re-entering the TUI after each
    // run must reuse it, because a second reader leaks a thread blocked on the
    // console input handle and starves the live one.
    let reader = spawn_event_reader();

    loop {
        if is_interactive {
            let mut app = AppState::new(strategy::get_strategies());
            if let Err(e) = zapret_tui::run_tui(&mut app, &reader) {
                println!("{}{}", rust_i18n::t!("err_tui"), e);
                exit(1);
            }

            #[cfg(target_os = "linux")]
            {
                use_interface = app.interface().to_string();
                use_backend = app.selected_backend;
            }
            use_strategy = app.strategies.get(app.selected_strategy).cloned();
            use_gamefilter_tcp = app.tcp_gamefilter;
            use_gamefilter_udp = app.udp_gamefilter;

            if app.should_quit {
                println!("{}", rust_i18n::t!("msg_exited"));
                return;
            }
        }

        let strategy_file = match use_strategy.clone() {
            Some(s) => s,
            None => {
                println!("{}", rust_i18n::t!("msg_no_strat"));
                if !is_interactive {
                    exit(1);
                }
                continue;
            }
        };

        let nfqws_ok = paths::nfqws_installed();
        let strat_ok = paths::strategies_installed();
        if !nfqws_ok || !strat_ok {
            let missing = match (nfqws_ok, strat_ok) {
                (false, false) => rust_i18n::t!("msg_err_both_missing"),
                (false, true) => rust_i18n::t!("msg_err_nfqws_missing"),
                _ => rust_i18n::t!("msg_err_strat_missing"),
            };
            eprintln!("{missing}");
            if !is_interactive {
                exit(1);
            }
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        let backend = firewall_backend(use_backend);

        // The interface is part of the run only where there is one to choose.
        #[cfg(target_os = "linux")]
        let scope = format!(", interface={use_interface}");
        #[cfg(not(target_os = "linux"))]
        let scope = String::new();

        println!(
            "{}{}, {}gamefiltertcp={}, gamefilterudp={}{}",
            rust_i18n::t!("msg_run_params"),
            strategy_file,
            scope,
            use_gamefilter_tcp,
            use_gamefilter_udp,
            backend_info(use_backend)
        );

        let req = RunRequest::new(&strategy_file, use_gamefilter_tcp, use_gamefilter_udp);
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
