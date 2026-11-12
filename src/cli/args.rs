//! The flag groups.
//!
//! The command line is read in the same groups the help prints and the code
//! uses: run parameters, persistent settings, downloads, inspection, editors,
//! autotune, the service, and Defender. Each group is a `clap` struct of its
//! own, so adding a flag to one area does not turn the top-level struct into a
//! list of everything the program can do.

use clap::{Args, Parser};

/// Everything the program accepts.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "run",
    about = "DPI bypass for Discord and YouTube. Opens the TUI by default; pass a flag to perform an action without it.",
    disable_help_flag = true
)]
pub struct Cli {
    #[command(flatten)]
    pub common: CommonArgs,
    #[command(flatten)]
    pub run: RunArgs,
    #[command(flatten)]
    pub settings: SettingsArgs,
    #[command(flatten)]
    pub downloads: DownloadArgs,
    #[command(flatten)]
    pub inspect: InspectArgs,
    #[command(flatten)]
    pub edit: EditArgs,
    #[command(flatten)]
    pub autotune: AutotuneArgs,
    #[command(flatten)]
    pub service: ServiceArgs,
    #[command(flatten)]
    pub defender: DefenderArgs,

    #[arg(short = 'h', long = "help", help = "Show this help")]
    pub help: bool,
}

/// Options that apply no matter what the program is asked to do.
#[derive(Args, Debug, Clone)]
pub struct CommonArgs {
    #[arg(
        long = "cache-dir",
        short = 'd',
        help = "Cache directory for downloaded dependencies and strategies"
    )]
    pub cache_dir: Option<String>,
}

/// Starting zapret: the old command-line surface plus the two headless modes.
#[derive(Args, Debug, Clone)]
pub struct RunArgs {
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

    /// Do not open the TUI. With no other action flag this runs zapret using
    /// the saved configuration plus the run flags below.
    #[arg(long = "no-tui", alias = "headless", help = "Never open the TUI")]
    pub no_tui: bool,

    /// Run zapret once, in the foreground, without the TUI.
    #[arg(long = "run", help = "Run zapret without the TUI")]
    pub run: bool,
}

/// Persistent settings: the rows the TUI saves and the next run reads back.
#[derive(Args, Debug, Clone)]
pub struct SettingsArgs {
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
        value_parser = super::parse_toggle,
        help = "Persist gamefilter TCP"
    )]
    pub set_gamefilter_tcp: Option<bool>,

    #[arg(
        long = "set-gamefilter-udp",
        value_name = "ON|OFF",
        num_args = 0..=1,
        default_missing_value = "on",
        value_parser = super::parse_toggle,
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
        value_parser = super::parse_toggle,
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
}

impl SettingsArgs {
    /// True when at least one setting is being changed.
    pub fn has_any(&self) -> bool {
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
}

/// Downloading the runtime and the strategy repository.
#[derive(Args, Debug, Clone)]
pub struct DownloadArgs {
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
}

/// Read-only inspection: print something and exit.
#[derive(Args, Debug, Clone)]
pub struct InspectArgs {
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
}

/// Opening a list or domain file in the user's editor.
#[derive(Args, Debug, Clone)]
pub struct EditArgs {
    #[arg(long = "edit-list", value_name = "FILE", help = "Open one list file in $EDITOR")]
    pub edit_list: Option<String>,

    #[arg(
        long = "edit-domains",
        value_name = "PRESET|ttl",
        help = "Open an autotune preset or the TTL domain file in $EDITOR"
    )]
    pub edit_domains: Option<String>,
}

/// The strategy sweep and the TTL sweep.
#[derive(Args, Debug, Clone)]
pub struct AutotuneArgs {
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

    #[arg(long = "ttl-autopick", help = "Sweep the DPI TTL range and save the result")]
    pub ttl_autopick: bool,
}

impl AutotuneArgs {
    /// True when a flag only makes sense together with `--autotune`.
    pub fn has_options(&self) -> bool {
        self.autotune_presets.is_some()
            || self.autotune_strategies.is_some()
            || self.autotune_requests.is_some()
            || self.autotune_protocols.is_some()
    }
}

/// The background service.
#[derive(Args, Debug, Clone)]
pub struct ServiceArgs {
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
}

/// Windows Defender exclusions.
#[derive(Args, Debug, Clone)]
pub struct DefenderArgs {
    #[arg(long = "defender-add", help = "Add the cache folder to Defender exclusions")]
    pub defender_add: bool,

    #[arg(long = "defender-remove", help = "Remove the cache folder from Defender exclusions")]
    pub defender_remove: bool,

    #[arg(long = "defender-status", help = "Print the Defender exclusion status")]
    pub defender_status: bool,
}
