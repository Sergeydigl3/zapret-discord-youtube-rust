use regex::Regex;
use std::fs;
use std::path::Path;

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

/// Options controlling how game-filter port placeholders are resolved.
#[derive(Debug, Clone)]
pub struct GameFilterPorts {
    pub ports: String,
    pub tcp_ports: String,
    pub udp_ports: String,
}

/// Parse a Zapret strategy `.bat` file into its constituent parts.
///
/// # Errors
/// Returns an error string when:
/// - The file does not exist.
/// - `--wf-tcp` or `--wf-udp` are missing or appear more than once.
pub fn parse_bat_file(
    file_path: &str,
    game_filter: Option<&GameFilterPorts>,
) -> Result<ParsedStrategy, String> {
    if !Path::new(file_path).exists() {
        return Err(format!("Strategy file not found: {}", file_path));
    }

    let raw = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut content = raw.replace('\r', "");

    // Replace static path placeholders.
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

    // --- port extraction -------------------------------------------------
    let wf_tcp_re = Regex::new(r"--wf-tcp=([0-9,-]+)").unwrap();
    let wf_udp_re = Regex::new(r"--wf-udp=([0-9,-]+)").unwrap();

    let tcp_matches: Vec<_> = wf_tcp_re.find_iter(&content).collect();
    let udp_matches: Vec<_> = wf_udp_re.find_iter(&content).collect();

    if tcp_matches.is_empty() || udp_matches.is_empty() {
        return Err(format!(
            "--wf-tcp or --wf-udp not found in '{}'",
            file_path
        ));
    }
    if tcp_matches.len() > 1 {
        return Err(format!(
            "Multiple --wf-tcp entries found in '{}'",
            file_path
        ));
    }
    if udp_matches.len() > 1 {
        return Err(format!(
            "Multiple --wf-udp entries found in '{}'",
            file_path
        ));
    }

    let tcp_ports = wf_tcp_re.captures(&content).unwrap()[1].to_string();
    let udp_ports = wf_udp_re.captures(&content).unwrap()[1].to_string();

    // --- per-filter argument extraction ----------------------------------
    let filter_re =
        Regex::new(r"--filter-(tcp|udp)=([0-9,-]+)\s+([\s\S]*?--new|.*)").unwrap();

    let nfqws_params = filter_re
        .captures_iter(&content)
        .map(|caps| {
            let args = caps[3]
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
