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

/// The two flags that pin the DPI TTL of the desync'd packets.
///
/// A fixed lower bound, not a fixed value: the desync is told to start at `ttl`
/// and walk up to 20, so the sweep measures the lowest hop count that survives
/// without also giving up the room winws would otherwise have to adapt.
///
/// A fixed value has to be written into every profile, and before the `--new`
/// that closes it — see [`build_args`].
fn push_ttl(args: &mut Vec<String>, ttl: u8) {
    args.push(format!("--dpi-desync-autottl=1:{}-{}", ttl, 20));
    args.push(format!("--dpi-desync-autottl6=1:{}-{}", ttl, 20));
}

/// Render the daemon arguments for a parsed strategy.
pub fn build_args(parsed: &ParsedStrategy, ttl: Option<u8>) -> Vec<String> {
    let mut args = platform_head(parsed);
    // Stripping only makes sense when a hop count is actually being asked for.
    // Without this guard an ordinary run would lose the strategy's own autottl
    // just because a fixed TTL was configured.
    let strip = ttl.is_some();

    // Each strategy group is a separate desync profile: `--new` finalizes the
    // current profile and starts a fresh one, so TTL options must be injected
    // into every group (not just at the end of the whole command line).
    for param in &parsed.nfqws_params {
        let mut injected = false;
        for p in param.split_whitespace() {
            let p = p.replace('"', "");
            if p.is_empty() || p == "^" {
                continue;
            }
            // A fixed TTL overrides any TTL/autottl settings baked into the
            // strategy file, otherwise they would shadow our values.
            if strip && (p.starts_with("--dpi-desync-ttl") || p.starts_with("--dpi-desync-autottl")) {
                continue;
            }
            // `--new` closes this profile and opens the next one, so a flag
            // written after it becomes an option of the *following* profile.
            // Injecting there shifted every fixed TTL one group along, leaving
            // the first profile — the one the probe domains actually ride on —
            // with the strategy's own TTL, and a sweep that then changed
            // nothing between iterations.
            if p == "--new" {
                if let Some(t) = ttl {
                    push_ttl(&mut args, t);
                    injected = true;
                }
            }
            args.push(p.to_string());
        }
        // The last profile of a strategy is not closed by `--new`; its TTL
        // still has to be written out or the last group is left unpinned.
        if let Some(t) = ttl {
            if !injected {
                push_ttl(&mut args, t);
            }
        }
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::parser::ParsedStrategy;

    fn parsed(groups: &[&str]) -> ParsedStrategy {
        ParsedStrategy {
            tcp_ports: "80,443".into(),
            udp_ports: "443".into(),
            nfqws_params: groups.iter().map(|g| g.to_string()).collect(),
        }
    }

    /// How many fixed-TTL flags each `--filter-` profile received, in order.
    fn ttl_per_profile(args: &[String]) -> Vec<usize> {
        let mut counts = Vec::new();
        for a in args {
            if a.starts_with("--filter-") {
                counts.push(0);
            }
            if a.starts_with("--dpi-desync-autottl=") {
                let last = counts.last_mut().expect("a TTL flag before any profile");
                *last += 1;
            }
        }
        counts
    }

    /// The TTL has to belong to the profile it was written into. `--new` closes
    /// a profile, so a flag after it silently becomes an option of the next one
    /// — which left the first profile unpinned and made the sweep measure
    /// nothing.
    #[test]
    fn the_fixed_ttl_lands_inside_its_own_profile() {
        let args = build_args(
            &parsed(&[
                "--filter-tcp=80,443 --dpi-desync=multisplit --new",
                "--filter-udp=443 --dpi-desync=fake --new",
                // A strategy's last profile is not closed by `--new`.
                "--filter-tcp=443 --hostlist-domains=discord.media --dpi-desync=multisplit",
            ]),
            Some(4),
        );

        assert_eq!(ttl_per_profile(&args), vec![1, 1, 1]);
        // Nothing may open a profile after the last TTL, or the flags would
        // have been attributed to it.
        assert_eq!(args.last().map(String::as_str), Some("--dpi-desync-autottl6=1:4-20"));
    }

    /// A fixed lower bound, not a fixed value: the sweep measures the lowest
    /// hop count that survives, and winws keeps its own room to walk up.
    #[test]
    fn a_fixed_ttl_is_a_lower_bound_not_a_fixed_value() {
        let args = build_args(&parsed(&["--filter-udp=443 --dpi-desync=fake --new"]), Some(3));
        assert!(args.iter().any(|a| a == "--dpi-desync-autottl=1:3-20"));
        assert!(args.iter().any(|a| a == "--dpi-desync-autottl6=1:3-20"));
        assert!(
            !args.iter().any(|a| a.starts_with("--dpi-desync-ttl=")),
            "a pinned value would take away the room winws adapts in: {args:?}"
        );
    }

    #[test]
    fn no_fixed_ttl_means_no_injected_flags() {
        let args = build_args(&parsed(&["--filter-tcp=80,443 --dpi-desync=multisplit --new"]), None);
        assert!(!args.iter().any(|a| a.starts_with("--dpi-desync-autottl")));
        assert!(!args.iter().any(|a| a.starts_with("--dpi-desync-ttl")));
    }

    /// A strategy that pins its own TTL must not shadow the one being tested.
    #[test]
    fn a_fixed_ttl_replaces_the_strategys_own() {
        let args = build_args(
            &parsed(&["--filter-udp=443 --dpi-desync-ttl=9 --dpi-desync-autottl=2 --new"]),
            Some(6),
        );
        assert!(!args.iter().any(|a| a == "--dpi-desync-ttl=9"));
        assert!(!args.iter().any(|a| a == "--dpi-desync-autottl=2"));
        assert!(args.iter().any(|a| a == "--dpi-desync-autottl=1:6-20"));
    }
}
