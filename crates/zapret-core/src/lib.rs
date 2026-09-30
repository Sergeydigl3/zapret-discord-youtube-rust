//! The launch kernel of zapret-rust.
//!
//! Two concerns, and nothing else: the firewall rules the daemon needs, and the
//! daemon process itself. Both are strategy-agnostic. Nothing in this crate
//! knows what a `.bat` file is, reads the configuration file, resolves a path,
//! prints a message or writes a log — the caller fills in a
//! [`daemon::LaunchPlan`] and turns the [`daemon::LaunchOutcome`] back into
//! whatever the interface needs.
//!
//! That is the whole point of the split. The knowledge of zapret's file formats
//! lives in `zapret-wrapper`, which sits on top of this crate; the compiler,
//! not a code review, keeps the two apart.
//!
//! Layers, top to bottom. A module may only depend on the layers above it in
//! this list; see `docs/architecture.md` for the full rules.
//!
//! ```text
//! L2  daemon
//! L1  firewall, process
//! L0  error
//! ```
//!
//! Within a layer there are no ordering rules, and cycles are forbidden.

// The macro joins this path onto CARGO_MANIFEST_DIR, so `../..` reaches the
// workspace-level locale directory shared with the other crates. It is here for
// the backend names the firewall macro generates out of the backend file names.
rust_i18n::i18n!("../../locales", fallback = "en");

// L0 — the crate's error contract in one place.
pub mod error;

// L1 — the two things that have to be true before the daemon can start, and the
// network description it runs behind.
pub mod firewall;
pub mod process;

// L2 — starting it and stopping it again.
pub mod daemon;
