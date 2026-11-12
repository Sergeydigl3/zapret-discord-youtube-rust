//! Non-interactive equivalents of every TUI action.
//!
//! The TUI is still the default. When a flag asks for an action, this module
//! applies the persistent settings first and then performs that one action, so
//! a script can configure and run the program without ever drawing a frame.

use std::path::{Path, PathBuf};

use zapret_wrapper::autotune::{self, AutotuneConfig, BlockCheckType, BlockChecks, SweepEvent};
use zapret_wrapper::config::{self, RunConfig};
use zapret_wrapper::domains;
use zapret_wrapper::fakes::{self, FakeTarget};
use zapret_wrapper::lists::{self, IpsetMode};
use zapret_wrapper::paths;
use zapret_wrapper::service;

use crate::cli::{Action, Cli, DefenderAction, ServiceAction};

/// Execute the operation selected on the command line.
pub fn run(args: &Cli) -> Result<(), String> {
    let action = args.action()?;
    apply_settings(args)?;
    match action {
        Some(action) => execute(args, action),
        // Only setting flags were passed: persist them and exit.
        None => Ok(()),
    }
}

fn execute(args: &Cli, action: Action) -> Result<(), String> {
    match action {
        Action::Run => {
            // Reject a mistyped strategy before the firewall is touched, and
            // let a shorthand such as `discord` resolve to `discord.bat`.
            if let Some(strategy) = &args.strategy {
                resolve_strategy_name(strategy)?;
            }
            crate::app::run_headless(args);
            Ok(())
        }
        Action::DownloadZapret(version) => {
            let version = zapret_version(&version);
            zapret_fetch::install_dependencies(&zapret_wrapper::install_targets(), &version, "skip")?;
            println!("{}", rust_i18n::t!("msg_dl_zapret_ok"));
            Ok(())
        }
        Action::DownloadStrategies(version) => {
            let version = strategies_version(&version);
            zapret_fetch::install_dependencies(&zapret_wrapper::install_targets(), "skip", &version)?;
            println!("{}", rust_i18n::t!("msg_dl_strat_ok"));
            Ok(())
        }
        Action::DownloadDefaults => {
            zapret_fetch::install_dependencies(
                &zapret_wrapper::install_targets(),
                zapret_fetch::ZAPRET_REC_VER,
                "recommended",
            )?;
            println!("{}", rust_i18n::t!("msg_dl_all_ok"));
            Ok(())
        }
        #[cfg(target_os = "linux")]
        Action::ListInterfaces => {
            for interface in zapret_wrapper::platform::get_interfaces() {
                println!("{}", interface);
            }
            Ok(())
        }
        Action::ListStrategies => {
            for strategy in zapret_wrapper::strategy::get_strategies() {
                println!("{}", strategy);
            }
            Ok(())
        }
        Action::ListFakes => {
            for fake in fakes::load_fakes_state().available {
                println!("{}", fake.filename);
            }
            Ok(())
        }
        Action::ListLists => {
            for file in lists::get_lists_files() {
                println!("{}", file);
            }
            Ok(())
        }
        Action::ListDomains => {
            // Listing must stay read-only: the files are created by the TUI or
            // by --edit-domains, and a normal user may not be able to write the
            // cache directory.
            for (index, preset) in domains::PRESETS.iter().enumerate() {
                println!(
                    "{}\t{}",
                    preset.name,
                    domains::preset_domains_file_path(index).display()
                );
            }
            println!("TTL\t{}", domains::ttl::ttl_domains_file_path().display());
            Ok(())
        }
        Action::ListTagsZapret => {
            for tag in zapret_fetch::fetch_repo_tags(zapret_fetch::ZAPRET_REPO)? {
                println!("{}", tag);
            }
            Ok(())
        }
        Action::ListTagsStrategies => {
            for tag in zapret_fetch::fetch_repo_tags("Flowseal/zapret-discord-youtube")? {
                println!("{}", tag);
            }
            Ok(())
        }
        Action::ShowConfig => {
            let path = config_path(args);
            let content = std::fs::read_to_string(&path).map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    rust_i18n::t!("cli_err_config_missing")
                        .replace("{}", &path.display().to_string())
                        .to_string()
                } else {
                    format!("Cannot read config '{}': {}", path.display(), e)
                }
            })?;
            print!("{}", content);
            Ok(())
        }
        Action::Status => show_status(args),
        Action::ShowAutotuneResults => {
            let report =
                autotune::load_results_file().ok_or_else(|| rust_i18n::t!("cli_err_no_results").to_string())?;
            print!("{}", report);
            Ok(())
        }
        Action::EditList(file) => {
            let path = resolve_list_file(&file)?;
            zapret_tui::editor::open_editor(&path.to_string_lossy())
                .map_err(|e| format!("Cannot open editor: {}", e))?;
            Ok(())
        }
        Action::EditDomains(preset) => {
            domains::ensure_domain_files()?;
            let path = resolve_domain_file(&preset)?;
            zapret_tui::editor::open_editor(&path.to_string_lossy())
                .map_err(|e| format!("Cannot open editor: {}", e))?;
            Ok(())
        }
        Action::Autotune => run_autotune(args),
        Action::TtlAutopick => run_ttl_autopick(args),
        Action::Service(action) => run_service(args, action),
        Action::Defender(action) => run_defender(action),
    }
}

// ---------------------------------------------------------------------------
// Persistent settings
// ---------------------------------------------------------------------------

fn apply_settings(args: &Cli) -> Result<(), String> {
    if !args.has_settings() {
        return Ok(());
    }

    let path = config_path(args);
    ensure_config(&path, args.config.is_some())?;
    let mut cfg = load_config(&path)?;

    #[cfg(target_os = "linux")]
    if let Some(interface) = &args.set_interface {
        if !zapret_wrapper::platform::get_interfaces()
            .iter()
            .any(|i| i == interface)
        {
            return Err(rust_i18n::t!("cli_err_interface").replace("{}", interface).to_string());
        }
        cfg.interface = interface.clone();
    }

    if let Some(strategy) = &args.set_strategy {
        cfg.strategy = resolve_strategy_name(strategy)?;
    }

    if let Some(enabled) = args.set_gamefilter_tcp {
        cfg.gamefilter_tcp = enabled;
    }
    if let Some(enabled) = args.set_gamefilter_udp {
        cfg.gamefilter_udp = enabled;
    }

    #[cfg(target_os = "linux")]
    if let Some(backend) = &args.set_backend {
        cfg.backend = resolve_backend(backend)?;
    }

    #[cfg(target_os = "linux")]
    if let Some(enabled) = args.set_router {
        cfg.router = enabled;
    }

    if let Some(ttl) = &args.set_ttl {
        cfg.dpi_desync_ttl = parse_ttl(ttl)?;
    }

    if let Some(mode) = &args.set_ipset_mode {
        apply_ipset_mode(mode)?;
    }

    if let Some(file) = &args.set_fake_discord {
        set_fake(file, FakeTarget::DiscordUdp, &mut cfg)?;
    }
    if let Some(file) = &args.set_fake_gamefilter {
        set_fake(file, FakeTarget::GameUdp, &mut cfg)?;
    }

    config::save_config_to(&path, &cfg)
}

fn config_path(args: &Cli) -> PathBuf {
    args.config
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(paths::config_path)
}

fn ensure_config(path: &Path, explicit: bool) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    if explicit {
        return Err(rust_i18n::t!("cli_err_config_missing")
            .replace("{}", &path.display().to_string())
            .to_string());
    }
    config::ensure_default_config()
}

fn load_config(path: &Path) -> Result<RunConfig, String> {
    config::load_config(&path.to_string_lossy())
}

/// The configuration a one-shot action should use.
fn effective_config(args: &Cli) -> Result<RunConfig, String> {
    let path = config_path(args);
    ensure_config(&path, args.config.is_some())?;
    load_config(&path)
}

fn resolve_strategy_name(name: &str) -> Result<String, String> {
    let normalized = zapret_wrapper::strategy::normalize_name(name);
    if zapret_wrapper::strategy::resolve(&normalized).is_file() {
        Ok(normalized)
    } else {
        Err(rust_i18n::t!("cli_err_strategy").replace("{}", name).to_string())
    }
}

#[cfg(target_os = "linux")]
fn resolve_backend(name: &str) -> Result<String, String> {
    use zapret_core::firewall::LinuxBackend;

    if let Some(backend) = LinuxBackend::variants()
        .into_iter()
        .find(|b| b.to_config().eq_ignore_ascii_case(name))
    {
        return Ok(backend.to_config().to_string());
    }
    let names: Vec<&str> = LinuxBackend::variants().iter().map(|b| b.to_config()).collect();
    Err(rust_i18n::t!("cli_err_backend")
        .replace("{}", name)
        .replace("{valid}", &names.join(", ")))
}

fn parse_ttl(value: &str) -> Result<Option<u8>, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "off" | "auto" | "none" | "" => Ok(None),
        other => match other.parse::<u8>() {
            Ok(ttl) if (domains::ttl::TTL_MIN..=domains::ttl::TTL_MAX).contains(&ttl) => Ok(Some(ttl)),
            _ => Err(rust_i18n::t!("cli_err_ttl")
                .replace("{}", other)
                .replace("{min}", &domains::ttl::TTL_MIN.to_string())
                .replace("{max}", &domains::ttl::TTL_MAX.to_string())),
        },
    }
}

fn ipset_mode_list() -> String {
    IpsetMode::all()
        .iter()
        .map(|mode| mode.keyword())
        .collect::<Vec<_>>()
        .join(", ")
}

fn domain_preset_list() -> String {
    let mut names: Vec<String> = domains::PRESETS
        .iter()
        .map(|preset| preset.name.to_lowercase())
        .collect();
    names.push("ttl".to_string());
    names.join(", ")
}

fn protocol_list() -> String {
    autotune::types::PROTOCOL_NAMES.join(", ")
}

fn block_check_list() -> String {
    autotune::types::BlockCheckType::all()
        .iter()
        .map(|check| check.keyword())
        .collect::<Vec<_>>()
        .join(", ")
}

fn parse_ipset_mode(value: &str) -> Result<IpsetMode, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" | "off" => Ok(IpsetMode::None),
        "any" | "empty" => Ok(IpsetMode::Any),
        "loaded" | "load" => Ok(IpsetMode::Loaded),
        "custom" => Ok(IpsetMode::Custom),
        other => Err(rust_i18n::t!("cli_err_ipset")
            .replace("{}", other)
            .replace("{valid}", &ipset_mode_list())),
    }
}

fn apply_ipset_mode(value: &str) -> Result<(), String> {
    let requested = parse_ipset_mode(value)?;
    let available = lists::get_available_modes();
    if !available.contains(&requested) {
        return Err(rust_i18n::t!("cli_err_ipset_unavailable")
            .replace("{}", value)
            .to_string());
    }
    let current = lists::determine_current_mode();
    lists::apply_ipset_mode(current, requested);
    Ok(())
}

fn set_fake(file: &str, target: FakeTarget, cfg: &mut RunConfig) -> Result<(), String> {
    let state = fakes::load_fakes_state();
    let wanted = Path::new(file)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file.to_string());
    let source = state
        .available
        .iter()
        .find(|f| {
            f.filename.eq_ignore_ascii_case(&wanted)
                || f.display_name.eq_ignore_ascii_case(file)
                || f.filename.eq_ignore_ascii_case(file)
        })
        .ok_or_else(|| {
            let names: Vec<&str> = state.available.iter().map(|f| f.filename.as_str()).collect();
            rust_i18n::t!("cli_err_fake")
                .replace("{}", file)
                .replace("{valid}", &names.join(", "))
        })?;

    fakes::apply_active_fake(&state, &target, source)?;
    match target {
        FakeTarget::DiscordUdp => cfg.active_discord_fake = source.filename.clone(),
        FakeTarget::GameUdp => cfg.active_gamefilter_fake = source.filename.clone(),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Downloads and inspection
// ---------------------------------------------------------------------------

/// `recommended` stays symbolic for the strategies downloader; everything else
/// is a tag or a commit.
fn strategies_version(version: &str) -> String {
    match version.trim().to_ascii_lowercase().as_str() {
        "" | "recommended" | "rec" => "recommended".to_string(),
        "latest" => "latest".to_string(),
        _ => version.to_string(),
    }
}

fn zapret_version(version: &str) -> String {
    match version.trim().to_ascii_lowercase().as_str() {
        "" | "recommended" | "rec" => zapret_fetch::ZAPRET_REC_VER.to_string(),
        "latest" => "latest".to_string(),
        _ => version.to_string(),
    }
}

fn show_status(args: &Cli) -> Result<(), String> {
    let path = config_path(args);
    println!("config={}", path.display());
    println!("config_exists={}", path.exists());
    println!("nfqws_installed={}", paths::nfqws_installed());
    println!("strategies_installed={}", paths::strategies_installed());

    match service::get_detected_manager() {
        Some(manager) => {
            println!("service_detected=true");
            println!("service_installed={}", manager.is_installed());
            println!("service_active={}", manager.is_active());
        }
        None => println!("service_detected=false"),
    }

    let report_path = paths::cache_dir().join(autotune::RESULTS_FILE);
    println!(
        "autotune_results={}",
        if report_path.exists() {
            report_path.display().to_string()
        } else {
            "none".to_string()
        }
    );

    #[cfg(target_os = "windows")]
    {
        println!(
            "defender_excluded={:?}",
            zapret_wrapper::defender::check_defender_exclusion().ok()
        );
    }
    Ok(())
}

fn resolve_list_file(file: &str) -> Result<PathBuf, String> {
    let direct = PathBuf::from(file);
    if direct.is_file() {
        return Ok(direct);
    }

    let files = lists::get_lists_files();
    let wanted = Path::new(file)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| file.to_string());

    files
        .iter()
        .map(PathBuf::from)
        .find(|path| {
            path.to_string_lossy() == file
                || path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(&wanted))
        })
        .ok_or_else(|| rust_i18n::t!("cli_err_list_file").replace("{}", file).to_string())
}

fn resolve_domain_file(preset: &str) -> Result<PathBuf, String> {
    let lower = preset.trim().to_ascii_lowercase();
    if lower == "ttl" {
        return Ok(domains::ttl::ttl_domains_file_path());
    }
    if lower == "all" {
        return Err(rust_i18n::t!("cli_err_domain_file")
            .replace("{}", preset)
            .replace("{valid}", &domain_preset_list()));
    }

    if let Ok(index) = lower.parse::<usize>() {
        if index < domains::PRESETS.len() {
            return Ok(domains::preset_domains_file_path(index));
        }
    }

    for (index, candidate) in domains::PRESETS.iter().enumerate() {
        if candidate.name.eq_ignore_ascii_case(&lower) {
            return Ok(domains::preset_domains_file_path(index));
        }
    }
    Err(rust_i18n::t!("cli_err_domain_file")
        .replace("{}", preset)
        .replace("{valid}", &domain_preset_list()))
}

// ---------------------------------------------------------------------------
// Autotune and the TTL sweep
// ---------------------------------------------------------------------------

fn run_autotune(args: &Cli) -> Result<(), String> {
    if zapret_wrapper::run::queue_in_use() {
        return Err(rust_i18n::t!("autotune_err_nfqws_running").to_string());
    }
    if !paths::nfqws_installed() {
        return Err(rust_i18n::t!("msg_err_nfqws_missing").to_string());
    }
    if !paths::strategies_installed() {
        return Err(rust_i18n::t!("msg_err_strat_missing").to_string());
    }

    let cfg = effective_config(args)?;
    let autotune_config = build_autotune_config(args)?;

    #[cfg(target_os = "linux")]
    let interface = args.interface.clone().unwrap_or_else(|| cfg.interface.clone());
    #[cfg(target_os = "linux")]
    let backend = zapret_core::firewall::LinuxBackend::from_config(&cfg.backend);
    #[cfg(target_os = "windows")]
    let backend = zapret_core::firewall::windivert::WinDivertBackend;

    let mut on_event = |event: SweepEvent| -> bool {
        match event {
            SweepEvent::Phase(phase) => println!("== {} ==", phase),
            SweepEvent::Log { text, .. } => println!("{}", text),
            SweepEvent::Step { .. } => {}
        }
        true
    };

    #[cfg(target_os = "linux")]
    let results = autotune::run_all(&autotune_config, &mut on_event, &backend, &interface);
    #[cfg(not(target_os = "linux"))]
    let results = autotune::run_all(&autotune_config, &mut on_event, &backend);

    println!(
        "{}",
        rust_i18n::t!("cli_autotune_done").replace("{}", &results.elapsed_secs.to_string())
    );
    if !results.common_strategies.is_empty() {
        println!(
            "{}{}",
            rust_i18n::t!("autotune_common_strats"),
            results.common_strategies.join(", ")
        );
    }
    Ok(())
}

fn build_autotune_config(args: &Cli) -> Result<AutotuneConfig, String> {
    let mut cfg = AutotuneConfig::default();

    if let Some(presets) = &args.autotune_presets {
        cfg.preset_indices = parse_presets(presets)?;
    }

    if let Some(requests) = args.autotune_requests {
        if requests == 0 {
            return Err(rust_i18n::t!("cli_err_requests").to_string());
        }
        cfg.num_requests = requests;
    }

    let strategies = zapret_wrapper::strategy::get_strategies();
    cfg.strategy_indices = match &args.autotune_strategies {
        Some(value) => parse_strategies(value, &strategies)?,
        // A headless run without an explicit selection means "test what is
        // installed"; the TUI makes the user tick them one by one.
        None => (0..strategies.len()).collect(),
    };

    cfg.check_http = true;
    cfg.check_tls12 = true;
    cfg.check_tls13 = true;
    cfg.check_quic = true;
    if let Some(protocols) = &args.autotune_protocols {
        let (http, tls12, tls13, quic) = parse_protocols(protocols)?;
        cfg.check_http = http;
        cfg.check_tls12 = tls12;
        cfg.check_tls13 = tls13;
        cfg.check_quic = quic;
    }

    if let Some(checks) = &args.autotune_block_checks {
        cfg.block_checks = parse_block_checks(checks)?;
    }

    Ok(cfg)
}

fn parse_presets(value: &str) -> Result<Vec<usize>, String> {
    let mut indices = Vec::new();
    for token in split_list(value) {
        if token.eq_ignore_ascii_case("all") {
            return Ok((0..domains::PRESETS.len()).collect());
        }
        if let Ok(index) = token.parse::<usize>() {
            if index < domains::PRESETS.len() {
                indices.push(index);
                continue;
            }
        }
        match domains::PRESETS
            .iter()
            .position(|preset| preset.name.eq_ignore_ascii_case(&token))
        {
            Some(index) => indices.push(index),
            None => return Err(rust_i18n::t!("cli_err_presets").replace("{}", &token).to_string()),
        }
    }
    indices.sort_unstable();
    indices.dedup();
    if indices.is_empty() {
        return Err(rust_i18n::t!("cli_err_presets").replace("{}", value).to_string());
    }
    Ok(indices)
}

fn parse_strategies(value: &str, all: &[String]) -> Result<Vec<usize>, String> {
    let tokens = split_list(value);
    if tokens.is_empty() || (tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("none")) {
        return Ok(Vec::new());
    }
    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("all") {
        return Ok((0..all.len()).collect());
    }

    let mut indices = Vec::new();
    for token in tokens {
        if let Ok(index) = token.parse::<usize>() {
            if index < all.len() {
                indices.push(index);
                continue;
            }
        }
        match all.iter().position(|name| {
            name.eq_ignore_ascii_case(&token)
                || name.eq_ignore_ascii_case(&format!("{}.bat", token))
                || name
                    .strip_suffix(".bat")
                    .is_some_and(|stem| stem.eq_ignore_ascii_case(&token))
        }) {
            Some(index) => indices.push(index),
            None => return Err(rust_i18n::t!("cli_err_strategies").replace("{}", &token).to_string()),
        }
    }
    indices.sort_unstable();
    indices.dedup();
    Ok(indices)
}

fn parse_protocols(value: &str) -> Result<(bool, bool, bool, bool), String> {
    let tokens = split_list(value);
    if tokens.is_empty() || (tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("none")) {
        return Ok((false, false, false, false));
    }
    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("all") {
        return Ok((true, true, true, true));
    }

    let mut http = false;
    let mut tls12 = false;
    let mut tls13 = false;
    let mut quic = false;
    for token in tokens {
        match token.to_ascii_lowercase().as_str() {
            "http" => http = true,
            "tls12" | "tls1.2" | "tls-1.2" => tls12 = true,
            "tls13" | "tls1.3" | "tls-1.3" => tls13 = true,
            "quic" => quic = true,
            _ => {
                return Err(rust_i18n::t!("cli_err_protocols")
                    .replace("{}", &token)
                    .replace("{valid}", &protocol_list()))
            }
        }
    }
    Ok((http, tls12, tls13, quic))
}

fn parse_block_checks(value: &str) -> Result<BlockChecks, String> {
    let tokens = split_list(value);
    if tokens.is_empty() || (tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("none")) {
        return Ok(BlockChecks {
            enabled: vec![false; BlockCheckType::all().len()],
        });
    }
    if tokens.len() == 1 && tokens[0].eq_ignore_ascii_case("all") {
        return Ok(BlockChecks::all_enabled());
    }

    let mut enabled = vec![false; BlockCheckType::all().len()];
    for token in tokens {
        let index = match token.to_ascii_lowercase().as_str() {
            "dns" | "dns-spoof" | "dnsspoof" => 0,
            "tcp-rst" | "tcprst" | "rst" => 1,
            "sni" | "sni-block" => 2,
            "siberian" | "siberian-block" => 3,
            "quic" | "quic-block" => 4,
            "cidr" | "cidr-whitelist" => 5,
            _ => {
                return Err(rust_i18n::t!("cli_err_block_checks")
                    .replace("{}", &token)
                    .replace("{valid}", &block_check_list()))
            }
        };
        enabled[index] = true;
    }

    // Keep the order documented by `BlockCheckType::all`; the index mapping
    // above is that order.
    Ok(BlockChecks { enabled })
}

fn run_ttl_autopick(args: &Cli) -> Result<(), String> {
    if !paths::nfqws_installed() {
        return Err(rust_i18n::t!("msg_err_nfqws_missing").to_string());
    }
    if !paths::strategies_installed() {
        return Err(rust_i18n::t!("msg_err_strat_missing").to_string());
    }

    let mut cfg = effective_config(args)?;
    let strategy = args
        .strategy
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| cfg.strategy.clone());
    if strategy.is_empty() {
        return Err(rust_i18n::t!("msg_no_strat").to_string());
    }
    let strategy = resolve_strategy_name(&strategy)?;

    #[cfg(target_os = "linux")]
    let interface = args.interface.clone().unwrap_or_else(|| cfg.interface.clone());
    #[cfg(target_os = "linux")]
    let backend = zapret_core::firewall::LinuxBackend::from_config(&cfg.backend);
    #[cfg(target_os = "windows")]
    let backend = zapret_core::firewall::windivert::WinDivertBackend;

    let _quiet = zapret_core::firewall::QuietGuard::new();
    let mut on_event = |event: domains::TtlEvent| -> bool {
        match event {
            domains::TtlEvent::Trying(ttl) => {
                println!("{}", rust_i18n::t!("cli_ttl_trying").replace("{}", &ttl.to_string()))
            }
            domains::TtlEvent::Refused(ttl, line) => {
                println!("TTL {}: {}", ttl, line.lines().next().unwrap_or_default())
            }
            domains::TtlEvent::Probed { domain, ok } => {
                println!("  {} {}", if ok { "OK" } else { "FAIL" }, domain)
            }
            domains::TtlEvent::Found(ttl) => println!("{}", rust_i18n::t!("ttl_found").replace("{}", &ttl.to_string())),
        }
        true
    };

    #[cfg(target_os = "linux")]
    let result = domains::ttl::autopick_ttl(&strategy, &interface, &backend, &mut on_event);
    #[cfg(not(target_os = "linux"))]
    let result = domains::ttl::autopick_ttl(&strategy, &backend, &mut on_event);

    let ttl = result?;
    cfg.dpi_desync_ttl = Some(ttl);
    config::save_config_to(&config_path(args), &cfg)?;
    Ok(())
}

fn split_list(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Service and Defender
// ---------------------------------------------------------------------------

fn run_service(args: &Cli, action: ServiceAction) -> Result<(), String> {
    let manager = service::get_detected_manager().ok_or_else(|| rust_i18n::t!("msg_err_init").to_string())?;

    match action {
        ServiceAction::Status => {
            println!("service_installed={}", manager.is_installed());
            println!("service_active={}", manager.is_active());
            return Ok(());
        }
        ServiceAction::Install => {
            if !paths::nfqws_installed() {
                return Err(rust_i18n::t!("msg_err_nfqws_missing").to_string());
            }
            if !paths::strategies_installed() {
                return Err(rust_i18n::t!("msg_err_strat_missing").to_string());
            }
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            let config_path = config_path(args);
            ensure_config(&config_path, args.config.is_some())?;
            manager
                .install(&exe, &config_path, &paths::cache_dir())
                .and_then(|_| manager.start())?;
        }
        ServiceAction::Start => manager.start()?,
        ServiceAction::Stop => manager.stop()?,
        ServiceAction::Restart => manager.restart()?,
        ServiceAction::Uninstall => manager.uninstall()?,
    }

    println!("{}", rust_i18n::t!("msg_op_ok"));
    Ok(())
}

fn run_defender(action: DefenderAction) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use zapret_wrapper::defender;

        match action {
            DefenderAction::Add => defender::add_defender_exclusion()?,
            DefenderAction::Remove => defender::remove_defender_exclusion()?,
            DefenderAction::Status => {
                let excluded = defender::check_defender_exclusion()?;
                println!("defender_excluded={}", excluded);
                return Ok(());
            }
        }
        println!("{}", rust_i18n::t!("msg_op_ok"));
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = action;
        Err(rust_i18n::t!("cli_err_defender_platform").to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttl_accepts_off_auto_and_the_supported_range() {
        assert_eq!(parse_ttl("off").unwrap(), None);
        assert_eq!(parse_ttl("auto").unwrap(), None);
        assert_eq!(parse_ttl("5").unwrap(), Some(5));
        assert!(parse_ttl("2").is_err());
        assert!(parse_ttl("21").is_err());
        assert!(parse_ttl("nope").is_err());
    }

    #[test]
    fn ipset_modes_have_readable_aliases() {
        assert_eq!(parse_ipset_mode("off").unwrap(), IpsetMode::None);
        assert_eq!(parse_ipset_mode("empty").unwrap(), IpsetMode::Any);
        assert_eq!(parse_ipset_mode("load").unwrap(), IpsetMode::Loaded);
        assert_eq!(parse_ipset_mode("CUSTOM").unwrap(), IpsetMode::Custom);
        assert!(parse_ipset_mode("bogus").is_err());
    }

    #[test]
    fn preset_all_covers_every_preset() {
        assert_eq!(
            parse_presets("all").unwrap(),
            (0..domains::PRESETS.len()).collect::<Vec<_>>()
        );
        assert_eq!(parse_presets("0,discord").unwrap(), vec![0]);
        assert!(parse_presets("").is_err());
        assert!(parse_presets("nope").is_err());
    }

    #[test]
    fn strategies_accept_names_indices_and_all() {
        let all = vec!["discord.bat".to_string(), "general.bat".to_string()];
        assert_eq!(parse_strategies("all", &all).unwrap(), vec![0, 1]);
        assert_eq!(parse_strategies("none", &all).unwrap(), Vec::<usize>::new());
        assert_eq!(parse_strategies("discord", &all).unwrap(), vec![0]);
        assert_eq!(parse_strategies("general.bat", &all).unwrap(), vec![1]);
        assert_eq!(parse_strategies("1", &all).unwrap(), vec![1]);
        assert!(parse_strategies("missing", &all).is_err());
    }

    #[test]
    fn protocols_and_block_checks_parse_lists() {
        assert_eq!(parse_protocols("none").unwrap(), (false, false, false, false));
        assert_eq!(parse_protocols("all").unwrap(), (true, true, true, true));
        assert_eq!(parse_protocols("http, tls1.3").unwrap(), (true, false, true, false));
        assert!(parse_protocols("http,bogus").is_err());

        assert_eq!(parse_block_checks("none").unwrap().count_enabled(), 0);
        assert_eq!(parse_block_checks("all").unwrap().count_enabled(), 6);
        let checks = parse_block_checks("dns,cidr").unwrap();
        assert!(checks.get(0) && checks.get(5) && !checks.get(1));
        assert!(parse_block_checks("bogus").is_err());
    }
}
