//! Everything zapret-rust knows about zapret, wrapped around a launch kernel.
//!
//! `zapret-core` knows how to put firewall rules in place and start `nfqws`.
//! This crate knows what a *strategy* is: that a `.bat` file has to be parsed
//! into an argument vector, that `ACTIVE_DISCORD_UDP.bin` is an alias for
//! whichever payload the user picked, that a domain list is a text file next to
//! the binary, that a fixed DPI TTL has to be injected into every `--new` group.
//! It owns the paths, the configuration file, the platform, the init systems,
//! the launch log — and it turns all of that into the [`zapret_core::daemon::LaunchPlan`]
//! the kernel runs.
//!
//! Layers, top to bottom. A module may only depend on the layers above it in
//! this list; see `docs/architecture.md` for the full rules.
//!
//! ```text
//! L4  autotune
//! L3  run, plan, service, diagnose
//! L2  lists, strategy
//! L1  config, defender, domains, fakes, platform, router
//! L0  paths
//! ```
//!
//! Within a layer there are no ordering rules, and cycles are forbidden.

// The macro joins this path onto CARGO_MANIFEST_DIR, so `../..` reaches the
// workspace-level locale directory shared with the other crates. It must run at
// the crate root: `t!` reaches the generated table through
// `crate::_rust_i18n_t`, which only exists at the root.
rust_i18n::i18n!("../../locales", fallback = "en");

// L0 — every path in the program. Nothing below this line may exist.
pub mod paths;

// L1 — configuration, operating system access and the files they own.
pub mod config;
pub mod defender;
pub mod domains;
pub mod fakes;
pub mod platform;

#[cfg(target_os = "linux")]
pub mod router;

// L2 — how zapret describes the network on disk.
pub mod lists;
pub mod strategy;

// L3 — actions: running, installing a service, diagnosing.
pub mod diagnose;
pub mod plan;
pub mod run;
pub mod service;

// L4 — the auto-tuning feature, including its own probes. Self-contained:
// nothing outside it uses them, so they are not a layer of their own.
pub mod autotune;

/// Where the downloader should put what it fetches.
///
/// The layout is decided here, next to the rest of the path world, so that
/// `zapret_fetch` never has to know what a cache directory is.
pub fn install_targets() -> zapret_fetch::InstallTargets {
    zapret_fetch::InstallTargets {
        cache_dir: paths::cache_dir(),
        runtime_bin_dir: paths::bin_runtime_dir(),
        // Deliberately not `paths::repo_dir()`: strategies have always been
        // unpacked into `<cache>/<repo name>`, ignoring the REPO_DIR override.
        strategies_dir: paths::cache_dir().join(paths::REPO_DIR_NAME),
    }
}
