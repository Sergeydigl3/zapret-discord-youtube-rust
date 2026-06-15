use crate::firewalls::FirewallBackend;
use crate::strategy::{self, GameFilterPorts};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::Mutex;

static NFQWS_PROCESSES: Mutex<Vec<Child>> = Mutex::new(Vec::new());

/// Run the zapret firewall rule setup and spawn the nfqws daemon.
pub fn run_zapret(
    strategy_file: &str,
    interface: &str,
    use_tcp: bool,
    use_udp: bool,
    backend: &dyn FirewallBackend,
) {
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
    let _ = Command::new("pkill")
        .arg("-9")
        .arg("nfqws")
        .output();

    // 4. Ensure user list files exist (original scripts create empty ones)
    let lists_dir = repo_path.join("lists");
    for name in &["list-general-user.txt", "list-exclude-user.txt", "ipset-exclude-user.txt"] {
        let path = lists_dir.join(name);
        if !path.exists() {
            let _ = fs::write(&path, "");
        }
    }

    // 5. Start nfqws
    println!("{}", rust_i18n::t!("msg_start_nfqws"));

    let bin_dir = crate::config::get_cache_dir().join("bin");
    let bin_name = if env::consts::OS == "windows" {
        "winws.exe"
    } else {
        "nfqws"
    };
    let bin_path = bin_dir.join(bin_name);

    if !bin_path.exists() {
        println!("{}", rust_i18n::t!("err_bin_miss").replace("{:?}", &format!("{:?}", bin_path)));
        return;
    }

    // Set CAP_NET_ADMIN on binary so it can use nfqueue without root
    let cap_status = Command::new("setcap")
        .args(["cap_net_admin+ep", &bin_path.to_string_lossy()])
        .output();
    match cap_status {
        Ok(o) if o.status.success() => (),
        _ => println!("{}", rust_i18n::t!("err_setcap")),
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

    println!("{}{:?} {:?}", rust_i18n::t!("msg_cmd"), bin_path, args);
    match Command::new(&bin_path).args(&args).current_dir(&repo_path).spawn() {
        Ok(child) => {
            if let Ok(mut procs) = NFQWS_PROCESSES.lock() {
                procs.push(child);
            }
            println!("{}", rust_i18n::t!("msg_nfqws_run"));
        }
        Err(e) => {
            println!("{}{}", rust_i18n::t!("err_start_nfqws"), e);
        }
    }
}

/// Clear the firewall rules and stop any running processes.
pub fn stop_zapret(backend: &dyn FirewallBackend) {
    println!("{}", rust_i18n::t!("msg_zapret_stop"));
    if let Ok(mut procs) = NFQWS_PROCESSES.lock() {
        for child in procs.iter_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        procs.clear();
    }
    let _ = backend.clear();
    println!("{}", rust_i18n::t!("msg_zapret_clear"));
}
