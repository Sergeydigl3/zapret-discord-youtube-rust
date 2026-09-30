//! Single owner of the two things that have to be true before the daemon can
//! start: it needs `CAP_NET_ADMIN`, and no daemon from a previous run may still
//! be holding the queue.
//!
//! Anything else that spawns a helper program opens its own `std::process::Command`;
//! there is no shared argument handling to factor out at this size.

use std::path::Path;
use std::process::Command;

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

/// Kill daemons left over from a previous run.
pub fn kill_stale_zapret() {
    #[cfg(target_os = "linux")]
    let _ = Command::new("pkill").arg("-9").arg("nfqws").output();

    #[cfg(target_os = "windows")]
    let _ = Command::new("taskkill").args(["/F", "/IM", "winws.exe"]).output();
}
