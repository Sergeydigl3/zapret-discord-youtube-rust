//! Name-based process operations: grant the daemon `CAP_NET_ADMIN`, and make
//! sure no daemon from a previous run is still holding the queue.
//!
//! Callers ask for an operation (`set_cap`, `kill_stale_zapret`), never for a
//! `pkill` or `taskkill` line.

use std::path::Path;
use std::process::{Command, Stdio};

#[cfg(target_os = "windows")]
const DAEMON_IMAGE: &str = "winws.exe";
#[cfg(not(target_os = "windows"))]
const DAEMON_IMAGE: &str = "nfqws";

/// Image name of the zapret daemon on this platform.
pub fn daemon_image() -> &'static str {
    DAEMON_IMAGE
}

/// Full image name for a process, as the platform spells it. The caller passes
/// a bare name (`curl`); Windows needs the extension.
#[cfg(target_os = "windows")]
fn image_name(image: &str) -> String {
    if image.to_ascii_lowercase().ends_with(".exe") {
        image.to_string()
    } else {
        format!("{image}.exe")
    }
}

#[cfg(not(target_os = "windows"))]
fn image_name(image: &str) -> String {
    image.to_string()
}

/// True when at least one process with this image name is running.
pub fn is_process_running(image: &str) -> bool {
    running(&image_name(image))
}

#[cfg(target_os = "windows")]
fn running(image: &str) -> bool {
    let needle = image.to_lowercase();
    let filter = format!("IMAGENAME eq {image}");
    Command::new("tasklist")
        .args(["/FI", filter.as_str(), "/NH"])
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains(&needle))
}

#[cfg(not(target_os = "windows"))]
fn running(image: &str) -> bool {
    Command::new("pgrep")
        .arg("-x")
        .arg(image)
        .output()
        .is_ok_and(|o| o.status.success())
}

/// True when a zapret daemon is already running.
pub fn is_daemon_running() -> bool {
    is_process_running(DAEMON_IMAGE)
}

/// Force-kill every process with this image name, including its children.
///
/// Best-effort: a name that matches nothing is not an error, it just means there
/// was nothing to clean up.
pub fn kill_process(image: &str) -> bool {
    kill(&image_name(image))
}

#[cfg(target_os = "windows")]
fn kill(image: &str) -> bool {
    Command::new("taskkill")
        .args(["/F", "/T", "/IM", image])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(not(target_os = "windows"))]
fn kill(image: &str) -> bool {
    Command::new("pkill")
        .args(["-9", image])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Kill zapret daemons this program did not start. The daemon this program
/// started is stopped through its handle in [`crate::daemon`]; what is left is
/// the ones with no handle anywhere: a leftover, one started by hand, one
/// belonging to a managed service.
pub fn kill_stale_zapret() {
    let _ = kill_process(DAEMON_IMAGE);
}

/// Grant `CAP_NET_ADMIN` to the daemon binary so it can use nfqueue without
/// root. Returns `true` on the platforms that have no such capability and
/// nothing to do.
pub fn set_cap(bin_path: &Path) -> bool {
    try_set_cap(bin_path).unwrap_or(true)
}

/// Grant `CAP_NET_ADMIN`, reporting "the command ran and said no" separately
/// from "there is no `setcap` on this machine" — which is also the answer on
/// platforms that have no capability at all.
#[cfg(target_os = "linux")]
pub fn try_set_cap(bin_path: &Path) -> Option<bool> {
    Command::new("setcap")
        .args(["cap_net_admin+ep", &bin_path.to_string_lossy()])
        .output()
        .ok()
        .map(|o| o.status.success())
}

#[cfg(not(target_os = "linux"))]
pub fn try_set_cap(_bin_path: &Path) -> Option<bool> {
    None
}
