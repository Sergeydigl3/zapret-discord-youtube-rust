//! Process lifecycle for the zapret daemon.
//!
//! Owns the child-process registry, the spawn/teardown of `nfqws` / `winws` and
//! the terminal output that goes with each run. Building the command line lives
//! in [`args`], granting capabilities in [`caps`].

mod args;
mod caps;

use crate::firewall::FirewallBackend;
use crate::strategy;
use args::{build_args, game_filter, strategy_file_path};
use caps::set_cap;
use std::fs;
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

fn ensure_user_lists() {
    let lists_dir = crate::paths::repo_lists_dir();
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

/// Run the zapret firewall rule setup and spawn the nfqws daemon.
pub fn run_zapret(strategy_file: &str, interface: &str, use_tcp: bool, use_udp: bool, backend: &dyn FirewallBackend) {
    let mut term: Vec<String> = Vec::new();

    let repo_path = crate::paths::repo_dir();
    let path = strategy_file_path(&repo_path, strategy_file);

    let parsed = match strategy::parse_bat_file(path.to_str().unwrap(), game_filter(use_tcp, use_udp).as_ref()) {
        Ok(p) => p,
        Err(e) => {
            println!("{}{}", rust_i18n::t!("err_parse_strat"), e);
            return;
        }
    };

    // Setup firewall
    if let Err(e) = backend.setup(&parsed.tcp_ports, &parsed.udp_ports, interface) {
        println!("{}{}", rust_i18n::t!("msg_err_firewall"), e);
    }

    // Kill any leftover nfqws processes from previous runs
    crate::process::kill_stale_zapret();

    // Ensure user list files exist (original scripts create empty ones)
    ensure_user_lists();

    let msg = rust_i18n::t!("msg_start_nfqws").to_string();
    term.push(msg.clone());
    println!("{}", msg);

    let bin_path = crate::paths::binary_path();
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
    let tmp_log = crate::paths::nfqws_output_log();
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

    crate::diagnose::log_nfqws_launch(&bin_path.to_string_lossy(), &parsed.nfqws_params, &term);
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
    let repo_path = crate::paths::repo_dir();
    let path = strategy_file_path(&repo_path, strategy_file);

    let parsed = strategy::parse_bat_file(
        path.to_str().ok_or("invalid strategy path")?,
        game_filter(use_tcp, use_udp).as_ref(),
    )
    .map_err(|e| format!("parse error: {}", e))?;

    if let Err(e) = backend.setup(&parsed.tcp_ports, &parsed.udp_ports, interface) {
        return Err(format!("firewall setup error: {}", e));
    }

    crate::process::kill_stale_zapret();

    ensure_user_lists();

    let bin_path = crate::paths::binary_path();
    if !bin_path.exists() {
        return Err(format!("binary not found: {:?}", bin_path));
    }

    let _ = set_cap(&bin_path);
    let args = build_args(&parsed, ttl);

    crate::diagnose::log_nfqws_launch(&bin_path.to_string_lossy(), &parsed.nfqws_params, &[]);
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

    let msg = rust_i18n::t!("msg_zapret_clear").to_string();
    term.push(msg.clone());
    println!("{}", msg);

    crate::diagnose::log_stop(&term);
}
