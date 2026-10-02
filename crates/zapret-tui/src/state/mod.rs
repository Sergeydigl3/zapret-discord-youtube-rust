//! Application state.
//!
//! [`AppState`] is data plus the few operations that read the outside world
//! (refresh the installed/active flags, persist the selection). Everything a
//! *key* can do lives in [`actions`], one handler per screen.

pub mod actions;
pub mod screens;

use zapret_wrapper::autotune::{AutotuneConfig, AutotuneResults};
use zapret_wrapper::config;
use zapret_wrapper::domains;
use zapret_wrapper::fakes::FakesState;
use zapret_wrapper::lists::IpsetMode;

pub use screens::{
    ActiveScreen, AutotuneBlockChecksState, AutotuneMenuState, AutotuneProtocolsState, AutotuneReportTab,
    DownloadDepsMenuState, DownloadSubmenuState, ExtendedMenuState, FakesMenuState, FakesSelectTarget,
    GamefilterMenuState, History, MainMenuState, TtlMenuState, VersionTarget,
};

#[cfg(target_os = "windows")]
pub use screens::DefenderMenuState;

#[cfg(target_os = "linux")]
use zapret_core::firewall::LinuxBackend;

pub struct AppState {
    /// The interfaces the rules can be bound to, and the one that is.
    ///
    /// Linux-only. nftables and iptables match on the output device, WinDivert
    /// filters the whole system, so on Windows there is no such choice and the
    /// menu row that offered one is compiled out with it.
    #[cfg(target_os = "linux")]
    pub interfaces: Vec<String>,
    #[cfg(target_os = "linux")]
    pub selected_interface: usize,

    #[cfg(target_os = "linux")]
    pub selected_backend: LinuxBackend,

    /// Whether router mode is switched on: this machine forwards and
    /// masquerades another device's traffic. Linux-only.
    #[cfg(target_os = "linux")]
    pub router_mode: bool,

    pub available_ipset_modes: Vec<IpsetMode>,
    pub selected_ipset_mode: usize,

    pub strategies: Vec<String>,
    pub selected_strategy: usize,
    pub strategy_menu_index: usize,

    pub tcp_gamefilter: bool,
    pub udp_gamefilter: bool,

    pub active_screen: ActiveScreen,
    /// Where the last frame put its rows, for the mouse to point at. Rebuilt by
    /// every draw, so it always describes the frame that is on screen.
    pub hit: crate::mouse::HitMap,
    /// The screens behind the current one, oldest first. Esc walks it backwards
    /// and the breadcrumb reads it forwards.
    pub history: History,
    pub main_menu: MainMenuState,
    pub extended_menu: ExtendedMenuState,

    #[cfg(target_os = "windows")]
    pub defender_menu: DefenderMenuState,
    #[cfg(target_os = "windows")]
    pub defender_status_cache: Option<bool>,

    pub download_deps_menu: DownloadDepsMenuState,
    pub download_zapret_menu: DownloadSubmenuState,
    pub download_strategies_menu: DownloadSubmenuState,
    pub gamefilter_menu: GamefilterMenuState,
    pub fakes_state: FakesState,
    pub fakes_menu: FakesMenuState,
    pub fakes_select_index: usize,
    pub fakes_select_for: FakesSelectTarget,
    pub nfqws_target: VersionTarget,
    pub strat_target: VersionTarget,

    pub available_nfqws_tags: Vec<String>,
    pub available_strat_tags: Vec<String>,
    pub nfqws_tag_index: usize,
    pub strat_tag_index: usize,

    pub should_run: bool,
    pub should_quit: bool,
    pub should_download_zapret: bool,
    pub should_download_strategies: bool,
    pub should_download_defaults: bool,
    pub status_message: Option<String>,

    pub nfqws_installed: bool,
    pub strategies_installed: bool,

    pub service_installed: bool,
    pub service_active: bool,
    pub service_menu_index: usize,

    pub lists_files: Vec<String>,
    pub lists_menu_index: usize,
    pub should_open_editor: Option<String>,
    pub domain_files: Vec<(String, String)>,
    pub domain_files_index: usize,

    pub autotune_config: AutotuneConfig,
    pub autotune_results: Option<AutotuneResults>,
    pub has_autotune_results_file: bool,
    pub autotune_menu: AutotuneMenuState,
    pub autotune_menu_index: usize,
    pub autotune_protocols_menu: AutotuneProtocolsState,
    pub autotune_block_checks_menu: AutotuneBlockChecksState,
    pub autotune_preset_index: usize,
    pub autotune_strat_index: usize,
    pub autotune_results_index: usize,
    pub autotune_report_tab: AutotuneReportTab,
    pub should_run_autotune: bool,
    pub autotune_running: bool,
    pub autotune_request_editing: bool,
    pub autotune_request_buf: String,
    pub should_run_ttl: bool,
    pub dpi_desync_ttl: Option<u8>,
    pub ttl_menu: TtlMenuState,

    /// The sweep in flight, if any. While this is set the frame loop paints it
    /// instead of the menus and only reads the cancel key.
    pub job: Option<crate::jobs::Job>,
}

impl AppState {
    pub fn show_error(&mut self, msg: String) {
        self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), msg));
    }

    pub fn new(strategies: Vec<String>) -> Self {
        let _ = config::ensure_default_config();
        let _ = domains::ensure_domain_files();

        let domain_files: Vec<(String, String)> = {
            let mut files: Vec<(String, String)> = Vec::new();
            for (idx, preset) in domains::PRESETS.iter().enumerate() {
                let is_custom = idx == domains::PRESETS.len() - 1;
                let label = if is_custom {
                    rust_i18n::t!("menu_domain_custom").into_owned()
                } else {
                    preset.name.to_string()
                };
                files.push((
                    label,
                    domains::preset_domains_file_path(idx).to_string_lossy().into_owned(),
                ));
            }
            files.push((
                rust_i18n::t!("menu_domain_ttl").into_owned(),
                domains::ttl::ttl_domains_file_path().to_string_lossy().into_owned(),
            ));
            files
        };

        let saved_cfg = config::load_config(&zapret_wrapper::paths::config_path().to_string_lossy()).ok();

        #[cfg(target_os = "linux")]
        let interfaces = zapret_wrapper::platform::get_interfaces();
        #[cfg(target_os = "linux")]
        let selected_interface = saved_cfg.as_ref().map_or(0, |cfg| {
            interfaces.iter().position(|i| i == &cfg.interface).unwrap_or(0)
        });

        let selected_strategy = saved_cfg
            .as_ref()
            .map_or(0, |cfg| strategies.iter().position(|s| s == &cfg.strategy).unwrap_or(0));
        let tcp_gamefilter = saved_cfg.as_ref().is_some_and(|cfg| cfg.gamefilter_tcp);
        let udp_gamefilter = saved_cfg.as_ref().is_some_and(|cfg| cfg.gamefilter_udp);

        #[cfg(target_os = "linux")]
        let selected_backend = saved_cfg.as_ref().map_or_else(
            || LinuxBackend::from_config("nftables"),
            |cfg| LinuxBackend::from_config(&cfg.backend),
        );

        #[cfg(target_os = "linux")]
        let router_mode = saved_cfg.as_ref().is_some_and(|cfg| cfg.router);

        let available_ipset_modes = zapret_wrapper::lists::get_available_modes();
        let current_ipset_mode = zapret_wrapper::lists::determine_current_mode();
        let selected_ipset_mode = available_ipset_modes
            .iter()
            .position(|m| m == &current_ipset_mode)
            .unwrap_or(0);

        let mut app = Self {
            #[cfg(target_os = "linux")]
            interfaces,
            #[cfg(target_os = "linux")]
            selected_interface,
            #[cfg(target_os = "linux")]
            selected_backend,
            #[cfg(target_os = "linux")]
            router_mode,
            available_ipset_modes,
            selected_ipset_mode,
            strategies,
            selected_strategy,
            strategy_menu_index: selected_strategy,
            tcp_gamefilter,
            udp_gamefilter,
            active_screen: ActiveScreen::Main,
            hit: crate::mouse::HitMap::default(),
            history: History::default(),

            main_menu: MainMenuState::first(),
            extended_menu: ExtendedMenuState::Ttl,

            #[cfg(target_os = "windows")]
            defender_menu: DefenderMenuState::Add,
            #[cfg(target_os = "windows")]
            defender_status_cache: zapret_wrapper::defender::check_defender_exclusion().ok(),

            download_deps_menu: DownloadDepsMenuState::ZapretDownloader,
            download_zapret_menu: DownloadSubmenuState::Version,
            download_strategies_menu: DownloadSubmenuState::Version,
            gamefilter_menu: GamefilterMenuState::Tcp,
            fakes_state: zapret_wrapper::fakes::load_fakes_state(),
            fakes_menu: FakesMenuState::DiscordUdp,
            fakes_select_index: 0,
            fakes_select_for: FakesSelectTarget::DiscordUdp,
            nfqws_target: VersionTarget::Recommended,
            strat_target: VersionTarget::Recommended,

            available_nfqws_tags: Vec::new(),
            available_strat_tags: Vec::new(),
            nfqws_tag_index: 0,
            strat_tag_index: 0,

            should_run: false,
            should_quit: false,
            should_download_zapret: false,
            should_download_strategies: false,
            should_download_defaults: false,
            status_message: None,

            nfqws_installed: zapret_wrapper::paths::nfqws_installed(),
            strategies_installed: zapret_wrapper::paths::strategies_installed(),

            service_installed: false,
            service_active: false,
            service_menu_index: 0,

            lists_files: Vec::new(),
            lists_menu_index: 0,
            should_open_editor: None,
            domain_files,
            domain_files_index: 0,

            autotune_config: AutotuneConfig::default(),
            autotune_results: None,
            has_autotune_results_file: zapret_wrapper::autotune::load_results_file().is_some(),
            autotune_menu: AutotuneMenuState::PresetSelection,
            autotune_menu_index: 0,
            autotune_protocols_menu: AutotuneProtocolsState::Http,
            autotune_block_checks_menu: AutotuneBlockChecksState::DnsSpoof,
            autotune_preset_index: 0,
            autotune_strat_index: 0,
            autotune_results_index: 0,
            autotune_report_tab: AutotuneReportTab::Summary,
            should_run_autotune: false,
            autotune_running: false,
            autotune_request_editing: false,
            autotune_request_buf: String::new(),
            should_run_ttl: false,
            dpi_desync_ttl: config::load_ttl(),
            ttl_menu: TtlMenuState::DontTouch,
            job: None,
        };
        app.refresh_service_status();
        app
    }

    pub fn refresh_dep_status(&mut self) {
        self.nfqws_installed = zapret_wrapper::paths::nfqws_installed();
        self.strategies_installed = zapret_wrapper::paths::strategies_installed();
    }

    #[cfg(target_os = "windows")]
    pub fn refresh_defender_status(&mut self) {
        self.defender_status_cache = zapret_wrapper::defender::check_defender_exclusion().ok();
    }

    pub fn refresh_service_status(&mut self) {
        match zapret_wrapper::service::get_detected_manager() {
            Some(mgr) => {
                self.service_installed = mgr.is_installed();
                self.service_active = mgr.is_active();
            }
            None => {
                self.service_installed = false;
                self.service_active = false;
            }
        }

        let count = self.get_service_menu_count();
        if count > 0 && self.service_menu_index >= count {
            self.service_menu_index = count - 1;
        }
    }

    /// The firewall the jobs run behind.
    ///
    /// The single place that answers "which backend on this platform": the menu
    /// picks one on Linux, Windows has only WinDivert.
    pub fn firewall_backend(&self) -> &dyn zapret_core::firewall::FirewallBackend {
        #[cfg(target_os = "linux")]
        {
            &self.selected_backend
        }
        #[cfg(target_os = "windows")]
        {
            &zapret_core::firewall::windivert::WinDivertBackend
        }
    }

    /// An owned backend, for a sweep that runs on its own thread.
    ///
    /// [`Self::firewall_backend`] borrows `self`, and a worker thread cannot
    /// borrow the app. Every backend is a unit struct or a fieldless enum, so
    /// boxing a copy costs nothing and crosses the thread boundary.
    pub fn owned_backend(&self) -> Box<dyn zapret_core::firewall::FirewallBackend> {
        #[cfg(target_os = "linux")]
        {
            Box::new(self.selected_backend)
        }
        #[cfg(target_os = "windows")]
        {
            Box::new(zapret_core::firewall::windivert::WinDivertBackend)
        }
    }

    pub fn refresh_ipset_status(&mut self) {
        self.available_ipset_modes = zapret_wrapper::lists::get_available_modes();
        let current_ipset_mode = zapret_wrapper::lists::determine_current_mode();
        self.selected_ipset_mode = self
            .available_ipset_modes
            .iter()
            .position(|m| m == &current_ipset_mode)
            .unwrap_or(0);
    }

    pub(crate) fn save_current_config(&self) {
        let strategy = self
            .strategies
            .get(self.selected_strategy)
            .map(|s| s.as_str())
            .unwrap_or("");
        #[cfg(target_os = "linux")]
        let backend = self.selected_backend.to_config();
        #[cfg(not(target_os = "linux"))]
        let backend = "nftables";
        let _ = config::save_tui_state(
            #[cfg(target_os = "linux")]
            self.interfaces
                .get(self.selected_interface)
                .map(|s| s.as_str())
                .unwrap_or("any"),
            strategy,
            self.tcp_gamefilter,
            self.udp_gamefilter,
            backend,
        );
    }

    pub fn get_service_menu_count(&self) -> usize {
        if !self.service_installed {
            2
        } else if self.service_active {
            4
        } else {
            3
        }
    }

    pub fn check_dependencies(&mut self) -> bool {
        self.refresh_dep_status();
        if !self.nfqws_installed || !self.strategies_installed {
            let msg = if !self.nfqws_installed && !self.strategies_installed {
                rust_i18n::t!("msg_err_both_missing").into_owned()
            } else if !self.nfqws_installed {
                rust_i18n::t!("msg_err_nfqws_missing").into_owned()
            } else {
                rust_i18n::t!("msg_err_strat_missing").into_owned()
            };
            self.show_error(msg);
            false
        } else {
            true
        }
    }

    /// The interface the rules should be bound to.
    ///
    /// Linux only. Everywhere else the rules go on every interface and there is
    /// nothing for a sweep or a run to be told.
    #[cfg(target_os = "linux")]
    pub fn interface(&self) -> &str {
        self.interfaces
            .get(self.selected_interface)
            .map(|s| s.as_str())
            .unwrap_or("any")
    }

    pub(crate) fn toggle_block_check(&mut self, index: usize) {
        let mut bc = self.autotune_config.block_checks.clone();
        bc.set(index, !bc.get(index));
        self.autotune_config.block_checks = bc;
    }
}
