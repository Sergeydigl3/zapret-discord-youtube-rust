mod config;
mod download;
mod firewalls;
mod platform;
mod runner;
mod strategy;
mod tui;

use clap::Parser;
use std::process::exit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(not(target_os = "windows"))]
use firewalls::nftables::NftablesBackend;

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
    println!("Usage: run [options]");
    println!("\nRun zapret in foreground (useful for testing).");
    println!("\nOptions:");
    println!("    -c, --config FILE       Load configuration from file");
    println!("    -s, --strategy NAME     Use specific strategy");
    println!("    -i, --interface NAME    Network interface (default: any)");
    println!("    -t, --gamefiltertcp     Enable gamefiltertcp");
    println!("    -u, --gamefilterudp     Enable gamefilterudp");
    println!("    -d, --cache-dir PATH    Cache directory for downloads (default: binary directory)");
    println!("    -h, --help              Show this help");
    println!("\nModes:");
    println!("    1. Interactive mode (no options):");
    println!("       run");
    println!("       Prompts for all parameters");
    println!("\n    2. Load from config file:");
    println!("       run --config conf.env");
    println!("       Uses existing configuration file");
    println!("\n    3. Direct parameters:");
    println!("       run -s discord -i eth0 -t");
    println!("       Specify all parameters directly");
}

fn main() {
    #[cfg(target_os = "windows")]
    platform::ensure_admin();

    #[cfg(target_os = "linux")]
    {
        let not_root = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim() != "0")
            .unwrap_or(true);
        if not_root {
            println!("Для работы требуются права root. Попытка инициализации pkexec...");
            let mut status = std::process::Command::new("pkexec")
                .arg(std::env::current_exe().unwrap_or_default())
                .args(std::env::args().skip(1))
                .status();

            let pkexec_failed = match &status {
                Ok(st) => !st.success(),
                Err(_) => true,
            };

            if pkexec_failed {
                if let Ok(ref st) = status {
                    eprintln!("pkexec завершился с ошибкой (код: {})", st);
                } else if let Err(ref e) = status {
                    eprintln!("Не удалось запустить pkexec: {}", e);
                }
                println!("Попытка инициализации sudo...");
                status = std::process::Command::new("sudo")
                    .arg(std::env::current_exe().unwrap_or_default())
                    .args(std::env::args().skip(1))
                    .status();
            }

            match status {
                Ok(st) => {
                    std::process::exit(st.code().unwrap_or(1));
                }
                Err(e) => {
                    eprintln!("Ошибка запуска sudo/pkexec: {}. Попробуйте запустить вручную с правами root.", e);
                    std::process::exit(1);
                }
            }
        }
    }

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
    let mut is_interactive = true;

    if let Some(config_file) = &args.config {
        println!("Загрузка конфигурации из: {}", config_file);
        match config::load_config(config_file) {
            Ok(cfg) => {
                use_interface = cfg.interface;
                use_strategy = Some(cfg.strategy);
                use_gamefilter_tcp = cfg.gamefilter_tcp;
                use_gamefilter_udp = cfg.gamefilter_udp;
                is_interactive = false;
            }
            Err(e) => {
                println!("Error loading config: {}", e);
                exit(1);
            }
        }
    } else if use_strategy.is_some() {
        is_interactive = false;
    }

    if is_interactive {
        let interfaces = config::get_interfaces();
        let strategies = strategy::get_strategies();
        let mut app = tui::AppState::new(interfaces, strategies);

        let res = tui::run_tui(&mut app);
        if let Err(e) = res {
            println!("TUI Error: {}", e);
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

        if let Some(ref strat) = use_strategy {
            let _ = config::save_tui_state(
                &use_interface,
                strat,
                use_gamefilter_tcp,
                use_gamefilter_udp,
            );
        }

        if app.should_quit {
            println!("Exited by user.");
            return;
        }
    }

    let strategy_file = match use_strategy {
        Some(s) => s,
        None => {
            println!("No strategy selected.");
            exit(1);
        }
    };

    let nfqws_ok = download::check_nfqws_installed();
    let strat_ok = download::check_strategies_installed();
    if !nfqws_ok || !strat_ok {
        if !nfqws_ok && !strat_ok {
            eprintln!("Ошибка: Отсутствуют оба компонента (nfqws и стратегии). Запуск отклонен.");
        } else if !nfqws_ok {
            eprintln!("Ошибка: Отсутствует nfqws (ядро). Запуск отклонен.");
        } else {
            eprintln!("Ошибка: Отсутствуют стратегии. Запуск отклонен.");
        }
        exit(1);
    }

    #[cfg(not(target_os = "windows"))]
    let backend = NftablesBackend;

    #[cfg(target_os = "windows")]
    let backend = WinDivertBackend;

    println!(
        "Запуск с параметрами: strategy={}, interface={}, gamefiltertcp={}, gamefilterudp={}",
        strategy_file, use_interface, use_gamefilter_tcp, use_gamefilter_udp
    );

    runner::run_zapret(
        &strategy_file,
        &use_interface,
        use_gamefilter_tcp,
        use_gamefilter_udp,
        &backend,
    );

    println!("\nzapret запущен. Нажмите Ctrl+C для завершения...");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    runner::stop_zapret(&backend);
    exit(0);
}
