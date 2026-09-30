use std::env;
use std::path::{Path, PathBuf};

/// Name of the downloaded strategy repository, present both as a cache entry
/// and next to the executable.
pub const REPO_DIR_NAME: &str = "zapret-discord-youtube-linux";

/// Root for everything the application writes and downloads.
///
/// `ZAPRET_CACHE_DIR` wins, then the directory of the running executable, then
/// the current directory.
pub fn cache_dir() -> PathBuf {
    if let Ok(val) = env::var("ZAPRET_CACHE_DIR") {
        PathBuf::from(val)
    } else if let Ok(exe_path) = env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            parent.to_path_buf()
        } else {
            PathBuf::from(".")
        }
    } else {
        PathBuf::from(".")
    }
}

/// Directory holding the downloaded strategy `.bat` files and the lists.
///
/// `REPO_DIR` wins, then `<cache>/zapret-discord-youtube-linux`.
pub fn repo_dir() -> PathBuf {
    PathBuf::from(
        env::var("REPO_DIR").unwrap_or_else(|_| cache_dir().join(REPO_DIR_NAME).to_string_lossy().into_owned()),
    )
}

/// Directory holding the list files (`ipset-all.txt` and friends).
///
/// Resolved strictly relative to the executable, with the current directory as
/// a dev-mode fallback. It deliberately does **not** consult `ZAPRET_CACHE_DIR`,
/// so `--cache-dir` never reaches the lists.
///
/// FIXME(zapret): `--cache-dir` should reach the lists as well. This resolver
/// exists to keep the current behaviour byte-for-byte while the duplicated path
/// logic is being collapsed into one place. Fixing it means calling `repo_dir()`
/// here, and that is a behaviour change, so it belongs in its own `fix(paths)`
/// commit rather than in the restructuring.
pub fn exe_relative_lists_dir() -> PathBuf {
    let exe_dir = env::current_exe()
        .map(|p| p.parent().unwrap().to_path_buf())
        .unwrap_or_else(|_| env::current_dir().unwrap_or_default());

    let base_dir = exe_dir.join(REPO_DIR_NAME);
    let lists_dir = base_dir.join("lists");

    if lists_dir.exists() && lists_dir.is_dir() {
        return lists_dir;
    }
    if base_dir.exists() && base_dir.is_dir() {
        return base_dir;
    }

    // Fallback to the current directory for dev mode.
    let local_base = Path::new(REPO_DIR_NAME);
    let local_lists = local_base.join("lists");
    if local_lists.exists() && local_lists.is_dir() {
        local_lists
    } else {
        local_base.to_path_buf()
    }
}

const CONFIG_FILENAME: &str = "conf.env";

/// Path of the runtime configuration file.
pub fn config_path() -> PathBuf {
    cache_dir().join(CONFIG_FILENAME)
}

/// Directory with the user-supplied strategy `.bat` files.
///
/// It lives in the cache directory rather than inside the downloaded repository
/// so user strategies survive a repository re-download.
pub fn custom_strategies_dir() -> PathBuf {
    cache_dir().join("custom-strategies")
}

/// Directory the runtime binary (`nfqws` / `winws.exe`) is installed into.
pub fn bin_runtime_dir() -> PathBuf {
    cache_dir().join("bin")
}

/// Path of the runtime binary for the current platform.
pub fn binary_path() -> PathBuf {
    let bin_name = if env::consts::OS == "windows" {
        "winws.exe"
    } else {
        "nfqws"
    };
    bin_runtime_dir().join(bin_name)
}

/// Directory with the `.bin` payload files referenced by the strategies.
///
/// Distinct from `bin_runtime_dir()` even though both are called `bin` on disk.
pub fn bin_assets_dir() -> PathBuf {
    repo_dir().join("bin")
}

/// Directory the nfqws daemon runs in.
pub fn repo_lists_dir() -> PathBuf {
    repo_dir().join("lists")
}

/// Directory for log files.
pub fn logs_dir() -> PathBuf {
    cache_dir().join("logs")
}

/// Scratch file that captures the daemon output while it starts up.
pub fn nfqws_output_log() -> PathBuf {
    logs_dir().join("nfqws_output.tmp")
}

/// True when the runtime binary is present.
pub fn nfqws_installed() -> bool {
    binary_path().exists()
}

/// True when at least one strategy `.bat` is available, either in the
/// repository root or in the custom-strategies directory.
pub fn strategies_installed() -> bool {
    let repo_dir = cache_dir().join(REPO_DIR_NAME);
    if !repo_dir.exists() {
        return false;
    }
    let mut has_bat = false;
    if let Ok(entries) = std::fs::read_dir(&repo_dir) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".bat") {
                    has_bat = true;
                    break;
                }
            }
        }
    }
    if !has_bat {
        if let Ok(entries) = std::fs::read_dir(repo_dir.join("custom-strategies")) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.ends_with(".bat") {
                        has_bat = true;
                        break;
                    }
                }
            }
        }
    }
    has_bat
}
