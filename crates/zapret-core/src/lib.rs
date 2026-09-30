//! Core domain logic of zapret-rust.
//!
//! Layers, top to bottom. A module may only depend on the layers above it in
//! this list; see `docs/architecture.md` for the full rules.
//!
//! ```text
//! L4  autotune
//! L3  net, run, service, diagnose
//! L2  download, firewall, lists, strategy
//! L1  config, defender, domains, fakes, platform
//! L0  error, i18n, paths, process
//! ```
//!
//! Within a layer there are no ordering rules, and cycles are forbidden.

// The macro joins this path onto CARGO_MANIFEST_DIR, so `../..` reaches the
// workspace-level locale directory shared with `zapret-tui` and the binary.
// It must run at the crate root: `t!` reaches the generated table through
// `crate::_rust_i18n_t`, which only exists at the root.
rust_i18n::i18n!("../../locales", fallback = "en");

// L0 — infrastructure with no dependencies of its own.
pub mod error;
pub mod i18n;
pub mod paths;
pub mod process;

// L1 — configuration, operating system access and the files they own.
pub mod config;
pub mod defender;
pub mod domains;
pub mod fakes;
pub mod platform;

// L2 — what the application installs and how it describes the network.
pub mod download;
pub mod firewall;
pub mod lists;
pub mod strategy;

// L3 — actions: probing, running, installing a service, diagnosing.
pub mod diagnose;
pub mod net;
pub mod run;
pub mod service;

// L4 — the auto-tuning feature, composed from the layers above.
pub mod autotune;
