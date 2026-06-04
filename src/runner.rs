use crate::firewalls::FirewallBackend;
use crate::strategy::{self, GameFilterPorts};
use std::env;
use std::path::Path;

/// Run the zapret firewall rule setup and spawn the nfqws daemon.
pub fn run_zapret(
    strategy_file: &str,
    interface: &str,
    use_tcp: bool,
    use_udp: bool,
    backend: &dyn FirewallBackend,
) {
    // 1. Clear existing firewall rules
    let _ = backend.clear();

    // 2. Parse strategy file
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
            println!("Error parsing strategy: {}", e);
            return;
        }
    };

    // 3. Setup firewall
    if let Err(e) = backend.setup(&parsed.tcp_ports, &parsed.udp_ports, interface) {
        println!("Error configuring firewall: {}", e);
    }

    // 4. Start nfqws
    println!("Запуск nfqws...");
    let mut nfqws_args = vec![
        "--daemon".to_string(),
        "--dpi-desync-fwmark=0x40000000".to_string(),
        "--qnum=200".to_string(),
    ];
    for param in parsed.nfqws_params {
        for p in param.split_whitespace() {
            nfqws_args.push(p.to_string());
        }
    }

    let bin_dir = crate::config::get_cache_dir().join("bin");
    let bin_path = if env::consts::OS == "windows" {
        bin_dir.join("winws.exe")
    } else {
        bin_dir.join("nfqws")
    };

    println!("[Stub] {:?} spawned with args: {:?}", bin_path, nfqws_args);
}

/// Clear the firewall rules and stop any running processes.
pub fn stop_zapret(backend: &dyn FirewallBackend) {
    println!("\n[Stub] Остановка nfqws...");
    let _ = backend.clear();
    println!("Очистка завершена.");
}
