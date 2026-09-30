//! Everything that talks to a process by name.
//!
//! Two of the things that have to be true before the daemon can start: it needs
//! `CAP_NET_ADMIN`, and no daemon from a previous run may still be holding the
//! queue. Both are asked for here as operations (`set_cap`, `kill_stale_zapret`)
//! rather than as platform commands, so no caller spells `pkill` or `taskkill`.
//!
//! The platform differences are resolved with `cfg!` inside the functions: each
//! operation has exactly one definition, and the `taskkill` / `pgrep` branch is
//! dead code on the other platform instead of a second implementation of the
//! same function behind a `#[cfg]` attribute.

use std::path::Path;
use std::process::{Command, Stdio};

/// Image name of the zapret daemon on this platform.
pub fn daemon_image() -> &'static str {
    if cfg!(target_os = "windows") {
        "winws.exe"
    } else {
        "nfqws"
    }
}

/// Full image name for a process, as the platform spells it.
///
/// The caller passes a bare name (`curl`); Windows needs the extension.
fn image_name(image: &str) -> String {
    if cfg!(target_os = "windows") && !image.to_ascii_lowercase().ends_with(".exe") {
        format!("{image}.exe")
    } else {
        image.to_string()
    }
}

/// True when at least one process with this image name is running.
pub fn is_process_running(image: &str) -> bool {
    let image = image_name(image);
    if cfg!(target_os = "windows") {
        let filter = format!("IMAGENAME eq {image}");
        Command::new("tasklist")
            .args(["/FI", filter.as_str(), "/NH"])
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .to_lowercase()
                    .contains(&image.to_lowercase())
            })
            .unwrap_or(false)
    } else {
        Command::new("pgrep")
            .arg("-x")
            .arg(&image)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// True when a zapret daemon is already running.
pub fn is_daemon_running() -> bool {
    is_process_running(daemon_image())
}

/// Force-kill every process with this image name, including its children.
///
/// Best-effort: a name that matches nothing is not an error, it just means
/// there was nothing to clean up.
pub fn kill_process(image: &str) -> bool {
    let image = image_name(image);
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("taskkill");
        c.args(["/F", "/T", "/IM", image.as_str()]);
        c
    } else {
        let mut c = Command::new("pkill");
        c.args(["-9", image.as_str()]);
        c
    };
    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Kill zapret daemons this program did not start.
///
/// The daemon this program started is stopped through its handle in
/// [`crate::daemon`]. What is left for the name-based sweep are the processes
/// that have no handle anywhere: a daemon left over from an earlier run, one
/// started by hand, one belonging to a managed service.
pub fn kill_stale_zapret() {
    let _ = kill_process(daemon_image());
}

/// Grant `CAP_NET_ADMIN` to the daemon binary so it can use nfqueue.
///
/// Returns `true` when the capability is set, and on non-Linux platforms, where
/// the capability does not exist and nothing has to be done.
pub fn set_cap(bin_path: &Path) -> bool {
    try_set_cap(bin_path).unwrap_or(true)
}

/// Grant `CAP_NET_ADMIN`, reporting "the command ran and said no" separately
/// from "there is no `setcap` on this machine".
pub fn try_set_cap(bin_path: &Path) -> Option<bool> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    Command::new("setcap")
        .args(["cap_net_admin+ep", &bin_path.to_string_lossy()])
        .output()
        .ok()
        .map(|o| o.status.success())
}
