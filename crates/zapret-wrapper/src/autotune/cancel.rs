//! Cancellation of a running autotune sweep.
//!
//! The probes are in-process HTTP/TLS/QUIC calls, so there is no child process
//! to kill: a cancelled sweep stops as soon as the request in flight returns,
//! and every probe checks the flag before it starts another one.

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
}
