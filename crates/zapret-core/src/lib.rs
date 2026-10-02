//! The launch kernel of zapret-rust: the firewall rules the daemon needs, and
//! the daemon process itself. Both are strategy-agnostic — nothing here knows
//! what a `.bat` file is, reads the configuration file, resolves a path or
//! prints a message. The caller fills in a [`daemon::LaunchPlan`] and renders
//! the [`daemon::LaunchOutcome`].
//!
//! Layers, top to bottom; a module may only use the layers above it, and cycles
//! are forbidden (see `docs/architecture.md`):
//!
//! ```text
//! L2  daemon
//! L1  firewall, process
//! L0  error
//! ```

// The path is joined onto CARGO_MANIFEST_DIR, so `../..` reaches the
// workspace-level locale directory shared with the other crates.
rust_i18n::i18n!("../../locales", fallback = "en");

pub mod daemon;
pub mod error;
pub mod firewall;
pub mod process;
