mod config;
mod download;
mod firewalls;
pub mod inits;
mod platform;
// Removed i18n module, using rust_i18n directly
mod runner;
mod strategy;
mod tui;
mod utils;
mod ipset;

rust_i18n::i18n!("locales", fallback = "en");

use clap::Parser;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
#[cfg(not(target_os = "windows"))]
use nix::sys::signal::{self, SigAction, SigHandler, Signal, SaFlags};

static RUNNING: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "linux")]
use firewalls::LinuxBackend;

#[cfg(target_os = "windows")]
use firewalls::windivert::WinDivertBackend;

#[cfg(target_os = "windows")]
pub mod defender;

#[derive(Parser, Debug)]
#[command(
    name = "run",
    about = "Run zapret in foreground (useful for testing).",
    disable_help_flag = true
)]
struct Cli {
    #[arg(short = 'c', long = "config", help = "Load configuration from file")]
    config: Option<String>,

    #[arg(short = 's', long = "strategy", help = "Use specific strategy")]
    strategy: Option<String>,

    #[arg(
        short = 'i',
        long = "interface",
        default_value = "any",
        help = "Network interface (default: any)"
    )]
    interface: String,

    #[arg(long = "gamefiltertcp", short = 't', help = "Enable gamefiltertcp")]
    gamefiltertcp: bool,

    #[arg(long = "gamefilterudp", short = 'u', help = "Enable gamefilterudp")]
    gamefilterudp: bool,

    #[arg(long = "cache-dir", short = 'd', help = "Cache directory for downloaded dependencies and strategies")]
    cache_dir: Option<String>,

    #[arg(short = 'h', long = "help", help = "Show this help")]
    help: bool,
}

fn show_help() {
    println!("{}", rust_i18n::t!("cli_usage"));
    println!("{}", rust_i18n::t!("cli_desc"));
    println!("{}", rust_i18n::t!("cli_opts"));
    println!("{}", rust_i18n::t!("cli_opt_c"));
    println!("{}", rust_i18n::t!("cli_opt_s"));
    println!("{}", rust_i18n::t!("cli_opt_i"));
    println!("{}", rust_i18n::t!("cli_opt_t"));
    println!("{}", rust_i18n::t!("cli_opt_u"));
    println!("{}", rust_i18n::t!("cli_opt_d"));
    println!("{}", rust_i18n::t!("cli_opt_h"));
    println!("{}", rust_i18n::t!("cli_modes"));
    println!("{}", rust_i18n::t!("cli_mode1"));
    println!("{}", rust_i18n::t!("cli_mode2"));
    println!("{}", rust_i18n::t!("cli_mode3"));
}

fn main() {
    let loc = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    if loc.to_lowercase().starts_with("ru") {
        rust_i18n::set_locale("ru");
    } else {
        rust_i18n::set_locale("en");
    }

    #[cfg(target_os = "windows")]
    {
        if std::env::args().any(|arg| arg == "--service") {
            if let Err(e) = inits::winservice::run_service() {
                eprintln!("{}{}", rust_i18n::t!("err_srv"), e);
                std::process::exit(1);
            }
            return;
        }
    }

    platform::ensure_admin();

    let args = Cli::parse();

    if let Some(ref d) = args.cache_dir {
        std::env::set_var("ZAPRET_CACHE_DIR", d);
    }

    if args.help {
        show_help();
        return;
    }

    let mut use_interface = args.interface.clone();
    let mut use_strategy = args.strategy.clone();
    let mut use_gamefilter_tcp = args.gamefiltertcp;
    let mut use_gamefilter_udp = args.gamefilterudp;
    #[cfg(target_os = "linux")]
    let mut use_backend: LinuxBackend = LinuxBackend::Nftables;
    let mut is_interactive = true;

    if let Some(config_file) = &args.config {
        println!("{}{}", rust_i18n::t!("msg_load_cfg"), config_file);
        match config::load_config(config_file) {
            Ok(cfg) => {
                use_interface = cfg.interface;
                use_strategy = Some(cfg.strategy);
                use_gamefilter_tcp = cfg.gamefilter_tcp;
                use_gamefilter_udp = cfg.gamefilter_udp;
                #[cfg(target_os = "linux")]
                { use_backend = LinuxBackend::from_config(&cfg.backend); }
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

    loop {
        if is_interactive {
            let interfaces = config::get_interfaces();
            let strategies = strategy::get_strategies();
            let mut app = tui::AppState::new(interfaces, strategies);

            let res = tui::run_tui(&mut app);
            if let Err(e) = res {
                println!("{}{}", rust_i18n::t!("err_tui"), e);
                exit(1);
            }

            use_interface = app
                .interfaces
                .get(app.selected_interface)
                .unwrap_or(&"any".to_string())
                .to_string();
            use_strategy = app.strategies.get(app.selected_strategy).cloned();
            use_gamefilter_tcp = app.tcp_gamefilter;
            use_gamefilter_udp = app.udp_gamefilter;
            #[cfg(target_os = "linux")]
            { use_backend = app.selected_backend; }

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

        let nfqws_ok = download::check_nfqws_installed();
        let strat_ok = download::check_strategies_installed();
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

        #[cfg(target_os = "linux")]
        let backend = use_backend;

        #[cfg(target_os = "windows")]
        let backend = WinDivertBackend;

        #[cfg(target_os = "linux")]
        let backend_info = format!(", backend={}", use_backend.to_config());
        #[cfg(not(target_os = "linux"))]
        let backend_info = String::new();

        println!(
            "{}{}, interface={}, gamefiltertcp={}, gamefilterudp={}{}",
            rust_i18n::t!("msg_run_params"), strategy_file, use_interface, use_gamefilter_tcp, use_gamefilter_udp, backend_info
        );

        runner::run_zapret(
            &strategy_file,
            &use_interface,
            use_gamefilter_tcp,
            use_gamefilter_udp,
            &backend,
        );

        thread::sleep(Duration::from_millis(100));
        println!("{}", rust_i18n::t!("msg_zapret_started"));

        RUNNING.store(true, Ordering::SeqCst);

        while RUNNING.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100));
        }

        runner::stop_zapret(&backend);

        if !is_interactive {
            break;
        }
    }
}
