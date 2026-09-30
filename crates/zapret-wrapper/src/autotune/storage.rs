//! Serialization of autotune results.
//!
//! The domain list files are *not* stored here: they are shared with the TTL
//! sweep and therefore live in `crate::domains`.

use std::io::Write;

use super::types::{status_str_file, AutotuneResults, CheckStatus};

pub const RESULTS_FILE: &str = "autotune_results.txt";

pub fn save_results_file(results: &AutotuneResults) {
    let path = crate::paths::cache_dir().join(RESULTS_FILE);
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

    let mins = results.elapsed_secs / 60;
    let secs = results.elapsed_secs % 60;
    let _ = writeln!(
        file,
        "⏱ {}: {:02}:{:02}\n",
        rust_i18n::t!("autotune_time_elapsed"),
        mins,
        secs
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
    let path = crate::paths::cache_dir().join(RESULTS_FILE);
    if path.exists() {
        std::fs::read_to_string(&path).ok()
    } else {
        None
    }
}

pub fn save_ipset() -> Option<String> {
    let path = crate::lists::get_ipset_all_path();
    std::fs::read_to_string(&path).ok()
}

pub fn restore_ipset(content: &str) {
    let _ = std::fs::write(crate::lists::get_ipset_all_path(), content);
}

pub fn set_ipset_any() {
    let _ = std::fs::write(crate::lists::get_ipset_all_path(), "");
    println!("  {}", rust_i18n::t!("autotune_ipset_any"));
}
