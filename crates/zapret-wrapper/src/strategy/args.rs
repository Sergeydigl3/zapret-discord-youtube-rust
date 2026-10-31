//! Turning a parsed strategy into a command line for the daemon.
//!
//! The platform head and the TTL override live here. The parser hands over the
//! strategy's own tokens with the capture ports taken out, and this module
//! decides what the daemon actually gets on this OS: Windows receives the
//! WinDivert capture filters and the Lua libraries, Linux receives the queue
//! number, the loop-prevention mark and the Lua libraries.

use super::parser::{GameFilterPorts, ParsedStrategy};
use crate::paths;

/// The arguments that come before the strategy's own parameters.
///
/// Both platforms need the Lua libraries: the whole desync engine of zapret2
/// lives in `zapret-lib.lua` and `zapret-antidpi.lua`, installed next to the
/// daemon binary by the downloader. Beyond that, Linux marks its packets and
/// picks the queue number the firewall diverts to; Windows gets the capture
/// filters rebuilt from the ports the parser extracted.
fn platform_head(parsed: &ParsedStrategy) -> Vec<String> {
    let mut head = if cfg!(target_os = "windows") {
        vec![
            format!("--wf-tcp-in={}", parsed.tcp_in),
            format!("--wf-tcp-out={}", parsed.tcp_out),
            format!("--wf-udp-in={}", parsed.udp_in),
            format!("--wf-udp-out={}", parsed.udp_out),
        ]
    } else {
        vec![
            // The mark the firewall rules skip; 0x40000000 is also nfqws2's
            // own default, pinned here so the rule and the daemon cannot drift.
            "--fwmark=0x40000000".to_string(),
            "--qnum=200".to_string(),
        ]
    };
    for script in ["zapret-lib.lua", "zapret-antidpi.lua"] {
        head.push(format!("--lua-init=@{}", paths::lua_dir().join(script).display()));
    }
    head
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

/// The TTL params of a `--lua-desync` token, as nfqws2 spells them.
///
/// `ip_autottl=delta,min-max` mirrors the zapret 1 flag
/// `--dpi-desync-autottl=delta:min-max`: a fixed lower bound, not a fixed
/// value — the desync starts at `ttl` and walks up to 20, so the sweep measures
/// the lowest hop count that survives without also giving up the room the
/// engine would otherwise have to adapt.
fn autottl_params(ttl: u8) -> [String; 2] {
    [
        format!("ip_autottl=1,{}-{}", ttl, 20),
        format!("ip6_autottl=1,{}-{}", ttl, 20),
    ]
}

/// The Lua params that set a packet TTL and would shadow ours.
const TTL_PARAM_NAMES: [&str; 4] = ["ip_ttl=", "ip6_ttl=", "ip_autottl=", "ip6_autottl="];

/// Rewrite one strategy token for a fixed TTL run.
///
/// Desync functions take their fooling options as `:param=value` segments of
/// the `--lua-desync` token, so a TTL override both strips the TTL params the
/// strategy pinned and appends ours — to *every* desync token, because each
/// one is an independent function instance. Tokens that are not desync calls
/// have nothing to pin a TTL with and pass through unchanged.
fn apply_ttl(token: &str, ttl: u8) -> String {
    let Some(call) = token.strip_prefix("--lua-desync=") else {
        return token.to_string();
    };
    let mut segments: Vec<String> = call
        .split(':')
        .filter(|s| !TTL_PARAM_NAMES.iter().any(|p| s.starts_with(p)))
        .map(str::to_string)
        .collect();
    segments.extend(autottl_params(ttl));
    format!("--lua-desync={}", segments.join(":"))
}

/// Render the daemon arguments for a parsed strategy.
pub fn build_args(parsed: &ParsedStrategy, ttl: Option<u8>) -> Vec<String> {
    let mut args = platform_head(parsed);
    for token in &parsed.args {
        args.push(match ttl {
            Some(t) => apply_ttl(token, t),
            None => token.clone(),
        });
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::parser::ParsedStrategy;

    fn parsed(args: &[&str]) -> ParsedStrategy {
        ParsedStrategy {
            tcp_in: "80,443".into(),
            tcp_out: "80,443".into(),
            udp_in: "443".into(),
            udp_out: "443".into(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// A fixed lower bound, not a fixed value: the sweep measures the lowest
    /// hop count that survives, and the engine keeps its own room to walk up.
    #[test]
    fn a_fixed_ttl_is_an_autottl_lower_bound() {
        let args = build_args(
            &parsed(&["--filter-udp=443 --new", "--lua-desync=fake:blob=fake_quic"]),
            Some(3),
        );
        assert!(args
            .iter()
            .any(|a| a == "--lua-desync=fake:blob=fake_quic:ip_autottl=1,3-20:ip6_autottl=1,3-20"));
        assert!(
            !args.iter().any(|a| a.contains("ip_ttl=")),
            "a pinned value would take away the room the engine adapts in: {args:?}"
        );
    }

    #[test]
    fn no_fixed_ttl_means_untouched_tokens() {
        let args = build_args(&parsed(&["--lua-desync=fake:blob=fake_quic:ip_autottl=2,1-20"]), None);
        assert_eq!(
            args.last().map(String::as_str),
            Some("--lua-desync=fake:blob=fake_quic:ip_autottl=2,1-20")
        );
    }

    /// A strategy that pins its own TTL must not shadow the one being tested.
    #[test]
    fn a_fixed_ttl_replaces_every_ttl_param_the_strategy_pinned() {
        let args = build_args(
            &parsed(&[
                "--lua-desync=fake:blob=fake_quic:ip_ttl=9:repeats=6",
                "--lua-desync=multidisorder:pos=2:ip6_autottl=2,1-20",
            ]),
            Some(6),
        );
        assert!(args
            .iter()
            .any(|a| a == "--lua-desync=fake:blob=fake_quic:repeats=6:ip_autottl=1,6-20:ip6_autottl=1,6-20"));
        assert!(args
            .iter()
            .any(|a| a == "--lua-desync=multidisorder:pos=2:ip_autottl=1,6-20:ip6_autottl=1,6-20"));
    }

    /// Tokens that are not desync calls carry no TTL and must survive verbatim.
    #[test]
    fn non_desync_tokens_pass_through() {
        let args = build_args(
            &parsed(&[
                "--new",
                "--blob=fake_quic:@bin/quic_initial_www_google_com.bin",
                "--hostlist=lists/list-general.txt",
            ]),
            Some(4),
        );
        assert!(args
            .iter()
            .any(|a| a == "--blob=fake_quic:@bin/quic_initial_www_google_com.bin"));
        assert!(args.iter().any(|a| a == "--hostlist=lists/list-general.txt"));
    }

    #[test]
    fn every_desync_token_receives_the_ttl() {
        let args = build_args(
            &parsed(&[
                "--lua-desync=fake:blob=fake_quic",
                "--new",
                "--lua-desync=multisplit:pos=2",
            ]),
            Some(5),
        );
        let count = args.iter().filter(|a| a.contains("ip_autottl=1,5-20")).count();
        assert_eq!(count, 2, "each desync instance is independent: {args:?}");
    }

    /// The linux head pins the mark the firewall rules skip and the queue the
    /// rules divert to; the two must not drift apart. Only meaningful where
    /// cfg!(target_os = "windows") is false.
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn the_linux_head_carries_mark_queue_and_lua() {
        let args = platform_head(&parsed(&[]));
        assert!(args.iter().any(|a| a == "--fwmark=0x40000000"));
        assert!(args.iter().any(|a| a == "--qnum=200"));
        assert_eq!(args.iter().filter(|a| a.starts_with("--lua-init=@")).count(), 2);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn the_windows_head_carries_the_capture_filters_and_lua() {
        let args = platform_head(&parsed(&[]));
        assert!(args.iter().any(|a| a == "--wf-tcp-in=80,443"));
        assert!(args.iter().any(|a| a == "--wf-udp-out=443"));
        assert_eq!(args.iter().filter(|a| a.starts_with("--lua-init=@")).count(), 2);
    }
}
