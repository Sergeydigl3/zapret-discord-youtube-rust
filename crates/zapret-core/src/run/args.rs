//! Turning a parsed strategy into a command line for the daemon.
//!
//! The path lookup, the game-filter defaults and the argument list are kept
//! together so the spawn side of the runner only deals with process lifecycle.

use crate::paths;
use crate::strategy::{GameFilterPorts, ParsedStrategy};
use std::path::{Path, PathBuf};

/// Resolve a strategy file name to the file that should actually be parsed.
///
/// A custom strategy in the cache directory wins over one shipped in the
/// repository, so user overrides survive a repository re-download.
pub(crate) fn strategy_file_path(repo_path: &Path, strategy_file: &str) -> PathBuf {
    let cache_custom = paths::custom_strategies_dir().join(strategy_file);
    if cache_custom.exists() {
        return cache_custom;
    }
    let repo_custom = repo_path.join("custom-strategies").join(strategy_file);
    if repo_custom.exists() {
        repo_custom
    } else {
        repo_path.join(strategy_file)
    }
}

/// Build the game filter ports block, or `None` when game filtering is off.
pub(crate) fn game_filter(use_tcp: bool, use_udp: bool) -> Option<GameFilterPorts> {
    if use_tcp || use_udp {
        Some(GameFilterPorts {
            ports: "1024-65535".to_string(),
            tcp_ports: "1024-65535".to_string(),
            udp_ports: "1024-65535".to_string(),
        })
    } else {
        None
    }
}

/// Render the daemon arguments for a parsed strategy.
pub(crate) fn build_args(parsed: &ParsedStrategy, ttl: Option<u8>) -> Vec<String> {
    #[cfg(target_os = "linux")]
    let mut args = vec!["--dpi-desync-fwmark=0x40000000".to_string(), "--qnum=200".to_string()];

    #[cfg(target_os = "windows")]
    let mut args = vec![
        format!("--wf-tcp={}", parsed.tcp_ports),
        format!("--wf-udp={}", parsed.udp_ports),
    ];

    // Each strategy group is a separate desync profile: `--new` finalizes the
    // current profile and starts a fresh one, so TTL options must be injected
    // into every group (not just at the end of the whole command line).
    for param in &parsed.nfqws_params {
        for p in param.split_whitespace() {
            let p = p.replace('"', "");
            if p.is_empty() || p == "^" {
                continue;
            }
            // A fixed TTL overrides any TTL/autottl settings baked into the
            // strategy file, otherwise they would shadow our values.
            if ttl.is_some() && (p.starts_with("--dpi-desync-ttl") || p.starts_with("--dpi-desync-autottl")) {
                continue;
            }
            args.push(p.to_string());
        }
        // Injected at the end of the group so they win over the strategy's own
        // params in case the filtering above missed anything (winws applies
        // the last occurrence of a parameter).
        if let Some(ttl) = ttl {
            args.push(format!("--dpi-desync-ttl={}", ttl));
            args.push(format!("--dpi-desync-ttl6={}", ttl));
            args.push("--dpi-desync-autottl=-".to_string());
            args.push("--dpi-desync-autottl6=-".to_string());
        }
    }
    args
}
