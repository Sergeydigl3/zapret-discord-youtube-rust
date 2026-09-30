//! Firewall rule management.
//!
//! `FirewallBackend` is the seam: the runner, the TTL sweep and autotune only
//! ever talk to it, so which firewall is in use stays their business.
//!
//! The backends also say what they are doing on stdout, which is fine for a
//! console run and ruinous inside a TUI frame. A caller that paints its own
//! progress — the autotune sweep — turns that off with [`set_quiet`].

#[cfg(target_os = "linux")]
pub mod backends;

#[cfg(target_os = "windows")]
pub mod windivert;

use std::sync::atomic::{AtomicBool, Ordering};

static QUIET: AtomicBool = AtomicBool::new(false);

/// Whether to keep the backends' progress notices to themselves.
pub fn is_quiet() -> bool {
    QUIET.load(Ordering::Relaxed)
}

/// Silence the backends' progress notices until this is called again.
///
/// A sweep repaints the whole screen on every step, so a notice printed between
/// two frames lands on top of the last one and is not erased until the next
/// step happens to come along.
pub fn set_quiet(quiet: bool) {
    QUIET.store(quiet, Ordering::Relaxed);
}

/// [`set_quiet`] for the length of a scope.
///
/// `QUIET` is process-wide, so a sweep has to put it back the way it found it
/// rather than assume it was the only thing running.
pub struct QuietGuard {
    was_quiet: bool,
}

impl QuietGuard {
    pub fn new() -> Self {
        let was_quiet = is_quiet();
        set_quiet(true);
        Self { was_quiet }
    }
}

impl Default for QuietGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for QuietGuard {
    fn drop(&mut self) {
        set_quiet(self.was_quiet);
    }
}

/// Report one line of progress, unless someone asked us not to.
pub(crate) fn notice(msg: &str) {
    if !is_quiet() {
        println!("{}", msg);
    }
}

/// The seam every runner and sweep goes through.
///
/// `Send + Sync` because a sweep runs on its own thread: the TUI's frame loop
/// must keep drawing while the sweep works, so the backend reference has to
/// cross a thread boundary. Every backend is a unit struct or a fieldless enum
/// with its state outside the process, so this costs nothing.
pub trait FirewallBackend: Send + Sync {
    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str) -> Result<(), String>;
    fn clear(&self) -> Result<(), String>;
}

#[cfg(target_os = "linux")]
pub use backends::LinuxBackend;
