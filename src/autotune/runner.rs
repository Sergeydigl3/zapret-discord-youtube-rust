use std::path::Path;
use std::time::Duration;

use crate::firewalls::FirewallBackend;

use super::domain_checks::{check_domain, test_http, test_quic, test_tls};
use super::net_checks::run_network_checks;
use super::presets::PRESETS;
use super::storage::{
    get_domains_for_preset, restore_ipset, save_ipset, save_results_file, set_ipset_any,
};
use super::types::{
    AutotuneConfig, AutotuneResults, CheckStatus, DomainCheckResult, DomainProtocolCheck,
    PresetResult, StrategyCheckResult,
};

fn get_strategy_name(name: &str) -> String {
    name.trim_end_matches(".bat").to_string()
}

fn strategy_dir() -> String {
    std::env::var("REPO_DIR").unwrap_or_else(|_| {
        crate::config::get_cache_dir()
            .join("zapret-discord-youtube-linux")
            .to_string_lossy()
            .into_owned()
    })
}

fn load_strategy_files(indices: &[usize], all_strategies: &[String]) -> Vec<(String, String)> {
    if indices.is_empty() {
        return Vec::new();
    }
    let repo = strategy_dir();
    let mut result = Vec::new();
    for &idx in indices {
        if idx >= all_strategies.len() {
            continue;
        }
        let name = &all_strategies[idx];
        let path = Path::new(&repo).join("custom-strategies").join(name);
        let path = if path.exists() {
            path
        } else {
            Path::new(&repo).join(name)
        };
        if !path.exists() {
            continue;
        }
        result.push((get_strategy_name(name), path.to_string_lossy().to_string()));
    }
    result
}

fn count_protocol_steps(config: &AutotuneConfig) -> usize {
    config.check_http as usize
        + config.check_tls12 as usize
        + config.check_tls13 as usize
        + config.check_quic as usize
}

fn wait_for_nfqws(timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    let mut running = false;
    while std::time::Instant::now() < deadline {
        if crate::runner::nfqws_process_running() || crate::platform::is_nfqws_running() {
            running = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    if running {
        // Let nfqws bind its nfqueue before probing.
        std::thread::sleep(Duration::from_millis(500));
    }
    running
}

fn domain_check_error() -> DomainCheckResult {
    DomainCheckResult {
        domain: String::new(),
        alive: CheckStatus::Error,
        http: CheckStatus::Error,
        tls12: CheckStatus::Error,
        tls13: CheckStatus::Error,
        quic: CheckStatus::Error,
        baseline_pass: false,
        detail: "check thread panicked".to_string(),
        http_count: 0,
        quic_count: 0,
    }
}

pub fn run_all(
    config: &AutotuneConfig,
    progress: &dyn Fn(usize, usize),
    backend: &dyn FirewallBackend,
    interface: &str,
) -> AutotuneResults {
    // Run network checks once (shared across all presets)
    let block_results = run_network_checks(&config.block_checks);
    let net_check_count = config.block_checks.count_enabled();

    let all_strategies = crate::strategy::get_strategies();
    let loaded = load_strategy_files(&config.strategy_indices, &all_strategies);
    let strat_count = loaded.len();

    let proto_steps = count_protocol_steps(config);

    // Calculate total steps: network checks + per-preset baseline domain checks
    let mut total = net_check_count;
    for &preset_idx in config.preset_indices.iter() {
        let domain_count = get_domains_for_preset(preset_idx).len();
        total += domain_count * (1 + proto_steps);
    }

    let mut done = 0;

    // === Network checks ===
    for _result in block_results.iter() {
        done += 1;
        progress(done, total);
    }

    let mut preset_results: Vec<PresetResult> = Vec::new();
    let mut all_working_strategy_names: Vec<std::collections::HashSet<String>> = Vec::new();

    // Save ipset once for all presets
    let saved_ipset = save_ipset();
    set_ipset_any();

    for &preset_idx in config.preset_indices.iter() {
        let domains = get_domains_for_preset(preset_idx);
        let preset_name = if preset_idx < PRESETS.len() {
            PRESETS[preset_idx].name.to_string()
        } else {
            "Custom".to_string()
        };

        println!(
            "\n--- {} [{}] ---",
            rust_i18n::t!("autotune_domain_results"),
            preset_name
        );

        // === Per-domain protocol checks (without any strategy) ===
        let mut domain_checks = Vec::with_capacity(domains.len());
        let mut handles: Vec<std::thread::JoinHandle<DomainCheckResult>> = Vec::new();
        for d in &domains {
            let cfg = config.clone();
            let d = d.clone();
            handles.push(std::thread::spawn(move || check_domain(&cfg, &d)));
        }
        for handle in handles {
            domain_checks.push(handle.join().unwrap_or_else(|_| domain_check_error()));
            done += 1 + proto_steps;
            progress(done, total);
        }

        // Determine which domains are blocked (baseline TLS 1.3 failed)
        let blocked_domains: Vec<String> = domain_checks
            .iter()
            .filter(|dc| !dc.baseline_pass)
            .map(|dc| dc.domain.clone())
            .collect();

        // Accurately add strategy testing steps for this preset to `total`
        let tested_count = if !loaded.is_empty() && !blocked_domains.is_empty() {
            blocked_domains.len()
        } else if !loaded.is_empty() {
            domains.len()
        } else {
            0
        };
        total += strat_count * (1 + tested_count);

        // === Strategy testing with real nfqws ===
        let mut strategy_results: Vec<StrategyCheckResult> = Vec::new();
        let mut working_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        if !loaded.is_empty() && !blocked_domains.is_empty() {
            for (strat_name, strat_path) in &loaded {
                println!("  {} {}", rust_i18n::t!("autotune_testing"), strat_name);

                let started = crate::runner::run_zapret_silent(strat_path, interface, false, false, backend);
                done += 1;
                progress(done, total);

                if let Err(e) = started {
                    println!("    {} {}: {}", rust_i18n::t!("status_failed"), strat_name, e);
                    strategy_results.push(StrategyCheckResult::failed(strat_name, &blocked_domains));
                    for _ in &blocked_domains {
                        done += 1;
                        progress(done, total);
                    }
                    continue;
                }

                let nfqws_alive = wait_for_nfqws(Duration::from_secs(3));

                if !nfqws_alive {
                    println!(
                        "    {} {} (nfqws exited early)",
                        rust_i18n::t!("status_failed"),
                        strat_name
                    );
                    strategy_results.push(StrategyCheckResult::failed(strat_name, &blocked_domains));
                    crate::runner::stop_zapret(backend);
                    for _ in &blocked_domains {
                        done += 1;
                        progress(done, total);
                    }
                    continue;
                }

                const PROTOCOLS: usize = 4; // http, tls12, tls13, quic
                let mut handles: Vec<(usize, usize, std::thread::JoinHandle<bool>)> = Vec::new();
                for (di, domain) in blocked_domains.iter().enumerate() {
                    for proto in 0..PROTOCOLS {
                        let d = domain.clone();
                        let n = config.num_requests;
                        handles.push((
                            di,
                            proto,
                            std::thread::spawn(move || match proto {
                                0 => test_http(&d, n),
                                1 => test_tls(&d, "--tlsv1.2", n),
                                2 => test_tls(&d, "--tlsv1.3", n),
                                _ => test_quic(&d, n),
                            }),
                        ));
                    }
                }

                let mut results = vec![false; blocked_domains.len() * PROTOCOLS];
                for (di, proto, handle) in handles {
                    results[di * PROTOCOLS + proto] = handle.join().unwrap_or(false);
                }

                let mut pass = Vec::new();
                let mut fail = Vec::new();
                let mut http_works = false;
                let mut tls12_works = false;
                let mut tls13_works = false;
                let mut quic_works = false;
                let mut dc_results = Vec::with_capacity(blocked_domains.len());
                for (di, domain) in blocked_domains.iter().enumerate() {
                    let http_ok = results[di * PROTOCOLS];
                    let tls12_ok = results[di * PROTOCOLS + 1];
                    let tls13_ok = results[di * PROTOCOLS + 2];
                    let quic_ok = results[di * PROTOCOLS + 3];
                    if http_ok {
                        http_works = true;
                    }
                    if tls12_ok {
                        tls12_works = true;
                    }
                    if tls13_ok {
                        tls13_works = true;
                    }
                    if quic_ok {
                        quic_works = true;
                    }

                    // Browsers use HTTPS; plain HTTP (port 80) is not enough.
                    let ok = tls12_ok || tls13_ok;
                    if ok {
                        pass.push(domain.clone());
                    } else {
                        fail.push(domain.clone());
                    }
                    dc_results.push(DomainProtocolCheck {
                        domain: domain.clone(),
                        http: http_ok,
                        tls12: tls12_ok,
                        tls13: tls13_ok,
                        quic: quic_ok,
                    });
                    done += 1;
                    progress(done, total);
                }

                let mut protocols_working = Vec::new();
                if http_works {
                    protocols_working.push("HTTP".to_string());
                }
                if tls12_works {
                    protocols_working.push("TLS12".to_string());
                }
                if tls13_works {
                    protocols_working.push("TLS13".to_string());
                }
                if quic_works {
                    protocols_working.push("QUIC".to_string());
                }

                crate::runner::stop_zapret(backend);

                let works = pass.len() >= blocked_domains.len() / 2;
                if works {
                    working_names.insert(strat_name.clone());
                }
                strategy_results.push(StrategyCheckResult {
                    strategy_name: strat_name.clone(),
                    domains_pass: pass,
                    domains_fail: fail,
                    works,
                    protocols_working,
                    domain_checks: dc_results,
                });
            }
        } else if !loaded.is_empty() && blocked_domains.is_empty() {
            for (strat_name, _) in &loaded {
                done += 1;
                progress(done, total);
                for _ in &domains {
                    done += 1;
                    progress(done, total);
                }
                working_names.insert(strat_name.clone());
                strategy_results.push(StrategyCheckResult {
                    strategy_name: strat_name.clone(),
                    domains_pass: domains.clone(),
                    domains_fail: Vec::new(),
                    works: true,
                    protocols_working: vec![
                        "HTTP".to_string(),
                        "TLS12".to_string(),
                        "TLS13".to_string(),
                        "QUIC".to_string(),
                    ],
                    domain_checks: domains
                        .iter()
                        .map(|d| DomainProtocolCheck {
                            domain: d.clone(),
                            http: true,
                            tls12: true,
                            tls13: true,
                            quic: true,
                        })
                        .collect(),
                });
            }
        }

        preset_results.push(PresetResult {
            preset_name,
            domain_checks,
            strategy_results,
        });
        all_working_strategy_names.push(working_names);
    }

    // Find common strategies (work across ALL presets)
    let common_strategies = if config.preset_indices.len() > 1 && !all_working_strategy_names.is_empty() {
        let mut common: std::collections::HashSet<String> = all_working_strategy_names[0].clone();
        for wm in &all_working_strategy_names[1..] {
            common.retain(|name| wm.contains(name));
        }
        let mut v: Vec<String> = common.into_iter().collect();
        v.sort();
        v
    } else {
        // Single preset: all working strategies are "common"
        preset_results
            .first()
            .map(|pr| {
                let mut names: Vec<String> = pr
                    .strategy_results
                    .iter()
                    .filter(|s| s.works)
                    .map(|s| s.strategy_name.clone())
                    .collect();
                names.sort();
                names
            })
            .unwrap_or_default()
    };

    // Restore original ipset
    if let Some(ref saved) = saved_ipset {
        restore_ipset(saved);
        println!("  {}", rust_i18n::t!("autotune_ipset_restored"));
    }

    let results = AutotuneResults {
        block_results,
        preset_results,
        common_strategies,
    };

    save_results_file(&results);
    results
}
