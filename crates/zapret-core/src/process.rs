//! Single owner of external process launching.
//!
//! Anything that spawns a helper program goes through this module so the
//! argument handling, the error contract and the platform split live in one
//! place instead of being repeated per call site.

use std::path::Path;
use std::process::Command;

/// Device that swallows output, spelled per platform.
pub fn null_device() -> &'static str {
    if cfg!(target_os = "windows") {
        "NUL"
    } else {
        "/dev/null"
    }
}

/// Grant `CAP_NET_ADMIN` to the daemon binary so it can use nfqueue.
///
/// Returns `true` when the capability is set, and on non-Linux platforms, where
/// the capability does not exist and nothing has to be done.
pub fn set_cap(bin_path: &Path) -> bool {
    #[cfg(target_os = "linux")]
    {
        Command::new("setcap")
            .args(["cap_net_admin+ep", &bin_path.to_string_lossy()])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = bin_path;
        true
    }
}

/// Same as [`set_cap`], but reports whether `setcap` could be started at all.
///
/// The downloader distinguishes "the command ran and said no" from "there is no
/// `setcap` on this machine" and stays quiet in the second case, which
/// [`set_cap`] deliberately collapses into `true` for the runner.
pub fn try_set_cap(bin_path: &Path) -> Option<bool> {
    #[cfg(target_os = "linux")]
    {
        Command::new("setcap")
            .args(["cap_net_admin+ep", &bin_path.to_string_lossy()])
            .output()
            .ok()
            .map(|o| o.status.success())
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = bin_path;
        None
    }
}

/// Kill daemons left over from a previous run.
pub fn kill_stale_zapret() {
    #[cfg(target_os = "linux")]
    let _ = Command::new("pkill").arg("-9").arg("nfqws").output();

    #[cfg(target_os = "windows")]
    let _ = Command::new("taskkill").args(["/F", "/IM", "winws.exe"]).output();
}
