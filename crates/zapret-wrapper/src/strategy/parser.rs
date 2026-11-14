use regex::Regex;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static RE_LINE_CONTINUATION: OnceLock<Regex> = OnceLock::new();
static RE_WF_TCP: OnceLock<Regex> = OnceLock::new();
static RE_WF_UDP: OnceLock<Regex> = OnceLock::new();
static RE_FILTER: OnceLock<Regex> = OnceLock::new();

/// Parsed output from a strategy `.bat` script.
#[derive(Debug, Default)]
pub struct ParsedStrategy {
    /// TCP ports extracted from `--wf-tcp=<ports>`.
    pub tcp_ports: String,
    /// UDP ports extracted from `--wf-udp=<ports>`.
    pub udp_ports: String,
    /// Per-filter nfqws/winws argument strings.
    pub nfqws_params: Vec<String>,
}

/// Which game-filter protocols the user turned on.
///
/// TCP and UDP are independent switches, so the parser has to be able to strip
/// only the placeholder that is off. Keeping the whole port range here would
/// silently turn UDP on when only TCP was requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameFilterPorts {
    pub tcp: bool,
    pub udp: bool,
}

/// Expand one game-filter placeholder, or remove it together with the comma
/// that joins it to the surrounding port list.
fn expand_game_filter(content: String, placeholder: &str, enabled: bool) -> String {
    if enabled {
        return content.replace(placeholder, "1024-65535");
    }
    let mut content = content;
    for pattern in [format!(",{}", placeholder), format!("{},", placeholder)] {
        content = content.replace(&pattern, "");
    }
    content.replace(placeholder, "")
}

/// Parse a Zapret strategy `.bat` file into its constituent parts.
///
/// # Errors
/// Returns an error string when:
/// - The file does not exist.
/// - `--wf-tcp` or `--wf-udp` are missing or appear more than once.
pub fn parse_bat_file(file_path: &str, game_filter: Option<&GameFilterPorts>) -> Result<ParsedStrategy, String> {
    if !Path::new(file_path).exists() {
        return Err(format!("Strategy file not found: {}", file_path));
    }

    let raw = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut content = raw.replace('\r', "");

    let re_continuation = RE_LINE_CONTINUATION.get_or_init(|| Regex::new(r"\^\s*\n").unwrap());
    content = re_continuation.replace_all(&content, "\n").to_string();

    // Replace static path placeholders.
    content = content.replace("%BIN%", "bin/");
    content = content.replace("%LISTS%", "lists/");

    // Resolve (or strip) game-filter port placeholders. TCP and UDP are
    // separate switches, so one being on must not fill in the other.
    let (tcp, udp) = game_filter.map(|gf| (gf.tcp, gf.udp)).unwrap_or((false, false));
    content = expand_game_filter(content, "%GameFilter%", tcp || udp);
    content = expand_game_filter(content, "%GameFilterTCP%", tcp);
    content = expand_game_filter(content, "%GameFilterUDP%", udp);

    // --- port extraction -------------------------------------------------
    let wf_tcp_re = RE_WF_TCP.get_or_init(|| Regex::new(r"--wf-tcp=([0-9,-]+)").unwrap());
    let wf_udp_re = RE_WF_UDP.get_or_init(|| Regex::new(r"--wf-udp=([0-9,-]+)").unwrap());

    let tcp_matches: Vec<_> = wf_tcp_re.find_iter(&content).collect();
    let udp_matches: Vec<_> = wf_udp_re.find_iter(&content).collect();

    if tcp_matches.is_empty() || udp_matches.is_empty() {
        return Err(format!("--wf-tcp or --wf-udp not found in '{}'", file_path));
    }
    if tcp_matches.len() > 1 {
        return Err(format!("Multiple --wf-tcp entries found in '{}'", file_path));
    }
    if udp_matches.len() > 1 {
        return Err(format!("Multiple --wf-udp entries found in '{}'", file_path));
    }

    let tcp_ports = wf_tcp_re.captures(&content).unwrap()[1].to_string();
    let udp_ports = wf_udp_re.captures(&content).unwrap()[1].to_string();

    // --- per-filter argument extraction ----------------------------------
    let filter_re =
        RE_FILTER.get_or_init(|| Regex::new(r"--filter-(tcp|udp)=([0-9,-]+)\s+([\s\S]*?--new|.*)").unwrap());

    let nfqws_params = filter_re
        .captures_iter(&content)
        .map(|caps| {
            let args = caps[0]
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .replace("=^!", "=!");
            args
        })
        .collect();

    Ok(ParsedStrategy {
        tcp_ports,
        udp_ports,
        nfqws_params,
    })
}

#[cfg(test)]
mod tests {
    use super::expand_game_filter;

    #[test]
    fn a_disabled_gamefilter_placeholder_leaves_a_clean_port_list() {
        let content = "--wf-tcp=80,443,%GameFilterTCP% --filter-tcp=80,443,%GameFilterTCP%".to_string();

        let tcp_only = expand_game_filter(content.clone(), "%GameFilterTCP%", true);
        assert_eq!(tcp_only, "--wf-tcp=80,443,1024-65535 --filter-tcp=80,443,1024-65535");

        let udp_only = expand_game_filter(content, "%GameFilterTCP%", false);
        assert_eq!(udp_only, "--wf-tcp=80,443 --filter-tcp=80,443");
    }
}
