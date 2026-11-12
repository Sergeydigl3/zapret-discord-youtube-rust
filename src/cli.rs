//! Command-line surface of zapret-rust.
//!
//! The program is a TUI by default. Every menu action has a flag equivalent
//! here, so the same things can be configured and run from a script without a
//! terminal: run, downloads, interface/strategy/gamefilter/backend/ipset/TTL/
//! fakes/router, autotune, the service, Defender exclusions and the list files.

use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "run",
    about = "DPI bypass for Discord and YouTube. Opens the TUI by default; pass a flag to perform an action without it.",
    disable_help_flag = true
)]
pub struct Cli {
    // ---- Run parameters (the old CLI surface) ---------------------------
    #[arg(short = 'c', long = "config", help = "Load configuration from file")]
    pub config: Option<String>,

    #[arg(short = 's', long = "strategy", help = "Use specific strategy")]
    pub strategy: Option<String>,

    /// Linux only: nftables and iptables can bind the rules to one output
    /// device, WinDivert cannot, so on Windows there is no such option to pass.
    #[cfg(target_os = "linux")]
    #[arg(
        short = 'i',
        long = "interface",
        value_name = "NAME",
        help = "Network interface (default: any)"
    )]
    pub interface: Option<String>,

    #[arg(long = "gamefiltertcp", short = 't', help = "Enable gamefilter TCP")]
    pub gamefiltertcp: bool,

    #[arg(long = "gamefilterudp", short = 'u', help = "Enable gamefilter UDP")]
    pub gamefilterudp: bool,

    #[arg(
        long = "cache-dir",
        short = 'd',
        help = "Cache directory for downloaded dependencies and strategies"
    )]
    pub cache_dir: Option<String>,

    // ---- Modes ----------------------------------------------------------
    /// Do not open the TUI. With no other action flag this runs zapret using
    /// the saved configuration plus the run flags below.
    #[arg(long = "no-tui", alias = "headless", help = "Never open the TUI")]
    pub no_tui: bool,

    /// Run zapret once, in the foreground, without the TUI.
    #[arg(long = "run", help = "Run zapret without the TUI")]
    pub run: bool,

    // ---- Persistent settings (the TUI configuration rows) ---------------
    #[cfg(target_os = "linux")]
    #[arg(long = "set-interface", value_name = "NAME", help = "Persist the network interface")]
    pub set_interface: Option<String>,

    #[arg(long = "set-strategy", value_name = "NAME", help = "Persist the strategy")]
    pub set_strategy: Option<String>,

    #[arg(
        long = "set-gamefilter-tcp",
        value_name = "ON|OFF",
        num_args = 0..=1,
        default_missing_value = "on",
        value_parser = parse_toggle,
        help = "Persist gamefilter TCP"
    )]
    pub set_gamefilter_tcp: Option<bool>,

    #[arg(
        long = "set-gamefilter-udp",
        value_name = "ON|OFF",
        num_args = 0..=1,
        default_missing_value = "on",
        value_parser = parse_toggle,
        help = "Persist gamefilter UDP"
    )]
    pub set_gamefilter_udp: Option<bool>,

    #[cfg(target_os = "linux")]
    #[arg(long = "set-backend", value_name = "BACKEND", help = "Persist the firewall backend")]
    pub set_backend: Option<String>,

    #[arg(long = "set-ipset-mode", value_name = "MODE", help = "Switch the ipset-all.txt mode")]
    pub set_ipset_mode: Option<String>,

    #[cfg(target_os = "linux")]
    #[arg(
        long = "set-router",
        value_name = "ON|OFF",
        num_args = 0..=1,
        default_missing_value = "on",
        value_parser = parse_toggle,
        help = "Persist router mode"
    )]
    pub set_router: Option<bool>,

    #[arg(
        long = "set-ttl",
        value_name = "VALUE",
        help = "Persist a fixed DPI TTL (off/auto clears it)"
    )]
    pub set_ttl: Option<String>,

    #[arg(
        long = "set-fake-discord",
        value_name = "FILE",
        help = "Use FILE for the Discord UDP fake payload"
    )]
    pub set_fake_discord: Option<String>,

    #[arg(
        long = "set-fake-gamefilter",
        value_name = "FILE",
        help = "Use FILE for the gamefilter UDP fake payload"
    )]
    pub set_fake_gamefilter: Option<String>,

    // ---- Downloads ------------------------------------------------------
    /// VERSION is `recommended` (the default when the value is omitted),
    /// `latest`, or an explicit Git tag.
    #[arg(
        long = "download-zapret",
        value_name = "VERSION",
        num_args = 0..=1,
        default_missing_value = "recommended",
        help = "Download zapret/nfqws"
    )]
    pub download_zapret: Option<String>,

    /// VERSION is `recommended` (the default), `latest`, or a Git tag/commit.
    #[arg(
        long = "download-strategies",
        value_name = "VERSION",
        num_args = 0..=1,
        default_missing_value = "recommended",
        help = "Download the strategies"
    )]
    pub download_strategies: Option<String>,

    #[arg(long = "download-defaults", help = "Download both recommended versions")]
    pub download_defaults: bool,

    // ---- Read-only inspection -------------------------------------------
    #[cfg(target_os = "linux")]
    #[arg(long = "list-interfaces", help = "Print the available interfaces")]
    pub list_interfaces: bool,

    #[arg(long = "list-strategies", help = "Print the available strategy files")]
    pub list_strategies: bool,

    #[arg(long = "list-fakes", help = "Print the available fake payload files")]
    pub list_fakes: bool,

    #[arg(long = "list-lists", help = "Print the editable list files")]
    pub list_lists: bool,

    #[arg(long = "list-domains", help = "Print the autotune/TTL domain files")]
    pub list_domains: bool,

    #[arg(long = "list-tags-zapret", help = "Print the zapret Git tags")]
    pub list_tags_zapret: bool,

    #[arg(long = "list-tags-strategies", help = "Print the strategies Git tags")]
    pub list_tags_strategies: bool,

    #[arg(long = "show-config", help = "Print the active configuration file")]
    pub show_config: bool,

    #[arg(long = "status", help = "Print dependencies, service and autotune status")]
    pub status: bool,

    #[arg(long = "autotune-results", help = "Print the last autotune report")]
    pub autotune_results: bool,

    // ---- List/domain editor ---------------------------------------------
    #[arg(long = "edit-list", value_name = "FILE", help = "Open one list file in $EDITOR")]
    pub edit_list: Option<String>,

    #[arg(
        long = "edit-domains",
        value_name = "PRESET|ttl",
        help = "Open an autotune preset or the TTL domain file in $EDITOR"
    )]
    pub edit_domains: Option<String>,

    // ---- Autotune -------------------------------------------------------
    #[arg(long = "autotune", help = "Run the autotune sweep without the TUI")]
    pub autotune: bool,

    #[arg(
        long = "autotune-presets",
        value_name = "LIST",
        help = "Presets for --autotune: discord,youtube,social,custom,all"
    )]
    pub autotune_presets: Option<String>,

    #[arg(
        long = "autotune-strategies",
        value_name = "LIST",
        help = "Strategies for --autotune: names, indices, all or none"
    )]
    pub autotune_strategies: Option<String>,

    #[arg(
        long = "autotune-requests",
        value_name = "N",
        help = "Requests per domain and protocol for --autotune"
    )]
    pub autotune_requests: Option<usize>,

    #[arg(
        long = "autotune-protocols",
        value_name = "LIST",
        help = "Protocols for --autotune: http,tls12,tls13,quic,all,none"
    )]
    pub autotune_protocols: Option<String>,

    #[arg(
        long = "autotune-block-checks",
        value_name = "LIST",
        help = "Block checks for --autotune: dns,tcp-rst,sni,siberian,quic,cidr,all,none"
    )]
    pub autotune_block_checks: Option<String>,

    #[arg(long = "ttl-autopick", help = "Sweep the DPI TTL range and save the result")]
    pub ttl_autopick: bool,

    // ---- Service --------------------------------------------------------
    #[arg(long = "service-install", help = "Install and start the system service")]
    pub service_install: bool,

    #[arg(long = "service-start", help = "Start the system service")]
    pub service_start: bool,

    #[arg(long = "service-stop", help = "Stop the system service")]
    pub service_stop: bool,

    #[arg(long = "service-restart", help = "Restart the system service")]
    pub service_restart: bool,

    #[arg(long = "service-uninstall", help = "Uninstall the system service")]
    pub service_uninstall: bool,

    #[arg(long = "service-status", help = "Print the system service status")]
    pub service_status: bool,

    // ---- Windows Defender ----------------------------------------------
    #[arg(long = "defender-add", help = "Add the cache folder to Defender exclusions")]
    pub defender_add: bool,

    #[arg(long = "defender-remove", help = "Remove the cache folder from Defender exclusions")]
    pub defender_remove: bool,

    #[arg(long = "defender-status", help = "Print the Defender exclusion status")]
    pub defender_status: bool,

    #[arg(short = 'h', long = "help", help = "Show this help")]
    pub help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAction {
    Install,
    Start,
    Stop,
    Restart,
    Uninstall,
    Status,
}

impl ServiceAction {
    fn label(self) -> &'static str {
        match self {
            Self::Install => "--service-install",
            Self::Start => "--service-start",
            Self::Stop => "--service-stop",
            Self::Restart => "--service-restart",
            Self::Uninstall => "--service-uninstall",
            Self::Status => "--service-status",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefenderAction {
    Add,
    Remove,
    Status,
}

impl DefenderAction {
    fn label(self) -> &'static str {
        match self {
            Self::Add => "--defender-add",
            Self::Remove => "--defender-remove",
            Self::Status => "--defender-status",
        }
    }
}

/// One non-interactive operation. Setting-only flags are not actions: they are
/// applied first and then either an action runs or the program exits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Run,
    DownloadZapret(String),
    DownloadStrategies(String),
    DownloadDefaults,
    #[cfg(target_os = "linux")]
    ListInterfaces,
    ListStrategies,
    ListFakes,
    ListLists,
    ListDomains,
    ListTagsZapret,
    ListTagsStrategies,
    ShowConfig,
    Status,
    ShowAutotuneResults,
    EditList(String),
    EditDomains(String),
    Autotune,
    TtlAutopick,
    Service(ServiceAction),
    Defender(DefenderAction),
}

impl Action {
    /// Whether the action needs root (the daemon, firewall, downloads, init
    /// system or Defender). Read-only and editor actions do not, so `--status`
    /// can be used as a normal user.
    pub fn needs_admin(&self) -> bool {
        match self {
            Self::Run
            | Self::DownloadZapret(_)
            | Self::DownloadStrategies(_)
            | Self::DownloadDefaults
            | Self::Autotune
            | Self::TtlAutopick => true,
            Self::Service(action) => !matches!(action, ServiceAction::Status),
            Self::Defender(action) => {
                #[cfg(target_os = "windows")]
                {
                    !matches!(action, DefenderAction::Status)
                }
                // The action only reports that it is unavailable elsewhere, so
                // there is no reason to ask for a password first.
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = action;
                    false
                }
            }
            _ => false,
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Run => "--run/--no-tui",
            Self::DownloadZapret(_) => "--download-zapret",
            Self::DownloadStrategies(_) => "--download-strategies",
            Self::DownloadDefaults => "--download-defaults",
            #[cfg(target_os = "linux")]
            Self::ListInterfaces => "--list-interfaces",
            Self::ListStrategies => "--list-strategies",
            Self::ListFakes => "--list-fakes",
            Self::ListLists => "--list-lists",
            Self::ListDomains => "--list-domains",
            Self::ListTagsZapret => "--list-tags-zapret",
            Self::ListTagsStrategies => "--list-tags-strategies",
            Self::ShowConfig => "--show-config",
            Self::Status => "--status",
            Self::ShowAutotuneResults => "--autotune-results",
            Self::EditList(_) => "--edit-list",
            Self::EditDomains(_) => "--edit-domains",
            Self::Autotune => "--autotune",
            Self::TtlAutopick => "--ttl-autopick",
            Self::Service(action) => action.label(),
            Self::Defender(action) => action.label(),
        }
    }
}

impl Cli {
    /// True when at least one flag asks for something other than the TUI.
    pub fn has_non_tui_request(&self) -> bool {
        if self.no_tui || self.run || self.has_settings() {
            return true;
        }
        // An invalid combination is still a command-line request: let the
        // command layer report it instead of silently opening the TUI.
        match self.action() {
            Ok(action) => action.is_some(),
            Err(_) => true,
        }
    }

    /// True when a persistent setting is being changed.
    pub fn has_settings(&self) -> bool {
        #[cfg(target_os = "linux")]
        let platform = self.set_interface.is_some() || self.set_backend.is_some() || self.set_router.is_some();
        #[cfg(not(target_os = "linux"))]
        let platform = false;

        platform
            || self.set_strategy.is_some()
            || self.set_gamefilter_tcp.is_some()
            || self.set_gamefilter_udp.is_some()
            || self.set_ipset_mode.is_some()
            || self.set_ttl.is_some()
            || self.set_fake_discord.is_some()
            || self.set_fake_gamefilter.is_some()
    }

    /// The operation requested by the flags, or `None` for the normal TUI.
    ///
    /// `--no-tui` is a mode rather than an action: with an explicit action it
    /// just says "do not fall back to the TUI", and on its own it means `--run`.
    pub fn action(&self) -> Result<Option<Action>, String> {
        let mut actions: Vec<Action> = Vec::new();

        if self.run {
            actions.push(Action::Run);
        }
        if let Some(version) = &self.download_zapret {
            actions.push(Action::DownloadZapret(version.clone()));
        }
        if let Some(version) = &self.download_strategies {
            actions.push(Action::DownloadStrategies(version.clone()));
        }
        if self.download_defaults {
            actions.push(Action::DownloadDefaults);
        }
        #[cfg(target_os = "linux")]
        if self.list_interfaces {
            actions.push(Action::ListInterfaces);
        }
        if self.list_strategies {
            actions.push(Action::ListStrategies);
        }
        if self.list_fakes {
            actions.push(Action::ListFakes);
        }
        if self.list_lists {
            actions.push(Action::ListLists);
        }
        if self.list_domains {
            actions.push(Action::ListDomains);
        }
        if self.list_tags_zapret {
            actions.push(Action::ListTagsZapret);
        }
        if self.list_tags_strategies {
            actions.push(Action::ListTagsStrategies);
        }
        if self.show_config {
            actions.push(Action::ShowConfig);
        }
        if self.status {
            actions.push(Action::Status);
        }
        if self.autotune_results {
            actions.push(Action::ShowAutotuneResults);
        }
        if let Some(file) = &self.edit_list {
            actions.push(Action::EditList(file.clone()));
        }
        if let Some(preset) = &self.edit_domains {
            actions.push(Action::EditDomains(preset.clone()));
        }
        if self.autotune {
            actions.push(Action::Autotune);
        }
        if self.ttl_autopick {
            actions.push(Action::TtlAutopick);
        }
        if self.service_install {
            actions.push(Action::Service(ServiceAction::Install));
        }
        if self.service_start {
            actions.push(Action::Service(ServiceAction::Start));
        }
        if self.service_stop {
            actions.push(Action::Service(ServiceAction::Stop));
        }
        if self.service_restart {
            actions.push(Action::Service(ServiceAction::Restart));
        }
        if self.service_uninstall {
            actions.push(Action::Service(ServiceAction::Uninstall));
        }
        if self.service_status {
            actions.push(Action::Service(ServiceAction::Status));
        }
        if self.defender_add {
            actions.push(Action::Defender(DefenderAction::Add));
        }
        if self.defender_remove {
            actions.push(Action::Defender(DefenderAction::Remove));
        }
        if self.defender_status {
            actions.push(Action::Defender(DefenderAction::Status));
        }

        if actions.len() > 1 {
            let names: Vec<&str> = actions.iter().map(Action::label).collect();
            return Err(format!("{} {}", rust_i18n::t!("cli_err_conflict"), names.join(", ")));
        }

        let has_autotune_options = self.autotune_presets.is_some()
            || self.autotune_strategies.is_some()
            || self.autotune_requests.is_some()
            || self.autotune_protocols.is_some()
            || self.autotune_block_checks.is_some();
        if has_autotune_options && !self.autotune {
            return Err(rust_i18n::t!("cli_err_autotune_opts").into_owned());
        }

        // Run parameters are only useful for a launch. Supplying one means the
        // user asked for that launch, not for the menu.
        #[cfg(target_os = "linux")]
        let has_run_options = self.config.is_some()
            || self.strategy.is_some()
            || self.interface.is_some()
            || self.gamefiltertcp
            || self.gamefilterudp;
        #[cfg(not(target_os = "linux"))]
        let has_run_options =
            self.config.is_some() || self.strategy.is_some() || self.gamefiltertcp || self.gamefilterudp;

        Ok(actions
            .pop()
            .or_else(|| (self.no_tui || has_run_options).then_some(Action::Run)))
    }

    /// Whether the selected operation needs to run as root.
    pub fn needs_admin(&self) -> bool {
        if self.help {
            return false;
        }
        // Report an invalid combination (or an autotune option without
        // `--autotune`) without asking for a password first.
        let action = self.action();
        if action.is_err() {
            return false;
        }
        if self.has_settings() || self.no_tui || self.run {
            return true;
        }
        match action {
            Ok(Some(action)) => action.needs_admin(),
            // No action means the interactive TUI, which needs root.
            Ok(None) => true,
            Err(_) => false,
        }
    }
}

/// `on`/`off` and the usual spellings, for flags with an optional value.
fn parse_toggle(value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enable" | "enabled" => Ok(true),
        "0" | "false" | "no" | "off" | "disable" | "disabled" => Ok(false),
        other => Err(format!("expected on/off, got '{other}'")),
    }
}

/// Print the help, ignoring a closed pipe (`zapret-rust --help | head`).
pub fn show_help() {
    use std::io::Write;

    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(render_help().as_bytes());
    let _ = stdout.flush();
}

/// Build the whole help text.
///
/// Every option row is stored in the locale file as `flags<TAB>description`,
/// so all descriptions can be lined up in one column. Continuation lines (for
/// long value lists) are indented to that same column.
pub fn render_help() -> String {
    let sections = help_sections();
    let width = flag_column_width(&sections);

    let mut out = String::new();
    out.push_str(&rust_i18n::t!("cli_usage"));
    out.push('\n');
    out.push_str(&rust_i18n::t!("cli_desc"));
    out.push_str("\n\n");

    out.push_str(&rust_i18n::t!("cli_how_title"));
    out.push('\n');
    out.push_str("  ");
    out.push_str(&rust_i18n::t!("cli_how_1"));
    out.push('\n');
    out.push_str("  ");
    out.push_str(&rust_i18n::t!("cli_how_2"));
    out.push('\n');
    out.push_str("  ");
    out.push_str(&rust_i18n::t!("cli_how_3"));
    out.push('\n');

    for (title, rows) in &sections {
        push_section(&mut out, title, rows, width);
    }

    push_examples(&mut out);
    out
}

/// All option groups, in the order they are printed.
fn help_sections() -> Vec<(String, Vec<String>)> {
    vec![
        (rust_i18n::t!("cli_group_common").to_string(), common_rows()),
        (rust_i18n::t!("cli_group_run").to_string(), run_rows()),
        (rust_i18n::t!("cli_group_settings").to_string(), settings_rows()),
        (rust_i18n::t!("cli_group_downloads").to_string(), download_rows()),
        (rust_i18n::t!("cli_group_actions").to_string(), action_rows()),
        (rust_i18n::t!("cli_group_inspect").to_string(), inspect_rows()),
    ]
}

/// Rows of the "common" group: options that fit every other group.
fn common_rows() -> Vec<String> {
    vec![
        rust_i18n::t!("cli_opt_d").to_string(),
        rust_i18n::t!("cli_opt_h").to_string(),
    ]
}

/// Rows of the "run" group.
fn run_rows() -> Vec<String> {
    let mut rows = vec![
        rust_i18n::t!("cli_opt_c").to_string(),
        rust_i18n::t!("cli_opt_s").to_string(),
    ];
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_i").to_string());
    rows.extend([
        rust_i18n::t!("cli_opt_t").to_string(),
        rust_i18n::t!("cli_opt_u").to_string(),
        rust_i18n::t!("cli_opt_no_tui").to_string(),
        rust_i18n::t!("cli_opt_run").to_string(),
    ]);
    rows
}

/// `--set-backend` choices, taken from the compiled backend list.
#[cfg(target_os = "linux")]
fn backend_names() -> String {
    use zapret_core::firewall::LinuxBackend;

    LinuxBackend::variants()
        .iter()
        .map(|backend| backend.to_config())
        .collect::<Vec<_>>()
        .join(", ")
}

/// `--set-ipset-mode` choices, taken from the mode enum.
fn ipset_mode_names() -> String {
    zapret_wrapper::lists::IpsetMode::all()
        .iter()
        .map(|mode| mode.keyword())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Autotune preset names, taken from the preset table.
fn preset_names() -> String {
    zapret_wrapper::domains::PRESETS
        .iter()
        .map(|preset| preset.name.to_lowercase())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Autotune protocol switches, taken from the protocol table.
fn protocol_names() -> String {
    zapret_wrapper::autotune::types::PROTOCOL_NAMES.join(", ")
}

/// Autotune block-check switches, taken from the check enum.
fn block_check_names() -> String {
    zapret_wrapper::autotune::types::BlockCheckType::all()
        .iter()
        .map(|check| check.keyword())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Rows of the "settings" group.
fn settings_rows() -> Vec<String> {
    let mut rows = Vec::new();
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_set_interface").to_string());
    rows.push(rust_i18n::t!("cli_opt_set_strategy").to_string());
    rows.extend([
        rust_i18n::t!("cli_opt_set_gf_tcp").to_string(),
        rust_i18n::t!("cli_opt_set_gf_udp").to_string(),
    ]);
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_set_backend").replace("{backends}", &backend_names()));
    rows.push(rust_i18n::t!("cli_opt_set_ipset").replace("{modes}", &ipset_mode_names()));
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_set_router").to_string());
    rows.push(
        rust_i18n::t!("cli_opt_set_ttl")
            .replace("{min}", &zapret_wrapper::domains::ttl::TTL_MIN.to_string())
            .replace("{max}", &zapret_wrapper::domains::ttl::TTL_MAX.to_string()),
    );
    rows.extend([
        rust_i18n::t!("cli_opt_set_fake_discord").to_string(),
        rust_i18n::t!("cli_opt_set_fake_gamefilter").to_string(),
    ]);
    rows
}

/// Rows of the "downloads" group.
fn download_rows() -> Vec<String> {
    vec![
        rust_i18n::t!("cli_opt_dl_zapret").to_string(),
        rust_i18n::t!("cli_opt_dl_strat").to_string(),
        rust_i18n::t!("cli_opt_dl_defaults").to_string(),
    ]
}

/// Rows of the "actions" group.
fn action_rows() -> Vec<String> {
    vec![
        rust_i18n::t!("cli_opt_autotune").to_string(),
        rust_i18n::t!("cli_opt_autotune_presets").replace("{presets}", &preset_names()),
        rust_i18n::t!("cli_opt_autotune_strategies").to_string(),
        rust_i18n::t!("cli_opt_autotune_requests").to_string(),
        rust_i18n::t!("cli_opt_autotune_protocols").replace("{protocols}", &protocol_names()),
        rust_i18n::t!("cli_opt_autotune_block_checks").replace("{checks}", &block_check_names()),
        rust_i18n::t!("cli_opt_ttl_autopick").to_string(),
        rust_i18n::t!("cli_opt_service_install").to_string(),
        rust_i18n::t!("cli_opt_service_start").to_string(),
        rust_i18n::t!("cli_opt_service_stop").to_string(),
        rust_i18n::t!("cli_opt_service_restart").to_string(),
        rust_i18n::t!("cli_opt_service_uninstall").to_string(),
        rust_i18n::t!("cli_opt_service_status").to_string(),
        rust_i18n::t!("cli_opt_defender_add").to_string(),
        rust_i18n::t!("cli_opt_defender_remove").to_string(),
        rust_i18n::t!("cli_opt_defender_status").to_string(),
        rust_i18n::t!("cli_opt_edit_list").to_string(),
        rust_i18n::t!("cli_opt_edit_domains").replace("{presets}", &preset_names()),
    ]
}

/// Rows of the "inspect" group.
fn inspect_rows() -> Vec<String> {
    let mut rows = Vec::new();
    #[cfg(target_os = "linux")]
    rows.push(rust_i18n::t!("cli_opt_list_ifaces").to_string());
    rows.extend([
        rust_i18n::t!("cli_opt_list_strategies").to_string(),
        rust_i18n::t!("cli_opt_list_fakes").to_string(),
        rust_i18n::t!("cli_opt_list_lists").to_string(),
        rust_i18n::t!("cli_opt_list_domains").to_string(),
        rust_i18n::t!("cli_opt_list_tags_zapret").to_string(),
        rust_i18n::t!("cli_opt_list_tags_strategies").to_string(),
        rust_i18n::t!("cli_opt_status").to_string(),
        rust_i18n::t!("cli_opt_show_config").to_string(),
        rust_i18n::t!("cli_opt_autotune_results").to_string(),
    ]);
    rows
}

/// The widest flags column across every section, so descriptions line up in
/// one column from the first group to the last.
fn flag_column_width(sections: &[(String, Vec<String>)]) -> usize {
    sections
        .iter()
        .flat_map(|(_, rows)| rows)
        .filter_map(|row| row.split_once('\t'))
        .map(|(flags, _)| flags.chars().count())
        .max()
        .unwrap_or(0)
}

fn push_section(out: &mut String, title: &str, rows: &[String], width: usize) {
    out.push('\n');
    out.push_str(title);
    out.push('\n');
    for row in rows {
        push_row(out, row, width);
    }
}

/// Print one `flags<TAB>description` row, padding the flags to `width`.
fn push_row(out: &mut String, row: &str, width: usize) {
    let (flags, description) = match row.split_once('\t') {
        Some(parts) => parts,
        None => {
            out.push_str("  ");
            out.push_str(row);
            out.push('\n');
            return;
        }
    };

    let column = 2 + width + 2;
    let padding = width.saturating_sub(flags.chars().count()) + 2;
    out.push_str("  ");
    out.push_str(flags);
    out.push_str(&" ".repeat(padding));

    let mut lines = description.split('\n');
    if let Some(first) = lines.next() {
        out.push_str(first);
    }
    out.push('\n');
    for line in lines {
        out.push_str(&" ".repeat(column));
        out.push_str(line);
        out.push('\n');
    }
}

fn push_examples(out: &mut String) {
    out.push('\n');
    out.push_str(&rust_i18n::t!("cli_group_examples"));
    out.push('\n');
    for (title, command) in [
        (rust_i18n::t!("cli_example_1_title"), rust_i18n::t!("cli_example_1_cmd")),
        (rust_i18n::t!("cli_example_2_title"), rust_i18n::t!("cli_example_2_cmd")),
        (rust_i18n::t!("cli_example_3_title"), rust_i18n::t!("cli_example_3_cmd")),
        (rust_i18n::t!("cli_example_4_title"), rust_i18n::t!("cli_example_4_cmd")),
        (rust_i18n::t!("cli_example_5_title"), rust_i18n::t!("cli_example_5_cmd")),
    ] {
        out.push_str("  ");
        out.push_str(&title);
        out.push('\n');
        out.push_str("    ");
        out.push_str(&command);
        out.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Cli {
        Cli::try_parse_from(args).expect("the CLI should parse")
    }

    #[test]
    fn no_arguments_means_the_tui() {
        let cli = parse(&["zapret-rust"]);
        assert!(!cli.has_non_tui_request());
        assert_eq!(cli.action().unwrap(), None);
        assert!(cli.needs_admin());
    }

    #[test]
    fn no_tui_alone_runs_once() {
        let cli = parse(&["zapret-rust", "--no-tui"]);
        assert_eq!(cli.action().unwrap(), Some(Action::Run));
        assert!(cli.has_non_tui_request());
    }

    #[test]
    fn run_parameters_alone_run_once() {
        let cli = parse(&["zapret-rust", "-c", "conf.env"]);
        assert_eq!(cli.action().unwrap(), Some(Action::Run));

        let cli = parse(&["zapret-rust", "-t"]);
        assert_eq!(cli.action().unwrap(), Some(Action::Run));

        let cli = parse(&["zapret-rust", "-u"]);
        assert_eq!(cli.action().unwrap(), Some(Action::Run));

        #[cfg(target_os = "linux")]
        {
            let cli = parse(&["zapret-rust", "-i", "eth0"]);
            assert_eq!(cli.action().unwrap(), Some(Action::Run));
        }
    }

    #[test]
    fn no_tui_with_an_action_does_not_add_a_run() {
        let cli = parse(&["zapret-rust", "--no-tui", "--service-status"]);
        assert_eq!(cli.action().unwrap(), Some(Action::Service(ServiceAction::Status)));
    }

    #[test]
    fn two_actions_are_rejected() {
        let cli = parse(&["zapret-rust", "--download-defaults", "--status"]);
        assert!(cli.action().is_err());
    }

    #[test]
    fn autotune_options_need_autotune() {
        let cli = parse(&["zapret-rust", "--autotune-presets", "discord"]);
        assert!(cli.action().is_err());
    }

    #[test]
    fn optional_versions_default_to_recommended() {
        let cli = parse(&["zapret-rust", "--download-zapret"]);
        assert_eq!(
            cli.action().unwrap(),
            Some(Action::DownloadZapret("recommended".to_string()))
        );

        let cli = parse(&["zapret-rust", "--download-zapret", "latest"]);
        assert_eq!(
            cli.action().unwrap(),
            Some(Action::DownloadZapret("latest".to_string()))
        );
    }

    #[test]
    fn toggles_accept_on_and_off() {
        let cli = parse(&["zapret-rust", "--set-gamefilter-tcp"]);
        assert_eq!(cli.set_gamefilter_tcp, Some(true));

        let cli = parse(&["zapret-rust", "--set-gamefilter-tcp", "off"]);
        assert_eq!(cli.set_gamefilter_tcp, Some(false));

        let cli = parse(&["zapret-rust", "--set-gamefilter-udp=on"]);
        assert_eq!(cli.set_gamefilter_udp, Some(true));
    }

    #[test]
    fn read_only_actions_do_not_need_root() {
        let cli = parse(&["zapret-rust", "--status"]);
        assert!(!cli.needs_admin());
        let cli = parse(&["zapret-rust", "--list-strategies"]);
        assert!(!cli.needs_admin());
    }

    #[test]
    fn headless_is_an_alias_for_no_tui() {
        let cli = parse(&["zapret-rust", "--headless"]);
        assert_eq!(cli.action().unwrap(), Some(Action::Run));
    }

    #[test]
    fn settings_alone_save_and_exit() {
        let cli = parse(&["zapret-rust", "--set-ttl", "5"]);
        assert_eq!(cli.action().unwrap(), None);
        assert!(cli.has_settings());
        assert!(cli.has_non_tui_request());
        assert!(cli.needs_admin());
    }

    #[test]
    fn settings_can_be_combined_with_one_action() {
        let cli = parse(&["zapret-rust", "--set-strategy", "general", "--run"]);
        assert_eq!(cli.action().unwrap(), Some(Action::Run));
    }

    #[test]
    fn help_never_needs_root() {
        let cli = parse(&["zapret-rust", "--help"]);
        assert!(!cli.needs_admin());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn unavailable_defender_actions_do_not_need_root() {
        let cli = parse(&["zapret-rust", "--defender-add"]);
        assert!(!cli.needs_admin());
    }

    #[test]
    fn conflicts_do_not_prompt_for_a_password() {
        let cli = parse(&["zapret-rust", "--set-ttl", "5", "--status", "--show-config"]);
        assert!(cli.action().is_err());
        assert!(!cli.needs_admin());
    }

    #[test]
    fn every_help_row_has_flags_and_text() {
        let sections = help_sections();
        assert!(!sections.is_empty());
        for (title, rows) in &sections {
            assert!(!title.is_empty());
            assert!(!rows.is_empty(), "section '{title}' is empty");
            for row in rows {
                let (flags, description) = row.split_once('\t').unwrap_or_else(|| panic!("row '{row}' has no tab"));
                assert!(!flags.trim().is_empty(), "row '{row}' has no flags");
                assert!(!description.trim().is_empty(), "row '{row}' has no description");
            }
        }
    }

    #[test]
    fn help_descriptions_share_one_column() {
        let sections = help_sections();
        let width = flag_column_width(&sections);
        let column = 2 + width + 2;

        let mut out = String::new();
        for (title, rows) in &sections {
            push_section(&mut out, title, rows, width);
        }

        for row in sections.iter().flat_map(|(_, rows)| rows) {
            let (flags, description) = row.split_once('\t').unwrap();
            let first = description.split('\n').next().unwrap();
            let line = out
                .lines()
                .find(|line| line.starts_with(&format!("  {flags} ")))
                .unwrap_or_else(|| panic!("row '{flags}' is missing from the help"));
            let desc_at = line.chars().count() - first.chars().count();
            assert_eq!(desc_at, column, "misaligned description in '{row}'");
        }
    }

    #[test]
    fn continuation_lines_align_with_the_description_column() {
        let mut out = String::new();
        push_row(&mut out, "--wide VALUE\tFirst line\nSecond line", 14);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        let column = 2 + 14 + 2;
        assert_eq!(lines[0].chars().count() - "First line".chars().count(), column);
        assert_eq!(lines[1], format!("{}Second line", " ".repeat(column)));
    }

    #[test]
    fn help_mentions_every_group_and_example() {
        let help = render_help();
        for key in [
            "cli_how_title",
            "cli_group_common",
            "cli_group_run",
            "cli_group_settings",
            "cli_group_downloads",
            "cli_group_actions",
            "cli_group_inspect",
            "cli_group_examples",
        ] {
            assert!(help.contains(rust_i18n::t!(key).as_ref()), "help is missing '{key}'");
        }
        for n in 1..=5 {
            let title = match n {
                1 => rust_i18n::t!("cli_example_1_title"),
                2 => rust_i18n::t!("cli_example_2_title"),
                3 => rust_i18n::t!("cli_example_3_title"),
                4 => rust_i18n::t!("cli_example_4_title"),
                _ => rust_i18n::t!("cli_example_5_title"),
            };
            assert!(help.contains(title.as_ref()));
        }
    }
}
