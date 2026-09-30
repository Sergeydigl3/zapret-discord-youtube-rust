//! Router mode: making this machine a gateway for another device.
//!
//! The firewall rules live in `zapret_core::firewall` and are tied to a run.
//! This module owns the part that is not: the kernel has to be told to forward,
//! and that has to survive a reboot, so it is a sysctl drop-in rather than a
//! runtime write alone.

use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

const SYSCTL_FILE: &str = "/etc/sysctl.d/99-zapret-router.conf";
const PROC: &str = "/proc/sys/net/ipv4/ip_forward";

/// "Was never read", as opposed to the forwarding that was there before.
const UNKNOWN: u8 = 2;

static PREV: AtomicU8 = AtomicU8::new(UNKNOWN);
static FILE_OWNED: AtomicBool = AtomicBool::new(false);

/// Whether router mode is switched on in the configuration file.
pub fn enabled() -> bool {
    crate::config::load_config(&crate::paths::config_path().to_string_lossy())
        .map(|cfg| cfg.router)
        .unwrap_or(false)
}

/// Start forwarding and leave it on across reboots.
///
/// Both writes are attempted even if the first one fails: forwarding now comes
/// from `/proc`, persistence across reboots from the drop-in, and a box with
/// no `/etc/sysctl.d` should still route.
pub fn enable() -> Result<(), String> {
    if PREV.load(Ordering::Relaxed) == UNKNOWN {
        PREV.store(read_proc().unwrap_or(UNKNOWN), Ordering::Relaxed);
    }

    let mut err = None;

    match fs::write(SYSCTL_FILE, "# zapret router mode\nnet.ipv4.ip_forward=1\n") {
        Ok(()) => FILE_OWNED.store(true, Ordering::Relaxed),
        Err(e) => err = Some(format!("{}: {}", SYSCTL_FILE, e)),
    }

    if let Err(e) = fs::write(PROC, "1") {
        err.get_or_insert_with(|| format!("{}: {}", PROC, e));
    }

    match err {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

/// Stop forwarding, putting back what was there before.
///
/// Best effort on every step: a machine that cannot be un-routed is an
/// inconvenience, not a reason to fail a stop.
pub fn disable() {
    if FILE_OWNED.swap(false, Ordering::Relaxed) {
        let _ = fs::remove_file(SYSCTL_FILE);
    }

    let prev = PREV.swap(UNKNOWN, Ordering::Relaxed);
    let _ = fs::write(PROC, if prev == 1 { "1" } else { "0" });
}

fn read_proc() -> Option<u8> {
    match fs::read_to_string(PROC) {
        Ok(v) => Some(v.trim().parse().unwrap_or(UNKNOWN)),
        Err(_) => None,
    }
}
