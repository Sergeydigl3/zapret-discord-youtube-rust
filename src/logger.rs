use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::process::Command;

pub fn log_nfqws_launch(bin_path: &str, nfqws_params: &[String], terminal_output: &[String]) {
    let log_dir = crate::config::get_cache_dir().join("logs");
    let log_file = log_dir.join("zapret.log");

    if fs::create_dir_all(&log_dir).is_err() {
        return;
    }

    let mut file = match OpenOptions::new().create(true).write(true).truncate(true).open(&log_file) {
        Ok(f) => f,
        Err(_) => return,
    };

    let ts = timestamp();

    let _ = writeln!(file, "===== nfqws | {} =====", ts);
    let _ = writeln!(file, "");

    let _ = writeln!(file, "--- System Info ---");
    for line in collect_system_info() {
        let _ = writeln!(file, "{}", line);
    }
    let _ = writeln!(file, "");

    let config_path = crate::config::config_path();
    let config_content = fs::read_to_string(&config_path).unwrap_or_default();
    let _ = writeln!(file, "--- Config ---");
    let _ = write!(file, "{}", config_content);
    let _ = writeln!(file, "");

    if !terminal_output.is_empty() {
        let _ = writeln!(file, "--- Terminal ---");
        for line in terminal_output {
            let _ = writeln!(file, "{}", line);
        }
        let _ = writeln!(file, "");
    }

    let _ = writeln!(file, "--- Strategy params ---");
    let _ = writeln!(file, "binary: {}", bin_path);
    for param in nfqws_params {
        let _ = writeln!(file, "{}", param);
    }
}

fn collect_system_info() -> Vec<String> {
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
        let nf_modules: Vec<&str> = modules_raw.lines()
            .filter_map(|l| {
                let name = l.split_whitespace().next()?;
                if name.starts_with("nf_") || name.starts_with("nft_")
                    || name.starts_with("ip_t") || name.starts_with("ip6_t")
                    || name.starts_with("iptable") || name.starts_with("ip6table")
                    || name == "arptables" || name == "ebtables"
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
        let interfaces = crate::config::get_interfaces();
        info.push(format!("Interfaces: {}", interfaces.join(", ")));
    }

    let cache_dir = crate::config::get_cache_dir();

    let bin_dir = cache_dir.join("bin");
    let bin_name = if env::consts::OS == "windows" { "winws.exe" } else { "nfqws" };
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

    let strat_dir = cache_dir.join("zapret-discord-youtube-linux");
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

    info.push(format!("User: {}",
        env::var("USER").unwrap_or_else(|_| env::var("USERNAME").unwrap_or_default())));
    info.push(format!("Cache dir: {}", cache_dir.display()));

    info
}

pub fn log_stop(stop_output: &[String]) {
    let log_file = crate::config::get_cache_dir().join("logs").join("zapret.log");
    let mut file = match OpenOptions::new().create(true).append(true).open(&log_file) {
        Ok(f) => f,
        Err(_) => return,
    };

    let ts = timestamp();
    let _ = writeln!(file, "");
    let _ = writeln!(file, "===== stop | {} =====", ts);
    for line in stop_output {
        let _ = writeln!(file, "{}", line);
    }
}

fn timestamp() -> String {
    #[cfg(target_os = "linux")]
    if let Ok(output) = Command::new("date").args(["+%Y-%m-%d %H:%M:%S"]).output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            let t = s.trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(output) = Command::new("cmd").args(["/c", "echo %DATE% %TIME%"]).output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            let t = s.trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }

    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();

    let days = total_secs / 86400;
    let time = total_secs % 86400;
    let hours = time / 3600;
    let minutes = (time % 3600) / 60;
    let seconds = time % 60;

    fn is_leap(year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y as u64) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let month_days: &[i64] = if is_leap(y as u64) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0u32;
    for (i, &days_in_m) in month_days.iter().enumerate() {
        if remaining < days_in_m {
            m = (i + 1) as u32;
            break;
        }
        remaining -= days_in_m;
    }
    let d = remaining + 1;

    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, hours, minutes, seconds)
}
