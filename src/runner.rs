use crate::firewalls::FirewallBackend;
use crate::strategy::{self, GameFilterPorts, ParsedStrategy};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static NFQWS_PROCESSES: Mutex<Vec<Child>> = Mutex::new(Vec::new());

/// Returns true if any spawned zapret process is still running.
pub fn nfqws_process_running() -> bool {
    let Ok(mut procs) = NFQWS_PROCESSES.lock() else {
        return false;
    };
    procs
        .iter_mut()
        .any(|p| p.try_wait().map(|s| s.is_none()).unwrap_or(false))
}

#[cfg(target_os = "linux")]
fn kill_stale_zapret() {
    let _ = Command::new("pkill").arg("-9").arg("nfqws").output();
}

#[cfg(target_os = "windows")]
fn kill_stale_zapret() {
    let _ = Command::new("taskkill").args(["/F", "/IM", "winws.exe"]).output();
}

fn repo_path() -> PathBuf {
    PathBuf::from(env::var("REPO_DIR").unwrap_or_else(|_| {
        crate::config::get_cache_dir()
            .join("zapret-discord-youtube-linux")
            .to_string_lossy()
            .into_owned()
    }))
}

fn strategy_file_path(repo_path: &Path, strategy_file: &str) -> PathBuf {
    let custom = repo_path.join("custom-strategies").join(strategy_file);
    if custom.exists() {
        custom
    } else {
        repo_path.join(strategy_file)
    }
}

fn game_filter(use_tcp: bool, use_udp: bool) -> Option<GameFilterPorts> {
    if use_tcp || use_udp {
        Some(GameFilterPorts {
            ports: "50000-50100".to_string(),
            tcp_ports: "50000-50100".to_string(),
            udp_ports: "50000-50100".to_string(),
        })
    } else {
        None
    }
}

fn bin_path() -> PathBuf {
    let bin_name = if env::consts::OS == "windows" {
        "winws.exe"
    } else {
        "nfqws"
    };
    crate::config::get_cache_dir().join("bin").join(bin_name)
}

fn ensure_user_lists(repo_path: &Path) {
    let lists_dir = repo_path.join("lists");
    for name in &[
        "list-general-user.txt",
        "list-exclude-user.txt",
        "ipset-exclude-user.txt",
    ] {
        let path = lists_dir.join(name);
        if !path.exists() {
            let _ = fs::write(&path, "");
        }
    }
}

fn build_args(parsed: &ParsedStrategy, ttl: Option<u8>) -> Vec<String> {
    #[cfg(target_os = "linux")]
    let mut args = vec!["--dpi-desync-fwmark=0x40000000".to_string(), "--qnum=200".to_string()];

    #[cfg(target_os = "windows")]
    let mut args = vec![
        format!("--wf-tcp={}", parsed.tcp_ports),
        format!("--wf-udp={}", parsed.udp_ports),
    ];

    // Each strategy group is a separate desync profile: `--new` finalizes the
    // current profile and starts a fresh one, so TTL options must be injected
    // into every group (not just at the end of the whole command line).
    for param in &parsed.nfqws_params {
        for p in param.split_whitespace() {
            let p = p.replace('"', "");
            if p.is_empty() || p == "^" {
                continue;
            }
            // A fixed TTL overrides any TTL/autottl settings baked into the
            // strategy file, otherwise they would shadow our values.
            if ttl.is_some() && (p.starts_with("--dpi-desync-ttl") || p.starts_with("--dpi-desync-autottl")) {
                continue;
            }
            args.push(p.to_string());
        }
        // Injected at the end of the group so they win over the strategy's own
        // params in case the filtering above missed anything (winws applies
        // the last occurrence of a parameter).
        if let Some(ttl) = ttl {
            args.push(format!("--dpi-desync-ttl={}", ttl));
            args.push(format!("--dpi-desync-ttl6={}", ttl));
            args.push("--dpi-desync-autottl=-".to_string());
            args.push("--dpi-desync-autottl6=-".to_string());
        }
    }
    args
}

fn set_cap(bin_path: &Path) -> bool {
    #[cfg(target_os = "linux")]
    {
        Command::new("setcap")
            .args(["cap_net_admin+ep", &bin_path.to_string_lossy()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = bin_path;
        true
    }
}

/// Run the zapret firewall rule setup and spawn the nfqws daemon.
pub fn run_zapret(
    strategy_file: &str,
    interface: &str,
    use_tcp: bool,
    use_udp: bool,
    router_mode: bool,
    backend: &dyn FirewallBackend,
) {
    let mut term: Vec<String> = Vec::new();

    let repo_path = repo_path();
    let path = strategy_file_path(&repo_path, strategy_file);

    let parsed = match strategy::parse_bat_file(path.to_str().unwrap(), game_filter(use_tcp, use_udp).as_ref()) {
        Ok(p) => p,
        Err(e) => {
            println!("{}{}", rust_i18n::t!("err_parse_strat"), e);
            return;
        }
    };

    // Setup firewall
    if let Err(e) = backend.setup(&parsed.tcp_ports, &parsed.udp_ports, interface, router_mode) {
        println!("{}{}", rust_i18n::t!("msg_err_firewall"), e);
    }

    if router_mode {
        crate::platform::enable_ip_forward();
    }

    // Kill any leftover nfqws processes from previous runs
    kill_stale_zapret();

    // Ensure user list files exist (original scripts create empty ones)
    ensure_user_lists(&repo_path);

    let msg = rust_i18n::t!("msg_start_nfqws").to_string();
    term.push(msg.clone());
    println!("{}", msg);

    let bin_path = bin_path();
    if !bin_path.exists() {
        let msg = rust_i18n::t!("err_bin_miss").replace("{:?}", &format!("{:?}", bin_path));
        term.push(msg.clone());
        println!("{}", msg);
        return;
    }

    // Set CAP_NET_ADMIN on binary so it can use nfqueue without root
    if !set_cap(&bin_path) {
        let msg = rust_i18n::t!("err_setcap").to_string();
        term.push(msg.clone());
        println!("{}", msg);
    }

    let ttl = crate::config::load_ttl();
    let args = build_args(&parsed, ttl);

    let cmd_msg = format!("{}{:?} {:?}", rust_i18n::t!("msg_cmd"), bin_path, args);
    term.push(cmd_msg.clone());
    println!("{}", cmd_msg);

    // Capture nfqws output to a temp file
    let tmp_log = crate::config::get_cache_dir().join("logs").join("nfqws_output.tmp");
    let _ = fs::create_dir_all(tmp_log.parent().unwrap());
    let output_file = match fs::File::create(&tmp_log) {
        Ok(f) => f,
        Err(_) => {
            let msg = "failed to create temp log file".to_string();
            term.push(msg.clone());
            println!("{}", msg);
            return;
        }
    };

    let out_dup = match output_file.try_clone() {
        Ok(f) => f,
        Err(_) => {
            let msg = "failed to clone temp log file handle".to_string();
            term.push(msg.clone());
            println!("{}", msg);
            return;
        }
    };

    match Command::new(&bin_path)
        .args(&args)
        .current_dir(&repo_path)
        .stdin(Stdio::null())
        .stdout(output_file)
        .stderr(out_dup)
        .spawn()
    {
        Ok(child) => {
            if let Ok(mut procs) = NFQWS_PROCESSES.lock() {
                procs.push(child);
            }

            let msg = rust_i18n::t!("msg_nfqws_run").to_string();
            term.push(msg.clone());
            println!("{}", msg);

            // Wait briefly for nfqws startup output
            thread::sleep(Duration::from_millis(300));

            // Read captured output
            let nfqws_out = fs::read_to_string(&tmp_log).unwrap_or_default();
            let _ = fs::remove_file(&tmp_log);

            if !nfqws_out.is_empty() {
                term.push(nfqws_out.clone());
                print!("{}", nfqws_out);
            }
        }
        Err(e) => {
            let msg = format!("{}{}", rust_i18n::t!("err_start_nfqws"), e);
            term.push(msg.clone());
            println!("{}", msg);
        }
    }

    crate::logger::log_nfqws_launch(&bin_path.to_string_lossy(), &parsed.nfqws_params, &term);
}

/// Run zapret silently for autotune (no println, returns Result).
pub fn run_zapret_silent(
    strategy_file: &str,
    interface: &str,
    use_tcp: bool,
    use_udp: bool,
    backend: &dyn FirewallBackend,
) -> Result<(), String> {
    let ttl = crate::config::load_ttl();
    run_zapret_silent_impl(strategy_file, interface, use_tcp, use_udp, backend, ttl)
}

/// Run zapret silently with an explicit fixed DPI TTL override (used by TTL autopick).
pub fn run_zapret_silent_ttl(
    strategy_file: &str,
    interface: &str,
    use_tcp: bool,
    use_udp: bool,
    backend: &dyn FirewallBackend,
    ttl: u8,
) -> Result<(), String> {
    run_zapret_silent_impl(strategy_file, interface, use_tcp, use_udp, backend, Some(ttl))
}

fn run_zapret_silent_impl(
    strategy_file: &str,
    interface: &str,
    use_tcp: bool,
    use_udp: bool,
    backend: &dyn FirewallBackend,
    ttl: Option<u8>,
) -> Result<(), String> {
    let repo_path = repo_path();
    let path = strategy_file_path(&repo_path, strategy_file);

    let parsed = strategy::parse_bat_file(
        path.to_str().ok_or("invalid strategy path")?,
        game_filter(use_tcp, use_udp).as_ref(),
    )
    .map_err(|e| format!("parse error: {}", e))?;

    if let Err(e) = backend.setup(&parsed.tcp_ports, &parsed.udp_ports, interface, false) {
        return Err(format!("firewall setup error: {}", e));
    }

    kill_stale_zapret();

    ensure_user_lists(&repo_path);

    let bin_path = bin_path();
    if !bin_path.exists() {
        return Err(format!("binary not found: {:?}", bin_path));
    }

    let _ = set_cap(&bin_path);
    let args = build_args(&parsed, ttl);

    crate::logger::log_nfqws_launch(&bin_path.to_string_lossy(), &parsed.nfqws_params, &[]);
    match Command::new(&bin_path)
        .args(&args)
        .current_dir(&repo_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            if let Ok(mut procs) = NFQWS_PROCESSES.lock() {
                procs.push(child);
            }
            Ok(())
        }
        Err(e) => Err(format!("failed to start nfqws: {}", e)),
    }
}

/// Clear the firewall rules and stop any running processes.
pub fn stop_zapret(backend: &dyn FirewallBackend) {
    let mut term: Vec<String> = Vec::new();

    let msg = rust_i18n::t!("msg_zapret_stop").to_string();
    term.push(msg.clone());
    println!("{}", msg);

    if let Ok(mut procs) = NFQWS_PROCESSES.lock() {
        for child in procs.iter_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        procs.clear();
    }

    let _ = backend.clear();
    crate::platform::disable_ip_forward();

    let msg = rust_i18n::t!("msg_zapret_clear").to_string();
    term.push(msg.clone());
    println!("{}", msg);

    crate::logger::log_stop(&term);
}
