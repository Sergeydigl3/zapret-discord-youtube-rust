//! Parsing a strategy file into the daemon's argument vector.
//!
//! A zapret2 strategy is a plain text file holding the raw `winws2` / `nfqws2`
//! arguments: whitespace-separated tokens, `"` quotes glue tokens together, and
//! a line whose first non-blank character is `#` is a comment. The parser
//! expands the path placeholders, resolves the game-filter port placeholders,
//! and pulls the WinDivert capture ports out of the token stream — the firewall
//! needs them, and on Linux the daemon itself must not receive them.
//!
//! Everything else passes through untouched: profiles (`--new`), the
//! `--lua-desync` chains, `--blob` and payload filters are the strategy
//! author's business, not ours.

use std::fs;
use std::path::Path;

/// Parsed output from a strategy file.
#[derive(Debug, Default)]
pub struct ParsedStrategy {
    /// WinDivert TCP capture ports declared with `--wf-tcp-in=`.
    pub tcp_in: String,
    /// WinDivert TCP capture ports declared with `--wf-tcp-out=`; defaults to
    /// `tcp_in` when the strategy only declares the incoming side.
    pub tcp_out: String,
    /// WinDivert UDP capture ports declared with `--wf-udp-in=`.
    pub udp_in: String,
    /// WinDivert UDP capture ports declared with `--wf-udp-out=`; defaults to
    /// `udp_in` when the strategy only declares the incoming side.
    pub udp_out: String,
    /// The strategy's own argument tokens, capture ports removed.
    pub args: Vec<String>,
}

impl ParsedStrategy {
    /// TCP ports the firewall has to divert: the union of in and out.
    pub fn tcp_ports(&self) -> String {
        merge_ports(&self.tcp_in, &self.tcp_out)
    }

    /// UDP ports the firewall has to divert: the union of in and out.
    pub fn udp_ports(&self) -> String {
        merge_ports(&self.udp_in, &self.udp_out)
    }
}

/// Options controlling how game-filter port placeholders are resolved.
#[derive(Debug, Clone)]
pub struct GameFilterPorts {
    pub ports: String,
    pub tcp_ports: String,
    pub udp_ports: String,
}

/// Read and parse a strategy file.
///
/// # Errors
/// Returns an error string when the file does not exist or declares no
/// `--wf-tcp-in` / `--wf-udp-in` capture ports.
pub fn parse_strategy_file(file_path: &str, game_filter: Option<&GameFilterPorts>) -> Result<ParsedStrategy, String> {
    if !Path::new(file_path).exists() {
        return Err(format!("Strategy file not found: {}", file_path));
    }

    let raw = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    parse_strategy_text(&raw, game_filter).map_err(|e| format!("{}: {}", file_path, e))
}

/// Parse strategy text: placeholders, comments, tokens, capture ports.
pub(crate) fn parse_strategy_text(raw: &str, game_filter: Option<&GameFilterPorts>) -> Result<ParsedStrategy, String> {
    let mut content = raw.replace('\r', "");

    // Comment lines: the strategy file is a config, not a shell script, so a
    // `#` only introduces a comment where a token could start.
    content = content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    // Replace static path placeholders. They resolve relative to the daemon's
    // working directory (the strategy repository root).
    content = content.replace("%BIN%", "bin/");
    content = content.replace("%LISTS%", "lists/");

    // Resolve (or strip) game-filter port placeholders.
    if let Some(gf) = game_filter {
        content = content.replace("%GameFilter%", &gf.ports);
        content = content.replace("%GameFilterTCP%", &gf.tcp_ports);
        content = content.replace("%GameFilterUDP%", &gf.udp_ports);
    } else {
        for placeholder in &[
            ",%GameFilter%",
            "%GameFilter%,",
            ",%GameFilterTCP%",
            "%GameFilterTCP%,",
            ",%GameFilterUDP%",
            "%GameFilterUDP%,",
        ] {
            content = content.replace(placeholder, "");
        }
    }

    let mut tcp_in = String::new();
    let mut tcp_out = String::new();
    let mut udp_in = String::new();
    let mut udp_out = String::new();
    let mut args = Vec::new();

    for token in tokenize(&content) {
        match token.as_str() {
            t if t.starts_with("--wf-tcp-in=") => tcp_in = capture_ports(t, "--wf-tcp-in=")?,
            t if t.starts_with("--wf-tcp-out=") => tcp_out = capture_ports(t, "--wf-tcp-out=")?,
            t if t.starts_with("--wf-udp-in=") => udp_in = capture_ports(t, "--wf-udp-in=")?,
            t if t.starts_with("--wf-udp-out=") => udp_out = capture_ports(t, "--wf-udp-out=")?,
            // The spellings zapret 1 used carry no direction; zapret2 split
            // them. Neither the firewall nor the daemon can use them here.
            t if t.starts_with("--wf-tcp=") || t.starts_with("--wf-udp=") => {
                return Err(format!(
                    "'{}' is the zapret 1 spelling — declare capture ports as --wf-tcp-in/--wf-tcp-out/--wf-udp-in/--wf-udp-out",
                    t
                ));
            }
            t => args.push(t.to_string()),
        }
    }

    if tcp_in.is_empty() || udp_in.is_empty() {
        return Err("--wf-tcp-in and --wf-udp-in not found".to_string());
    }
    if tcp_out.is_empty() {
        tcp_out = tcp_in.clone();
    }
    if udp_out.is_empty() {
        udp_out = udp_in.clone();
    }

    Ok(ParsedStrategy {
        tcp_in,
        tcp_out,
        udp_in,
        udp_out,
        args,
    })
}

/// Split strategy text into argument tokens.
///
/// Whitespace separates tokens unless a `"` quote is open; the quote characters
/// themselves are dropped, so `--hostlist="my lists/x.txt"` becomes one token
/// with the space kept.
fn tokenize(content: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in content.chars() {
        match c {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// The port list of one capture-ports token.
fn capture_ports(token: &str, flag: &str) -> Result<String, String> {
    let value = token.strip_prefix(flag).unwrap_or_default();
    if value.is_empty() {
        return Err(format!("'{}' declares no ports", flag));
    }
    Ok(value.to_string())
}

/// Merge two comma-separated port lists, keeping the first occurrence of every
/// entry so `in` ports are not reordered behind `out` ones.
fn merge_ports(a: &str, b: &str) -> String {
    let mut merged: Vec<&str> = Vec::new();
    for part in a.split(',').chain(b.split(',')) {
        if !part.is_empty() && !merged.contains(&part) {
            merged.push(part);
        }
    }
    merged.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_split_on_whitespace_and_quotes_glue() {
        let tokens = tokenize("--filter-tcp=443 --hostlist=\"my lists/x.txt\" --new\n--lua-desync=fake:blob=fake_tls");
        assert_eq!(
            tokens,
            vec![
                "--filter-tcp=443",
                "--hostlist=my lists/x.txt",
                "--new",
                "--lua-desync=fake:blob=fake_tls",
            ]
        );
    }

    #[test]
    fn comment_lines_are_dropped() {
        let parsed = parse_strategy_text(
            "# the general strategy\n--wf-tcp-in=443 --wf-udp-in=443 --new\n# trailing note\n--lua-desync=fake",
            None,
        )
        .unwrap();
        assert_eq!(parsed.args, vec!["--new", "--lua-desync=fake"]);
    }

    #[test]
    fn capture_ports_default_out_to_in() {
        let parsed =
            parse_strategy_text("--wf-tcp-in=80,443 --wf-udp-in=443,50000-50100 --lua-desync=fake", None).unwrap();
        assert_eq!(parsed.tcp_in, "80,443");
        assert_eq!(parsed.tcp_out, "80,443");
        assert_eq!(parsed.udp_in, "443,50000-50100");
        assert_eq!(parsed.udp_out, "443,50000-50100");
    }

    #[test]
    fn declared_out_survives_and_firewall_gets_the_union() {
        let parsed = parse_strategy_text(
            "--wf-tcp-in=443 --wf-tcp-out=80,443 --wf-udp-in=443 --wf-udp-out=443,50000-50100",
            None,
        )
        .unwrap();
        assert_eq!(parsed.tcp_ports(), "443,80");
        assert_eq!(parsed.udp_ports(), "443,50000-50100");
    }

    #[test]
    fn capture_port_tokens_do_not_reach_the_daemon() {
        let parsed = parse_strategy_text("--wf-tcp-in=443 --wf-udp-in=443 --lua-desync=fake", None).unwrap();
        assert!(!parsed.args.iter().any(|t| t.starts_with("--wf-")));
    }

    #[test]
    fn missing_capture_ports_is_an_error() {
        assert!(parse_strategy_text("--lua-desync=fake", None).is_err());
        assert!(parse_strategy_text("--wf-tcp-in=443 --lua-desync=fake", None).is_err());
    }

    #[test]
    fn the_zapret1_port_spelling_is_rejected() {
        let err = parse_strategy_text("--wf-tcp=443 --wf-udp=443", None).unwrap_err();
        assert!(err.contains("zapret 1 spelling"), "{err}");
    }

    #[test]
    fn placeholders_expand_to_repo_relative_paths() {
        let parsed = parse_strategy_text(
            "--wf-tcp-in=443 --wf-udp-in=443 --hostlist=%LISTS%list-general.txt --blob=fake_quic:@%BIN%quic_initial_www_google_com.bin",
            None,
        )
        .unwrap();
        assert!(parsed.args.iter().any(|t| t == "--hostlist=lists/list-general.txt"));
        assert!(parsed
            .args
            .iter()
            .any(|t| t == "--blob=fake_quic:@bin/quic_initial_www_google_com.bin"));
    }

    #[test]
    fn gamefilter_on_expands_and_off_strips() {
        let gf = GameFilterPorts {
            ports: "1024-65535".into(),
            tcp_ports: "1024-65535".into(),
            udp_ports: "1024-65535".into(),
        };
        let on = parse_strategy_text(
            "--wf-tcp-in=443,%GameFilterTCP% --wf-udp-in=443,%GameFilterUDP%",
            Some(&gf),
        )
        .unwrap();
        assert_eq!(on.tcp_ports(), "443,1024-65535");

        let off = parse_strategy_text("--wf-tcp-in=443,%GameFilterTCP% --wf-udp-in=443,%GameFilterUDP%", None).unwrap();
        assert_eq!(off.tcp_ports(), "443");
        assert_eq!(off.udp_ports(), "443");
    }
}
