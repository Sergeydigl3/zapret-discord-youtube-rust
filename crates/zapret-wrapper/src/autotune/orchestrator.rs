use std::time::Duration;

use super::cancel::{is_cancelled, reset_cancel};
use super::progress::{LogLevel, Reporter, SweepEvent};
use crate::domains::{get_domains_for_preset, PRESETS};
use zapret_core::firewall::FirewallBackend;

use super::checks_domain::{check_domain, domain_check_error};
use super::checks_network::run_network_checks;
use super::probe::{test_http, test_quic, test_tls};
use super::storage::{restore_ipset, save_ipset, save_results_file, set_ipset_any};
use super::types::{
    AutotuneConfig, AutotuneResults, BlockCheckType, CheckStatus, DomainCheckResult, DomainProtocolCheck, PresetResult,
    StrategyCheckResult,
};

/// The strategies the sweep will test, as (display name, file name).
///
/// The file name is what goes into the run request; `strategy::resolve` decides
/// where it actually lives, exactly as it does for the ordinary run. Resolving
/// here as well used to be a second, subtly different copy of that rule, which
/// is why the sweep skipped `<cache>/custom-strategies`.
fn load_strategies(indices: &[usize], all_strategies: &[String]) -> Vec<(String, String)> {
    indices
        .iter()
        .filter_map(|&idx| all_strategies.get(idx))
        .filter(|name| crate::strategy::resolve(name).exists())
        .map(|name| (name.trim_end_matches(".bat").to_string(), name.clone()))
        .collect()
}

fn count_protocol_steps(config: &AutotuneConfig) -> usize {
    config.check_http as usize + config.check_tls12 as usize + config.check_tls13 as usize + config.check_quic as usize
}

/// Wait until the daemon the sweep just started is up.
///
/// The child handle is the answer: the sweep launched it, and nothing else can
/// be mistaken for it.
fn wait_for_nfqws(timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    let mut running = false;
    while std::time::Instant::now() < deadline {
        if is_cancelled() {
            return false;
        }
        if zapret_core::daemon::is_running() {
            running = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    if running && !is_cancelled() {
        // Let nfqws bind its nfqueue/WinDivert handle before probing.
        std::thread::sleep(Duration::from_millis(100));
    }
    running && !is_cancelled()
}

struct TtlGuard {
    original_ttl: Option<u8>,
}

impl Drop for TtlGuard {
    fn drop(&mut self) {
        let _ = crate::config::save_ttl(self.original_ttl);
    }
}

/// Keeps the firewall backends quiet for the length of the sweep.
///
/// The caller repaints the whole screen on every step, so a notice the backends
/// print between two frames lands on the last frame and is not erased until
/// some later step happens to come along.
type QuietGuard = zapret_core::firewall::QuietGuard;

/// Sweep strategy x domain x protocol and report every step through `on_event`.
///
/// The callback returns `false` to abort; whatever it has collected so far
/// comes back as a partial result, with the sweep's own cleanup (daemon, ipset,
/// TTL) already done.
pub fn run_all(
    config: &AutotuneConfig,
    on_event: &mut dyn FnMut(SweepEvent) -> bool,
    backend: &dyn FirewallBackend,
    #[cfg(target_os = "linux")] interface: &str,
) -> AutotuneResults {
    reset_cancel();
    let start_instant = std::time::Instant::now();
    // The sweep paints its own progress, so nothing underneath it may print.
    let _quiet_guard = QuietGuard::new();
    // Temporarily set TTL to auto (None) during autotune, restoring original TTL on exit
    let original_ttl = crate::config::load_ttl();
    let _ttl_guard = TtlGuard { original_ttl };
    let _ = crate::config::save_ttl(None);

    // Run network checks once (shared across all presets)
    on_event(SweepEvent::Phase(rust_i18n::t!("autotune_phase_net").into_owned()));
    let block_results = run_network_checks(&config.block_checks);
    let net_check_count = config.block_checks.count_enabled();

    let all_strategies = crate::strategy::get_strategies();
    let loaded = load_strategies(&config.strategy_indices, &all_strategies);
    let strat_count = loaded.len();

    let proto_steps = count_protocol_steps(config);

    // Calculate total steps upfront: network checks + per-preset baseline domain checks + strategy tests
    let mut total = net_check_count;
    for &preset_idx in config.preset_indices.iter() {
        let domain_count = get_domains_for_preset(preset_idx).len();
        let baseline_steps = domain_count * (1 + proto_steps);
        let strat_steps = if strat_count > 0 {
            strat_count * (1 + domain_count)
        } else {
            0
        };
        total += baseline_steps + strat_steps;
    }

    let mut report = Reporter::new(on_event, total);
    let mut done = 0;

    // Everything a cancelled sweep still owes the caller: the checks that
    // finished, the presets that finished, and how long it took.
    let partial = |preset_results: Vec<PresetResult>| AutotuneResults {
        block_results: block_results.clone(),
        preset_results,
        common_strategies: Vec::new(),
        elapsed_secs: start_instant.elapsed().as_secs(),
    };

    // === Network checks ===
    for _result in block_results.iter() {
        done += 1;
        if !report.step(done) {
            report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
            return partial(Vec::new());
        }
    }

    // What the network is doing, in one line: the log is empty for a minute
    // otherwise, and this is the first thing the result depends on.
    let detected: Vec<&str> = BlockCheckType::all()
        .iter()
        .zip(&block_results)
        .filter(|(_, r)| r.status == CheckStatus::Fail)
        .map(|(kind, _)| kind.name())
        .collect();
    if detected.is_empty() {
        report.log(LogLevel::Good, rust_i18n::t!("autotune_log_net_clean"));
    } else {
        report.log(
            LogLevel::Bad,
            rust_i18n::t!("autotune_log_net_blocked").replace("{}", &detected.join(", ")),
        );
    }

    let mut preset_results: Vec<PresetResult> = Vec::new();
    let mut all_working_strategy_names: Vec<std::collections::HashSet<String>> = Vec::new();

    // Save ipset once for all presets
    let saved_ipset = save_ipset();
    set_ipset_any();
    report.log(LogLevel::Info, rust_i18n::t!("autotune_ipset_any"));

    for &preset_idx in config.preset_indices.iter() {
        let domains = get_domains_for_preset(preset_idx);
        let preset_name = if preset_idx < PRESETS.len() {
            PRESETS[preset_idx].name.to_string()
        } else {
            "Custom".to_string()
        };

        report.phase(rust_i18n::t!("autotune_phase_baseline").replace("{}", &preset_name));

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
            if !report.step(done) {
                report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
                if let Some(ref saved) = saved_ipset {
                    restore_ipset(saved);
                }
                return partial(preset_results);
            }
        }

        // Determine which domains are blocked (baseline TLS 1.3 failed)
        let blocked_domains: Vec<String> = domain_checks
            .iter()
            .filter(|dc| !dc.baseline_pass)
            .map(|dc| dc.domain.clone())
            .collect();

        report.log(
            if blocked_domains.is_empty() {
                LogLevel::Good
            } else {
                LogLevel::Info
            },
            rust_i18n::t!("autotune_log_preset")
                .replace("{name}", &preset_name)
                .replace("{blocked}", &blocked_domains.len().to_string())
                .replace("{total}", &domains.len().to_string()),
        );

        // === Strategy testing with real nfqws ===
        let mut strategy_results: Vec<StrategyCheckResult> = Vec::new();
        let mut working_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        if !loaded.is_empty() && !blocked_domains.is_empty() {
            for (strat_name, strat_file) in &loaded {
                report.phase(rust_i18n::t!("autotune_testing").replace("{}", strat_name));

                let started = {
                    let req = crate::plan::RunRequest::new(strat_file, false, false);
                    #[cfg(target_os = "linux")]
                    let req = req.with_interface(interface);
                    crate::run::run_quiet(&req, backend, &crate::paths::nfqws_output_log())
                };
                done += 1;
                if !report.step(done) {
                    report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
                    crate::run::stop_quiet(backend);
                    if let Some(ref saved) = saved_ipset {
                        restore_ipset(saved);
                    }
                    return partial(preset_results);
                }

                if let Err(e) = started {
                    report.log(
                        LogLevel::Bad,
                        format!(
                            "{}: {}",
                            strat_name,
                            rust_i18n::t!("autotune_log_start_failed").replace("{}", &e)
                        ),
                    );
                    strategy_results.push(StrategyCheckResult::failed(strat_name, &blocked_domains));
                    for _ in 0..domains.len() {
                        done += 1;
                        if !report.step(done) {
                            report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
                            if let Some(ref saved) = saved_ipset {
                                restore_ipset(saved);
                            }
                            return partial(preset_results);
                        }
                    }
                    continue;
                }

                let nfqws_alive = wait_for_nfqws(Duration::from_secs(3));

                if !nfqws_alive {
                    report.log(
                        LogLevel::Bad,
                        format!("{}: {}", strat_name, rust_i18n::t!("autotune_log_nfqws_early")),
                    );
                    strategy_results.push(StrategyCheckResult::failed(strat_name, &blocked_domains));
                    crate::run::stop_quiet(backend);
                    for _ in 0..domains.len() {
                        done += 1;
                        if !report.step(done) {
                            report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
                            if let Some(ref saved) = saved_ipset {
                                restore_ipset(saved);
                            }
                            return partial(preset_results);
                        }
                    }
                    continue;
                }

                const PROTOCOLS: usize = 4; // http, tls12, tls13, quic
                let mut results = vec![false; blocked_domains.len() * PROTOCOLS];
                let n = config.num_requests;

                std::thread::scope(|s| {
                    let mut handles = Vec::with_capacity(blocked_domains.len() * PROTOCOLS);
                    for (di, domain) in blocked_domains.iter().enumerate() {
                        for proto in 0..PROTOCOLS {
                            handles.push((
                                di * PROTOCOLS + proto,
                                s.spawn(move || match proto {
                                    0 => test_http(domain, n),
                                    1 => test_tls(domain, "--tlsv1.2", n),
                                    2 => test_tls(domain, "--tlsv1.3", n),
                                    _ => test_quic(domain, n),
                                }),
                            ));
                        }
                    }
                    for (idx, handle) in handles {
                        results[idx] = handle.join().unwrap_or(false);
                    }
                });

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
                    if quic_works {
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
                    if !report.step(done) {
                        report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
                        crate::run::stop_quiet(backend);
                        if let Some(ref saved) = saved_ipset {
                            restore_ipset(saved);
                        }
                        return partial(preset_results);
                    }
                }

                // Credit steps for domains that passed baseline (unblocked)
                let unblocked_count = domains.len().saturating_sub(blocked_domains.len());
                for _ in 0..unblocked_count {
                    done += 1;
                    if !report.step(done) {
                        report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
                        crate::run::stop_quiet(backend);
                        if let Some(ref saved) = saved_ipset {
                            restore_ipset(saved);
                        }
                        return partial(preset_results);
                    }
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

                crate::run::stop_quiet(backend);

                let works = pass.len() >= blocked_domains.len() / 2;
                if works {
                    working_names.insert(strat_name.clone());
                }

                // One line per strategy is the heartbeat of a sweep that can run
                // for ten minutes, and it is the only place the per-strategy
                // verdict shows up before the report opens. The name is left out
                // because the phase line right above it already said which
                // strategy this is.
                let over = if protocols_working.is_empty() {
                    rust_i18n::t!("atv_none").to_string()
                } else {
                    protocols_working.join(" ")
                };
                let summary = rust_i18n::t!("autotune_log_strategy")
                    .replace("{score}", &pass.len().to_string())
                    .replace("{total}", &blocked_domains.len().to_string())
                    .replace("{}", &over);
                report.log(if works { LogLevel::Good } else { LogLevel::Bad }, summary);

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
            report.log(
                LogLevel::Good,
                rust_i18n::t!("autotune_nothing_blocked").replace("{}", &preset_name),
            );
            for (strat_name, _) in &loaded {
                done += 1;
                if !report.step(done) {
                    report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
                    if let Some(ref saved) = saved_ipset {
                        restore_ipset(saved);
                    }
                    return partial(preset_results);
                }
                for _ in &domains {
                    done += 1;
                    if !report.step(done) {
                        report.log(LogLevel::Bad, rust_i18n::t!("autotune_cancelled"));
                        if let Some(ref saved) = saved_ipset {
                            restore_ipset(saved);
                        }
                        return partial(preset_results);
                    }
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
        report.log(LogLevel::Good, rust_i18n::t!("autotune_ipset_restored"));
    }

    let results = AutotuneResults {
        block_results,
        preset_results,
        common_strategies,
        elapsed_secs: start_instant.elapsed().as_secs(),
    };

    match save_results_file(&results) {
        Ok(path) => report.log(
            LogLevel::Info,
            rust_i18n::t!("autotune_saving_results").replace("{}", &path),
        ),
        Err(e) => report.log(LogLevel::Bad, e),
    }
    results
}
