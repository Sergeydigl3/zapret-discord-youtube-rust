//! Command line shared by the generated service units and scripts.
//!
//! Every init system has to run the same binary with the same three paths, but
//! writes them in its own syntax (`ExecStart=`, `exec`, `command_args=` and so
//! on). The path conversion and the `--config ... --cache-dir ...` tail are the
//! part that is identical everywhere, so they are built here once and the
//! writers only supply the syntax around them.

use std::path::Path;

/// The three paths every generated unit or script has to carry.
pub(crate) struct DaemonPaths {
    pub exe: String,
    pub config: String,
    pub cache: String,
}

impl DaemonPaths {
    /// `--config <config> --cache-dir <cache>`, the argument tail that follows
    /// the executable.
    pub(crate) fn args(&self) -> String {
        format!("--config {} --cache-dir {}", self.config, self.cache)
    }

    /// `<exe> --config <config> --cache-dir <cache>`, for the init systems that
    /// put the executable and its arguments in a single command.
    pub(crate) fn command(&self) -> String {
        format!("{} {}", self.exe, self.args())
    }
}

/// Convert the executable, config and cache paths for embedding in a unit file.
pub(crate) fn daemon_paths(exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<DaemonPaths, String> {
    let exe_str = exe_path.to_str().ok_or(rust_i18n::t!("err_invalid_exe").into_owned())?;
    let config_str = config_path
        .to_str()
        .ok_or(rust_i18n::t!("err_invalid_cfg").into_owned())?;
    let cache_str = cache_dir
        .to_str()
        .ok_or(rust_i18n::t!("err_invalid_cache").into_owned())?;

    Ok(DaemonPaths {
        exe: exe_str.to_string(),
        config: config_str.to_string(),
        cache: cache_str.to_string(),
    })
}
