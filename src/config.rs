use std::env;
use std::fs;

/// Runtime configuration assembled from CLI flags or a config file.
#[derive(Debug)]
pub struct RunConfig {
    pub interface: String,
    pub strategy: String,
    pub gamefilter_tcp: bool,
    pub gamefilter_udp: bool,
    pub router_mode: bool,
    pub backend: String,
    pub active_discord_fake: String,
    pub active_gamefilter_fake: String,
    pub dpi_desync_ttl: Option<u8>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            interface: "any".to_string(),
            strategy: String::new(),
            gamefilter_tcp: false,
            gamefilter_udp: false,
            router_mode: false,
            backend: "nftables".to_string(),
            active_discord_fake: "quic_initial_steamcommunity_com.bin".to_string(),
            active_gamefilter_fake: "quic_initial_4pda.to.bin".to_string(),
            dpi_desync_ttl: None,
        }
    }
}

/// Parse a simple `key=value` env-style config file.
pub fn load_config(file: &str) -> Result<RunConfig, String> {
    let content = fs::read_to_string(file).map_err(|e| format!("Cannot read config '{}': {}", file, e))?;

    let mut cfg = RunConfig::default();

    for line in content.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("interface=") {
            cfg.interface = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("strategy=") {
            cfg.strategy = val.trim().to_string();
        } else if line == "gamefiltertcp=true" {
            cfg.gamefilter_tcp = true;
        } else if line == "gamefilterudp=true" {
            cfg.gamefilter_udp = true;
        } else if line == "router_mode=true" {
            cfg.router_mode = true;
        } else if line == "router_mode=false" {
            cfg.router_mode = false;
        } else if let Some(val) = line.strip_prefix("backend=") {
            cfg.backend = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("active_discord_fake=") {
            cfg.active_discord_fake = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("active_gamefilter_fake=") {
            cfg.active_gamefilter_fake = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("dpi_desync_ttl=") {
            cfg.dpi_desync_ttl = val.trim().parse::<u8>().ok();
        }
    }

    Ok(cfg)
}

/// Return available network interfaces.
/// On Windows and macOS there is no `/sys/class/net`, so only "any" is returned.
pub fn get_interfaces() -> Vec<String> {
    #[allow(unused_mut)]
    let mut interfaces = vec!["any".to_string()];

    #[cfg(target_os = "linux")]
    if let Ok(entries) = fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                interfaces.push(name);
            }
        }
    }

    let _ = env::consts::OS; // keep `env` import used on all platforms
    interfaces
}

pub fn get_cache_dir() -> std::path::PathBuf {
    if let Ok(val) = env::var("ZAPRET_CACHE_DIR") {
        std::path::PathBuf::from(val)
    } else if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            parent.to_path_buf()
        } else {
            std::path::PathBuf::from(".")
        }
    } else {
        std::path::PathBuf::from(".")
    }
}

const CONFIG_FILENAME: &str = "conf.env";

const DEFAULT_CONFIG_LINES: &[&str] = &[
    "interface=any",
    "strategy=",
    "gamefiltertcp=false",
    "gamefilterudp=false",
    "router_mode=false",
    "backend=nftables",
    "active_discord_fake=quic_initial_steamcommunity_com.bin",
    "active_gamefilter_fake=quic_initial_4pda.to.bin",
    "dpi_desync_ttl=",
];

pub fn config_path() -> std::path::PathBuf {
    get_cache_dir().join(CONFIG_FILENAME)
}

pub fn save_config(cfg: &RunConfig) -> Result<(), String> {
    let path = config_path();
    let ttl = cfg.dpi_desync_ttl.map(|v| v.to_string()).unwrap_or_default();
    let content = format!(
        "interface={}\nstrategy={}\ngamefiltertcp={}\ngamefilterudp={}\nrouter_mode={}\nbackend={}\nactive_discord_fake={}\nactive_gamefilter_fake={}\ndpi_desync_ttl={}\n",
        cfg.interface, cfg.strategy, cfg.gamefilter_tcp, cfg.gamefilter_udp, cfg.router_mode, cfg.backend,
        cfg.active_discord_fake, cfg.active_gamefilter_fake, ttl,
    );
    fs::write(&path, &content).map_err(|e| format!("Cannot write config '{}': {}", path.display(), e))?;
    Ok(())
}

/// Load the fixed DPI TTL value, if any.
pub fn load_ttl() -> Option<u8> {
    load_config(&config_path().to_string_lossy())
        .ok()
        .and_then(|cfg| cfg.dpi_desync_ttl)
}

/// Persist the fixed DPI TTL value (None = off / autottl).
pub fn save_ttl(ttl: Option<u8>) -> Result<(), String> {
    let path = config_path();
    let mut cfg = load_config(&path.to_string_lossy()).unwrap_or_default();
    cfg.dpi_desync_ttl = ttl;
    save_config(&cfg)
}

pub fn save_tui_state(
    interface: &str,
    strategy: &str,
    tcp: bool,
    udp: bool,
    router_mode: bool,
    backend: &str,
) -> Result<(), String> {
    let path = config_path();
    let mut cfg = load_config(&path.to_string_lossy()).unwrap_or_default();
    cfg.interface = interface.to_string();
    cfg.strategy = strategy.to_string();
    cfg.gamefilter_tcp = tcp;
    cfg.gamefilter_udp = udp;
    cfg.router_mode = router_mode;
    cfg.backend = backend.to_string();
    save_config(&cfg)
}

pub fn ensure_default_config() -> Result<(), String> {
    let path = config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Cannot create config directory: {}", e))?;
        }
        save_config(&RunConfig::default())?;
    }
    validate_config()?;
    Ok(())
}

fn validate_config() -> Result<(), String> {
    let path = config_path();
    let mut content =
        fs::read_to_string(&path).map_err(|e| format!("Cannot read config '{}': {}", path.display(), e))?;

    let existing_keys: Vec<&str> = content
        .lines()
        .filter_map(|line| line.trim().split_once('=').map(|(k, _)| k.trim()))
        .collect();

    let mut missing = Vec::new();
    for default_line in DEFAULT_CONFIG_LINES {
        if let Some(key) = default_line.split_once('=').map(|(k, _)| k.trim()) {
            if !existing_keys.contains(&key) {
                missing.push(*default_line);
            }
        }
    }

    let mut updated = false;
    if !missing.is_empty() {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        for line in &missing {
            content.push_str(line);
            content.push('\n');
        }
        updated = true;
    }

    let defaults = [
        ("active_discord_fake=", "quic_initial_steamcommunity_com.bin"),
        ("active_gamefilter_fake=", "quic_initial_4pda.to.bin"),
    ];

    for (key, default_val) in &defaults {
        let needs_fix = content
            .lines()
            .any(|line| line.trim().strip_prefix(*key).is_some_and(|val| val.trim().is_empty()));
        if needs_fix {
            content = content.replace(*key, &format!("{}{}", key, default_val));
            updated = true;
        }
    }

    if updated {
        fs::write(&path, &content).map_err(|e| format!("Cannot write config '{}': {}", path.display(), e))?;
    }

    Ok(())
}

pub fn load_active_fakes() -> (String, String) {
    let cfg = load_config(&config_path().to_string_lossy()).unwrap_or_default();
    (cfg.active_discord_fake, cfg.active_gamefilter_fake)
}

pub fn save_active_fakes(discord: &str, game: &str) -> Result<(), String> {
    let path = config_path();
    let mut cfg = load_config(&path.to_string_lossy()).unwrap_or_default();
    cfg.active_discord_fake = discord.to_string();
    cfg.active_gamefilter_fake = game.to_string();
    save_config(&cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_router_mode() {
        let content = "interface=eth0\nstrategy=general\nrouter_mode=true\ngamefiltertcp=false\ngamefilterudp=true\n";
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_zapret_conf.env");
        let _ = fs::write(&temp_file, content);

        let cfg = load_config(&temp_file.to_string_lossy()).expect("failed to load config");
        assert_eq!(cfg.interface, "eth0");
        assert_eq!(cfg.strategy, "general");
        assert!(cfg.router_mode);
        assert!(!cfg.gamefilter_tcp);
        assert!(cfg.gamefilter_udp);

        let _ = fs::remove_file(&temp_file);
    }
}
