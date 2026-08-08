use std::io::Write;
use std::path::PathBuf;

use super::presets::{PRESETS, PRESET_FILES};
use super::types::{status_str_file, AutotuneResults, CheckStatus};

pub const CUSTOM_DOMAINS_FILE: &str = "autotune_custom.txt";
pub const RESULTS_FILE: &str = "autotune_results.txt";

pub fn save_results_file(results: &AutotuneResults) {
    let path = crate::config::get_cache_dir().join(RESULTS_FILE);
    let mut file = match std::fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            println!("  [save_results_file] Failed to create {}: {}", path.display(), e);
            return;
        }
    };
    println!(
        "  {}",
        rust_i18n::t!("autotune_saving_results").replace("{}", &path.display().to_string())
    );

    let check_names = ["DNS", "TCP RST", "SNI", "SIBERIAN", "QUIC", "CIDR"];
    let _ = writeln!(file, "--- {} ---", rust_i18n::t!("autotune_net_results"));
    for (name, check) in check_names.iter().zip(&results.block_results) {
        let _ = writeln!(file, "  {}: {}", name, status_str_file(&check.status));
    }
    let _ = writeln!(file);

    for pr in &results.preset_results {
        let _ = writeln!(
            file,
            "--- {} [{}] ---",
            rust_i18n::t!("autotune_domain_results"),
            pr.preset_name
        );
        for dc in &pr.domain_checks {
            let _ = writeln!(
                file,
                "  {}: alive={} HTTP:{}({}) TLS1.2:{} TLS1.3:{} QUIC:{}({}) baseline={}",
                dc.domain,
                status_str_file(&dc.alive),
                status_str_file(&dc.http),
                dc.http_count,
                status_str_file(&dc.tls12),
                status_str_file(&dc.tls13),
                status_str_file(&dc.quic),
                dc.quic_count,
                status_str_file(if dc.baseline_pass {
                    &CheckStatus::Pass
                } else {
                    &CheckStatus::Fail
                }),
            );
        }
        let _ = writeln!(file);
    }

    if !results.preset_results.is_empty() {
        let _ = writeln!(file, "--- {} ---", rust_i18n::t!("autotune_strat_results"));
        for pr in &results.preset_results {
            let _ = writeln!(file, "  [{}]", pr.preset_name);
            for sr in &pr.strategy_results {
                let s = if sr.works { "WORKS" } else { "FAILS" };
                let protos = if sr.protocols_working.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", sr.protocols_working.join(", "))
                };
                let _ = writeln!(
                    file,
                    "    {}: {} ({}/{}){}",
                    sr.strategy_name,
                    s,
                    sr.score(),
                    sr.total(),
                    protos
                );
                for dc in &sr.domain_checks {
                    let _ = writeln!(
                        file,
                        "      {} HTTP:{} T12:{} T13:{} Q:{}",
                        dc.domain,
                        if dc.http { "✅" } else { "❌" },
                        if dc.tls12 { "✅" } else { "❌" },
                        if dc.tls13 { "✅" } else { "❌" },
                        if dc.quic { "✅" } else { "❌" },
                    );
                }
            }
            let _ = writeln!(file);
        }

        if !results.common_strategies.is_empty() {
            let _ = writeln!(
                file,
                "--- {} ({}) ---",
                rust_i18n::t!("autotune_common_strats"),
                results.common_strategies.len()
            );
            for name in &results.common_strategies {
                let _ = writeln!(file, "  ✅ {}", name);
            }
            let _ = writeln!(file);
        }
    }
}

pub fn load_results_file() -> Option<String> {
    let path = crate::config::get_cache_dir().join(RESULTS_FILE);
    if path.exists() {
        std::fs::read_to_string(&path).ok()
    } else {
        None
    }
}

pub fn preset_domains_file_path(preset_idx: usize) -> PathBuf {
    let name = PRESET_FILES.get(preset_idx).copied().unwrap_or(CUSTOM_DOMAINS_FILE);
    crate::config::get_cache_dir().join(name)
}

/// Read a domain list file. One domain per line, lines starting with `#` are
/// treated as comments and skipped.
pub fn load_domain_file(path: &std::path::Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Domains for a preset come entirely from its text file. If the file is
/// missing or empty, the built-in defaults are used as a fallback.
pub fn get_domains_for_preset(preset_idx: usize) -> Vec<String> {
    if preset_idx >= PRESETS.len() {
        return Vec::new();
    }
    let file_domains = load_domain_file(&preset_domains_file_path(preset_idx));
    if !file_domains.is_empty() {
        return file_domains;
    }
    PRESETS[preset_idx].domains.iter().map(|s| s.to_string()).collect()
}

/// Create/refresh the per-preset domain list files (and the TTL list) with the
/// full built-in domain list so the user can add/remove domains freely.
pub fn ensure_domain_files() -> Result<(), String> {
    for (idx, preset) in PRESETS.iter().enumerate() {
        let path = preset_domains_file_path(idx);
        let header = rust_i18n::t!("domain_file_header_full").replace("{}", preset.name);
        ensure_domain_file(&path, &header, preset.domains)?;
    }
    crate::ttl::ensure_ttl_file()
}

fn ensure_domain_file(path: &std::path::Path, header: &str, defaults: &[&str]) -> Result<(), String> {
    if path.exists() && !load_domain_file(path).is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create directory '{}': {}", parent.display(), e))?;
    }
    let mut content = format!("# {}\n", header);
    for d in defaults {
        content.push_str(d);
        content.push('\n');
    }
    std::fs::write(path, content).map_err(|e| format!("Cannot write '{}': {}", path.display(), e))
}

pub fn save_ipset() -> Option<String> {
    let path = crate::ipset::get_ipset_all_path();
    std::fs::read_to_string(&path).ok()
}

pub fn restore_ipset(content: &str) {
    let _ = std::fs::write(crate::ipset::get_ipset_all_path(), content);
}

pub fn set_ipset_any() {
    let _ = std::fs::write(crate::ipset::get_ipset_all_path(), "");
    println!("  {}", rust_i18n::t!("autotune_ipset_any"));
}
