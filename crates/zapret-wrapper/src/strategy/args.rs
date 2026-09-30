//! Turning a parsed strategy into a command line for the daemon.
//!
//! The game-filter defaults and the argument list live together because they are
//! the two halves of the same conversion: the parser needs to know which
//! placeholders to expand, and the expansion needs the platform head.

use super::parser::{GameFilterPorts, ParsedStrategy};

/// The arguments that come before the strategy's own parameters.
///
/// Linux marks its packets and picks the queue number; Windows takes the
/// winDivert filter ports, which are therefore not part of the strategy at all.
fn platform_head(parsed: &ParsedStrategy) -> Vec<String> {
    if cfg!(target_os = "windows") {
        vec![
            format!("--wf-tcp={}", parsed.tcp_ports),
            format!("--wf-udp={}", parsed.udp_ports),
        ]
    } else {
        vec!["--dpi-desync-fwmark=0x40000000".to_string(), "--qnum=200".to_string()]
    }
}

/// Build the game filter ports block, or `None` when game filtering is off.
pub fn game_filter(use_tcp: bool, use_udp: bool) -> Option<GameFilterPorts> {
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
pub fn build_args(parsed: &ParsedStrategy, ttl: Option<u8>) -> Vec<String> {
    let mut args = platform_head(parsed);

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
