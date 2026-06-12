use std::env;
use std::fs;

/// Runtime configuration assembled from CLI flags or a config file.
#[derive(Debug)]
pub struct RunConfig {
    pub interface: String,
    pub strategy: String,
    pub gamefilter_tcp: bool,
    pub gamefilter_udp: bool,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            interface: "any".to_string(),
            strategy: String::new(),
            gamefilter_tcp: false,
            gamefilter_udp: false,
        }
    }
}

/// Parse a simple `key=value` env-style config file.
pub fn load_config(file: &str) -> Result<RunConfig, String> {
    let content =
        fs::read_to_string(file).map_err(|e| format!("Cannot read config '{}': {}", file, e))?;

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

pub fn config_path() -> std::path::PathBuf {
    get_cache_dir().join(CONFIG_FILENAME)
}

pub fn save_config(cfg: &RunConfig) -> Result<(), String> {
    let path = config_path();
    let content = format!(
        "interface={}\nstrategy={}\ngamefiltertcp={}\ngamefilterudp={}\n",
        cfg.interface, cfg.strategy, cfg.gamefilter_tcp, cfg.gamefilter_udp
    );
    fs::write(&path, &content).map_err(|e| format!("Cannot write config '{}': {}", path.display(), e))?;
    Ok(())
}

pub fn save_tui_state(interface: &str, strategy: &str, tcp: bool, udp: bool) -> Result<(), String> {
    save_config(&RunConfig {
        interface: interface.to_string(),
        strategy: strategy.to_string(),
        gamefilter_tcp: tcp,
        gamefilter_udp: udp,
    })
}

pub fn ensure_default_config() -> Result<(), String> {
    let path = config_path();
    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Cannot create config directory: {}", e))?;
        }
        save_config(&RunConfig::default())?;
    }
    Ok(())
}
