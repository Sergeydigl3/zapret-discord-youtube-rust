//! Running the auto-tune sweep and printing its report.

use crossterm::event::Event;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::io::Write;
use std::sync::mpsc::Receiver;
use zapret_core::autotune::{self, cancel, CheckStatus, StrategyCheckResult};
use zapret_core::firewall::FirewallBackend;
use zapret_core::run;

use crate::event::{drain_events, wait_for_key};
use crate::screen::{begin_external_output, end_external_output};
use crate::state::AppState;

fn status_str(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pass => "✅",
        CheckStatus::Fail => "❌",
        CheckStatus::Skip => "⏩",
        CheckStatus::Error => "🚨",
    }
}

fn status_detail(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pass => "No blocking detected",
        CheckStatus::Fail => "Blocking detected",
        CheckStatus::Skip => "Skipped",
        CheckStatus::Error => "Error during check",
    }
}

pub fn run_autotune(
    app: &mut AppState,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rx: &Receiver<Event>,
) -> Result<(), io::Error> {
    app.autotune_running = true;
    app.autotune_results = None;

    begin_external_output(terminal)?;

    println!("{}", rust_i18n::t!("autotune_running"));
    println!();
    let config = &app.autotune_config;
    let interface = app
        .interfaces
        .get(app.selected_interface)
        .map(|s| s.as_str())
        .unwrap_or("any");
    #[cfg(target_os = "linux")]
    let backend: &dyn FirewallBackend = &app.selected_backend;
    #[cfg(target_os = "windows")]
    let backend: &dyn FirewallBackend = &zapret_core::firewall::windivert::WinDivertBackend;

    let start_time = std::time::Instant::now();
    drain_events(rx);

    let results = autotune::run_all(
        config,
        &|done, total| {
            let pct = done * 100 / total.max(1);
            let elapsed_sec = start_time.elapsed().as_secs();
            let mins = elapsed_sec / 60;
            let secs = elapsed_sec % 60;
            print!(
                "\r  {} {}/{} ({}%) [{:02}:{:02}]",
                rust_i18n::t!("autotune_progress"),
                done,
                total,
                pct,
                mins,
                secs
            );
            let _ = std::io::stdout().flush();

            while let Ok(event) = rx.try_recv() {
                if let crossterm::event::Event::Key(key) = event {
                    if key.code == crossterm::event::KeyCode::Char('q')
                        || key.code == crossterm::event::KeyCode::Char('Q')
                        || key.code == crossterm::event::KeyCode::Esc
                    {
                        cancel::trigger_cancel();
                        run::stop(backend);
                        return false; // Emergency stop requested!
                    }
                }
            }
            true
        },
        backend,
        interface,
    );
    println!();
    app.has_autotune_results_file = true;
    app.dpi_desync_ttl = zapret_core::config::load_ttl();

    let total_mins = results.elapsed_secs / 60;
    let total_secs = results.elapsed_secs % 60;
    println!();
    println!("{}", rust_i18n::t!("autotune_done"));
    println!(
        "⏱  {} {:02}:{:02}",
        rust_i18n::t!("autotune_time_elapsed"),
        total_mins,
        total_secs
    );
    println!();
    println!("{}", rust_i18n::t!("autotune_how_to_read"));
    println!();
    println!("--- {} ---", rust_i18n::t!("menu_autotune_net_checks"));
    let check_labels = ["DNS", "TCP RST", "SNI", "SIBERIAN", "QUIC", "CIDR"];
    for (label, check) in check_labels.iter().zip(&results.block_results) {
        println!(
            "  {}: {} - {}",
            label,
            status_str(&check.status),
            status_detail(&check.status)
        );
    }
    println!();

    for pr in &results.preset_results {
        println!(
            "--- {} [{}] ---",
            rust_i18n::t!("autotune_domain_results"),
            pr.preset_name
        );
        let req_count = pr
            .domain_checks
            .first()
            .map(|_| app.autotune_config.num_requests)
            .unwrap_or(3);
        for dc in &pr.domain_checks {
            println!(
                "  {}: alive={} HTTP:{}({}/{}) TLS1.2={} TLS1.3={} QUIC:{}({}/{}) baseline={}",
                dc.domain,
                status_str(&dc.alive),
                status_str(&dc.http),
                dc.http_count,
                req_count,
                status_str(&dc.tls12),
                status_str(&dc.tls13),
                status_str(&dc.quic),
                dc.quic_count,
                req_count,
                status_str(if dc.baseline_pass {
                    &CheckStatus::Pass
                } else {
                    &CheckStatus::Fail
                }),
            );
        }
        if !pr.strategy_results.is_empty() {
            println!();
            println!("  --- {} ---", rust_i18n::t!("autotune_strat_results"));
            for sr in &pr.strategy_results {
                let status = if sr.works { "✅ WORKS" } else { "❌ FAILS" };
                let protos = if sr.protocols_working.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", sr.protocols_working.join(", "))
                };
                println!(
                    "    {}: {} ({}/{} blocked domains unblocked){}",
                    sr.strategy_name,
                    status,
                    sr.score(),
                    sr.total(),
                    protos
                );
                for dc in &sr.domain_checks {
                    println!(
                        "      {} HTTP:{} T12:{} T13:{} Q:{}",
                        dc.domain,
                        if dc.http { "✅" } else { "❌" },
                        if dc.tls12 { "✅" } else { "❌" },
                        if dc.tls13 { "✅" } else { "❌" },
                        if dc.quic { "✅" } else { "❌" },
                    );
                }
            }
            let working: Vec<&StrategyCheckResult> = pr.strategy_results.iter().filter(|s| s.works).collect();
            if working.is_empty() {
                println!("    {}", rust_i18n::t!("autotune_strat_none_work"));
            } else {
                println!(
                    "    {} {} {}",
                    rust_i18n::t!("autotune_strat_works_count"),
                    working.len(),
                    rust_i18n::t!("autotune_strat_of_total").replace("{}", &pr.strategy_results.len().to_string())
                );
                for s in &working {
                    println!("      ✅ {} ({}/{})", s.strategy_name, s.score(), s.total());
                }
            }
        }
        println!();
    }

    if !results.common_strategies.is_empty() {
        println!(
            "--- {} ({}) ---",
            rust_i18n::t!("autotune_common_strats"),
            results.common_strategies.len()
        );
        for name in &results.common_strategies {
            println!("  ✅ {}", name);
        }
        println!();
    }
    println!("{}", rust_i18n::t!("msg_dl_key"));

    wait_for_key(rx)?;
    end_external_output(terminal, rx)?;

    app.autotune_results = Some(results);
    app.autotune_running = false;
    app.status_message = Some(rust_i18n::t!("autotune_done").into_owned());
    Ok(())
}
