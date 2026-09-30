use std::env;
use std::fs;
use std::process::Command;

/// Collects what the machine currently looks like, for the `--- System Info ---`
/// block of the log.
///
/// Shells out for the facts that have no cheap filesystem answer (`uname`, the
/// netfilter tools, `winws --version`) and reads `/proc`, `/etc` and the
/// strategies directory for the rest. Every collector is best-effort: a tool
/// that is missing or a file that cannot be read is reported as such rather
/// than aborting the log.
pub(super) fn collect_system_info() -> Vec<String> {
    let mut info = Vec::new();

    info.push(format!("OS: {}", env::consts::OS));
    info.push(format!("Arch: {}", env::consts::ARCH));

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("uname").arg("-r").output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                let v = s.trim();
                if !v.is_empty() {
                    info.push(format!("Kernel: {}", v));
                }
            }
        }

        if let Ok(content) = fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("PRETTY_NAME=") {
                    info.push(format!("Distro: {}", val.trim_matches('"')));
                    break;
                }
            }
        }

        let modules_raw = fs::read_to_string("/proc/modules").unwrap_or_default();
        let nf_modules: Vec<&str> = modules_raw
            .lines()
            .filter_map(|l| {
                let name = l.split_whitespace().next()?;
                if name.starts_with("nf_")
                    || name.starts_with("nft_")
                    || name.starts_with("ip_t")
                    || name.starts_with("ip6_t")
                    || name.starts_with("iptable")
                    || name.starts_with("ip6table")
                    || name == "arptables"
                    || name == "ebtables"
                {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();
        if nf_modules.is_empty() {
            info.push("Netfilter modules: none loaded".to_string());
        } else {
            info.push(format!("Netfilter modules: {}", nf_modules.join(", ")));
        }

        for tool in &["nft", "iptables", "iptables-nft"] {
            let ok = Command::new(tool)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            info.push(format!("{}: {}", tool, if ok { "available" } else { "not found" }));
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("cmd").args(["/c", "ver"]).output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                info.push(format!("Windows: {}", s.trim()));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            let mut interfaces: Vec<String> = entries
                .flatten()
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            interfaces.sort();
            let iface_str = if interfaces.is_empty() {
                "none".to_string()
            } else {
                interfaces.join(", ")
            };
            info.push(format!("Interfaces: {}", iface_str));
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let interfaces = crate::platform::get_interfaces();
        info.push(format!("Interfaces: {}", interfaces.join(", ")));
    }

    let cache_dir = crate::paths::cache_dir();

    let bin_dir = cache_dir.join("bin");
    let bin_name = if env::consts::OS == "windows" {
        "winws.exe"
    } else {
        "nfqws"
    };
    let bin_path = bin_dir.join(bin_name);
    if bin_path.exists() {
        info.push("nfqws: installed".to_string());
        if let Ok(output) = Command::new(&bin_path).arg("--version").output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                let ver = s.lines().next().unwrap_or("").trim().to_string();
                if !ver.is_empty() {
                    info.push(format!("nfqws version: {}", ver));
                }
            }
            if let Ok(s) = String::from_utf8(output.stderr) {
                let ver = s.lines().next().unwrap_or("").trim().to_string();
                if !ver.is_empty() && !info.iter().any(|l| l.starts_with("nfqws version:")) {
                    info.push(format!("nfqws version: {}", ver));
                }
            }
        }
    } else {
        info.push("nfqws: not installed".to_string());
    }

    let strat_dir = cache_dir.join(crate::paths::REPO_DIR_NAME);
    if strat_dir.exists() {
        let mut count = 0;
        if let Ok(entries) = fs::read_dir(&strat_dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.ends_with(".bat") {
                        count += 1;
                    }
                }
            }
        }
        if let Ok(entries) = fs::read_dir(strat_dir.join("custom-strategies")) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.ends_with(".bat") {
                        count += 1;
                    }
                }
            }
        }
        let version_file = strat_dir.join(".service").join("version.txt");
        let ver = fs::read_to_string(&version_file).unwrap_or_default();
        let ver_str = ver.trim();
        if ver_str.is_empty() {
            info.push(format!("Strategies: {} .bat files", count));
        } else {
            info.push(format!("Strategies: {} .bat files (version {})", count, ver_str));
        }
    } else {
        info.push("Strategies: not installed".to_string());
    }

    info.push(format!(
        "User: {}",
        env::var("USER").unwrap_or_else(|_| env::var("USERNAME").unwrap_or_default())
    ));
    info.push(format!("Cache dir: {}", cache_dir.display()));

    info
}
