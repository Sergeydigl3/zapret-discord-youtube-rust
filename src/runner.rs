use crate::firewalls::FirewallBackend;
use crate::strategy::{self, GameFilterPorts};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static NFQWS_PROCESSES: Mutex<Vec<Child>> = Mutex::new(Vec::new());

/// Run the zapret firewall rule setup and spawn the nfqws daemon.
pub fn run_zapret(
    strategy_file: &str,
    interface: &str,
    use_tcp: bool,
    use_udp: bool,
    backend: &dyn FirewallBackend,
) {
    let mut term: Vec<String> = Vec::new();

    // 1. Parse strategy file
    let repo_dir = env::var("REPO_DIR").unwrap_or_else(|_| {
        crate::config::get_cache_dir()
            .join("zapret-discord-youtube-linux")
            .to_string_lossy()
            .into_owned()
    });
    let repo_path = Path::new(&repo_dir);
    let mut path = repo_path.join("custom-strategies").join(strategy_file);
    if !path.exists() {
        path = repo_path.join(strategy_file);
    }

    // Stubs for game filter ports
    let game_filter = if use_tcp || use_udp {
        Some(GameFilterPorts {
            ports: "50000-50100".to_string(),
            tcp_ports: "50000-50100".to_string(),
            udp_ports: "50000-50100".to_string(),
        })
    } else {
        None
    };

    let parsed = match strategy::parse_bat_file(path.to_str().unwrap(), game_filter.as_ref()) {
        Ok(p) => p,
        Err(e) => {
            println!("{}{}", rust_i18n::t!("err_parse_strat"), e);
            return;
        }
    };

    // 2. Setup firewall
    if let Err(e) = backend.setup(&parsed.tcp_ports, &parsed.udp_ports, interface) {
        println!("{}{}", rust_i18n::t!("msg_err_firewall"), e);
    }

    // 3. Kill any leftover nfqws processes from previous runs
    let _ = Command::new("pkill").arg("-9").arg("nfqws").output();

    // 4. Ensure user list files exist (original scripts create empty ones)
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

    // 5. Start nfqws
    let msg = rust_i18n::t!("msg_start_nfqws").to_string();
    term.push(msg.clone());
    println!("{}", msg);

    let bin_dir = crate::config::get_cache_dir().join("bin");
    let bin_name = if env::consts::OS == "windows" {
        "winws.exe"
    } else {
        "nfqws"
    };
    let bin_path = bin_dir.join(bin_name);

    if !bin_path.exists() {
        let msg = rust_i18n::t!("err_bin_miss").replace("{:?}", &format!("{:?}", bin_path));
        term.push(msg.clone());
        println!("{}", msg);
        return;
    }

    // Set CAP_NET_ADMIN on binary so it can use nfqueue without root
    let cap_status = Command::new("setcap")
        .args(["cap_net_admin+ep", &bin_path.to_string_lossy()])
        .output();
    match cap_status {
        Ok(o) if o.status.success() => (),
        _ => {
            let msg = rust_i18n::t!("err_setcap").to_string();
            term.push(msg.clone());
            println!("{}", msg);
        }
    }

    #[cfg(target_os = "linux")]
    let mut args = vec![
        "--dpi-desync-fwmark=0x40000000".to_string(),
        "--qnum=200".to_string(),
    ];

    #[cfg(target_os = "windows")]
    let mut args = vec![
        format!("--wf-tcp={}", parsed.tcp_ports),
        format!("--wf-udp={}", parsed.udp_ports),
    ];

    for param in &parsed.nfqws_params {
        for p in param.split_whitespace() {
            let p = p.replace('"', "");
            if !p.is_empty() && p != "^" {
                args.push(p.to_string());
            }
        }
    }

    let cmd_msg = format!("{}{:?} {:?}", rust_i18n::t!("msg_cmd"), bin_path, args);
    term.push(cmd_msg.clone());
    println!("{}", cmd_msg);

    // Capture nfqws output to a temp file
    let tmp_log = crate::config::get_cache_dir()
        .join("logs")
        .join("nfqws_output.tmp");
    let _ = fs::create_dir_all(tmp_log.parent().unwrap());
    let output_file = match fs::File::create(&tmp_log) {
        Ok(f) => f,
        Err(_) => {
            let msg = format!("failed to create temp log file");
            term.push(msg.clone());
            println!("{}", msg);
            return;
        }
    };

    let out_dup = match output_file.try_clone() {
        Ok(f) => f,
        Err(_) => {
            let msg = format!("failed to clone temp log file handle");
            term.push(msg.clone());
            println!("{}", msg);
            return;
        }
    };

    match Command::new(&bin_path)
        .args(&args)
        .current_dir(&repo_path)
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
    let repo_dir = env::var("REPO_DIR").unwrap_or_else(|_| {
        crate::config::get_cache_dir()
            .join("zapret-discord-youtube-linux")
            .to_string_lossy()
            .into_owned()
    });
    let repo_path = Path::new(&repo_dir);
    let mut path = repo_path.join("custom-strategies").join(strategy_file);
    if !path.exists() {
        path = repo_path.join(strategy_file);
    }

    let game_filter = if use_tcp || use_udp {
        Some(GameFilterPorts {
            ports: "50000-50100".to_string(),
            tcp_ports: "50000-50100".to_string(),
            udp_ports: "50000-50100".to_string(),
        })
    } else {
        None
    };

    let parsed = strategy::parse_bat_file(
        path.to_str().ok_or("invalid strategy path")?,
        game_filter.as_ref(),
    )
    .map_err(|e| format!("parse error: {}", e))?;

    if let Err(e) = backend.setup(&parsed.tcp_ports, &parsed.udp_ports, interface) {
        return Err(format!("firewall setup error: {}", e));
    }

    let _ = Command::new("pkill").arg("-9").arg("nfqws").output();

    let lists_dir = repo_path.join("lists");
    for name in &[
        "list-general-user.txt",
        "list-exclude-user.txt",
        "ipset-exclude-user.txt",
    ] {
        let lists_path = lists_dir.join(name);
        if !lists_path.exists() {
            let _ = fs::write(&lists_path, "");
        }
    }

    let bin_dir = crate::config::get_cache_dir().join("bin");
    let bin_name = if env::consts::OS == "windows" {
        "winws.exe"
    } else {
        "nfqws"
    };
    let bin_path = bin_dir.join(bin_name);

    if !bin_path.exists() {
        return Err(format!("binary not found: {:?}", bin_path));
    }

    let _ = Command::new("setcap")
        .args(["cap_net_admin+ep", &bin_path.to_string_lossy()])
        .output();

    #[cfg(target_os = "linux")]
    let mut args = vec![
        "--dpi-desync-fwmark=0x40000000".to_string(),
        "--qnum=200".to_string(),
    ];

    #[cfg(target_os = "windows")]
    let mut args = vec![
        format!("--wf-tcp={}", parsed.tcp_ports),
        format!("--wf-udp={}", parsed.udp_ports),
    ];

    for param in &parsed.nfqws_params {
        for p in param.split_whitespace() {
            let p = p.replace('"', "");
            if !p.is_empty() && p != "^" {
                args.push(p.to_string());
            }
        }
    }

    crate::logger::log_nfqws_launch(&bin_path.to_string_lossy(), &parsed.nfqws_params, &[]);
    match Command::new(&bin_path)
        .args(&args)
        .current_dir(&repo_path)
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

    crate::logger::log_stop(&term);
}
