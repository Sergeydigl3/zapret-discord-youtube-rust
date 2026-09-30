//! Cancellation of a running autotune sweep.
//!
//! This lived inside the DTO module because the flag and the process control
//! were declared together. Killing probe processes is process management, so it
//! belongs next to the other process handling rather than among data types.

use std::sync::atomic::{AtomicBool, Ordering};

pub static CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn reset_cancel() {
    CANCELLED.store(false, Ordering::Relaxed);
}

pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::Relaxed)
}

pub fn trigger_cancel() {
    CANCELLED.store(true, Ordering::Relaxed);
    kill_active_curls();
}

/// Force-kill the probe processes a cancelled sweep left running.
pub fn kill_active_curls() {
    zapret_core::process::kill_process("curl");
}
