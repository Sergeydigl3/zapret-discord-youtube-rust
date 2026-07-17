#[cfg(target_os = "linux")]
use crate::firewalls::LinuxBackend;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActiveScreen {
    Main,
    #[cfg(target_os = "windows")]
    DefenderSubmenu,
    StrategySubmenu,
    DownloadDepsSubmenu,
    DownloadZapretSubmenu,
    DownloadStrategiesSubmenu,
    GamefilterSubmenu,
    ZapretTagSelect,
    StrategyTagSelect,
    ServiceSubmenu,
    ListsEditorSubmenu,
    AutotuneSubmenu,
    AutotuneProtocolsSubmenu,
    AutotuneBlockChecksSubmenu,
    AutotunePresetSelectionSubmenu,
    AutotuneStrategiesSubmenu,
    AutotuneResultsSubmenu,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MainMenuState {
    #[cfg(target_os = "windows")]
    DefenderSettings,
    DownloadDeps,
    Interface,
    Strategy,
    GamefilterSettings,
    #[cfg(target_os = "linux")]
    BackendSettings,
    IpsetMode,
    ListsEditor,
    Autotune,
    ServiceSettings,
    Run,
    Quit,
}

impl MainMenuState {
    pub fn next(self) -> Self {
        match self {
            #[cfg(target_os = "windows")]
            Self::DefenderSettings => Self::DownloadDeps,
            Self::DownloadDeps => Self::Interface,
            Self::Interface => Self::Strategy,
            Self::Strategy => Self::GamefilterSettings,
            #[cfg(target_os = "linux")]
            Self::GamefilterSettings => Self::BackendSettings,
            #[cfg(target_os = "linux")]
            Self::BackendSettings => Self::IpsetMode,
            #[cfg(not(target_os = "linux"))]
            Self::GamefilterSettings => Self::IpsetMode,
            Self::IpsetMode => Self::ListsEditor,
            Self::ListsEditor => Self::Autotune,
            Self::Autotune => Self::ServiceSettings,
            Self::ServiceSettings => Self::Run,
            Self::Run => Self::Quit,
            #[cfg(target_os = "windows")]
            Self::Quit => Self::DefenderSettings,
            #[cfg(not(target_os = "windows"))]
            Self::Quit => Self::DownloadDeps,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            #[cfg(target_os = "windows")]
            Self::DefenderSettings => Self::Quit,
            #[cfg(target_os = "windows")]
            Self::DownloadDeps => Self::DefenderSettings,
            #[cfg(not(target_os = "windows"))]
            Self::DownloadDeps => Self::Quit,
            Self::Interface => Self::DownloadDeps,
            Self::Strategy => Self::Interface,
            Self::GamefilterSettings => Self::Strategy,
            #[cfg(target_os = "linux")]
            Self::BackendSettings => Self::GamefilterSettings,
            #[cfg(target_os = "linux")]
            Self::IpsetMode => Self::BackendSettings,
            #[cfg(not(target_os = "linux"))]
            Self::IpsetMode => Self::GamefilterSettings,
            Self::ListsEditor => Self::IpsetMode,
            Self::Autotune => Self::ListsEditor,
            Self::ServiceSettings => Self::Autotune,
            Self::Run => Self::ServiceSettings,
            Self::Quit => Self::Run,
        }
    }
}


#[derive(PartialEq, Clone, Copy)]
pub enum GamefilterMenuState {
    Tcp,
    Udp,
    Back,
}

impl GamefilterMenuState {
    pub fn next(self) -> Self {
        match self {
            Self::Tcp => Self::Udp,
            Self::Udp => Self::Back,
            Self::Back => Self::Tcp,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Tcp => Self::Back,
            Self::Udp => Self::Tcp,
            Self::Back => Self::Udp,
        }
    }
}

#[cfg(target_os = "windows")]
#[derive(PartialEq, Clone, Copy)]
pub enum DefenderMenuState {
    Add,
    Remove,
    Back,
}

#[cfg(target_os = "windows")]
impl DefenderMenuState {
    pub fn next(self) -> Self {
        match self {
            Self::Add => Self::Remove,
            Self::Remove => Self::Back,
            Self::Back => Self::Add,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Add => Self::Back,
            Self::Remove => Self::Add,
            Self::Back => Self::Remove,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AutotuneMenuState {
    PresetSelection,
    NumRequests,
    Strategies,
    Protocols,
    BlockChecks,
    EditCustom,
    Results,
    Run,
    Back,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AutotuneProtocolsState {
    Http,
    Https,
    Tls12,
    Tls13,
    Quic,
    Back,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AutotuneBlockChecksState {
    DnsSpoof,
    TcpRst,
    SniBlock,
    SiberianBlock,
    QuicBlock,
    CidrWhitelist,
    Back,
}

#[derive(PartialEq, Clone, Copy)]
pub enum DownloadDepsMenuState {
    ZapretDownloader,
    StrategiesDownloader,
    DownloadDefaults,
    Back,
}

impl DownloadDepsMenuState {
    pub fn next(self) -> Self {
        match self {
            Self::ZapretDownloader => Self::StrategiesDownloader,
            Self::StrategiesDownloader => Self::DownloadDefaults,
            Self::DownloadDefaults => Self::Back,
            Self::Back => Self::ZapretDownloader,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::ZapretDownloader => Self::Back,
            Self::StrategiesDownloader => Self::ZapretDownloader,
            Self::DownloadDefaults => Self::StrategiesDownloader,
            Self::Back => Self::DownloadDefaults,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum DownloadSubmenuState {
    Version,
    SelectTag,
    Start,
    Back,
}

impl DownloadSubmenuState {
    pub fn next(self) -> Self {
        match self {
            Self::Version => Self::SelectTag,
            Self::SelectTag => Self::Start,
            Self::Start => Self::Back,
            Self::Back => Self::Version,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Version => Self::Back,
            Self::SelectTag => Self::Version,
            Self::Start => Self::SelectTag,
            Self::Back => Self::Start,
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum VersionTarget {
    Recommended,
    Latest,
    Tag(String),
}

impl VersionTarget {
    pub fn cycle(&self, forward: bool) -> Self {
        if forward {
            match self {
                Self::Recommended => Self::Latest,
                Self::Latest => Self::Recommended,
                Self::Tag(_) => Self::Recommended,
            }
        } else {
            match self {
                Self::Recommended => Self::Latest,
                Self::Latest => Self::Recommended,
                Self::Tag(_) => Self::Latest,
            }
        }
    }
}

pub struct AppState {
    pub interfaces: Vec<String>,
    pub selected_interface: usize,

    #[cfg(target_os = "linux")]
    pub selected_backend: LinuxBackend,

    pub available_ipset_modes: Vec<crate::ipset::IpsetMode>,
    pub selected_ipset_mode: usize,

    pub strategies: Vec<String>,
    pub selected_strategy: usize,
    pub strategy_menu_index: usize,

    pub tcp_gamefilter: bool,
    pub udp_gamefilter: bool,

    pub active_screen: ActiveScreen,
    pub main_menu: MainMenuState,
    
    #[cfg(target_os = "windows")]
    pub defender_menu: DefenderMenuState,
    #[cfg(target_os = "windows")]
    pub defender_status_cache: Option<bool>,

    pub download_deps_menu: DownloadDepsMenuState,
    pub download_zapret_menu: DownloadSubmenuState,
    pub download_strategies_menu: DownloadSubmenuState,
    pub gamefilter_menu: GamefilterMenuState,
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

    pub autotune_config: crate::autotune::AutotuneConfig,
    pub autotune_results: Option<crate::autotune::AutotuneResults>,
    pub autotune_menu: AutotuneMenuState,
    pub autotune_menu_index: usize,
    pub autotune_protocols_menu: AutotuneProtocolsState,
    pub autotune_block_checks_menu: AutotuneBlockChecksState,
    pub autotune_preset_index: usize,
    pub autotune_strat_index: usize,
    pub autotune_results_index: usize,
    pub should_run_autotune: bool,
    pub autotune_running: bool,
    pub autotune_request_editing: bool,
    pub autotune_request_buf: String,
}

impl AppState {
    pub fn show_error(&mut self, msg: String) {
        self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), msg));
    }

    pub fn new(interfaces: Vec<String>, strategies: Vec<String>) -> Self {
        let _ = crate::config::ensure_default_config();

        let saved_cfg = crate::config::load_config(
            &crate::config::config_path().to_string_lossy()
        ).ok();

        let selected_interface = saved_cfg.as_ref().map_or(0, |cfg| {
            interfaces.iter().position(|i| i == &cfg.interface).unwrap_or(0)
        });
        let selected_strategy = saved_cfg.as_ref().map_or(0, |cfg| {
            strategies.iter().position(|s| s == &cfg.strategy).unwrap_or(0)
        });
        let tcp_gamefilter = saved_cfg.as_ref().map_or(false, |cfg| cfg.gamefilter_tcp);
        let udp_gamefilter = saved_cfg.as_ref().map_or(false, |cfg| cfg.gamefilter_udp);

        #[cfg(target_os = "linux")]
        let selected_backend = saved_cfg.as_ref().map_or_else(|| {
            LinuxBackend::from_config("nftables")
        }, |cfg| {
            LinuxBackend::from_config(&cfg.backend)
        });

        let available_ipset_modes = crate::ipset::get_available_modes();
        let current_ipset_mode = crate::ipset::determine_current_mode();
        let selected_ipset_mode = available_ipset_modes.iter().position(|m| m == &current_ipset_mode).unwrap_or(0);

        let mut app = Self {
            interfaces,
            selected_interface,
            #[cfg(target_os = "linux")]
            selected_backend,
            available_ipset_modes,
            selected_ipset_mode,
            strategies,
            selected_strategy,
            strategy_menu_index: selected_strategy,
            tcp_gamefilter,
            udp_gamefilter,
            active_screen: ActiveScreen::Main,
            
            #[cfg(target_os = "windows")]
            main_menu: MainMenuState::DefenderSettings,
            #[cfg(not(target_os = "windows"))]
            main_menu: MainMenuState::DownloadDeps,
            
            #[cfg(target_os = "windows")]
            defender_menu: DefenderMenuState::Add,
            #[cfg(target_os = "windows")]
            defender_status_cache: crate::defender::check_defender_exclusion().ok(),
            
            download_deps_menu: DownloadDepsMenuState::ZapretDownloader,
            download_zapret_menu: DownloadSubmenuState::Version,
            download_strategies_menu: DownloadSubmenuState::Version,
            gamefilter_menu: GamefilterMenuState::Tcp,
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

            nfqws_installed: crate::download::check_nfqws_installed(),
            strategies_installed: crate::download::check_strategies_installed(),

            service_installed: false,
            service_active: false,
            service_menu_index: 0,

            lists_files: Vec::new(),
            lists_menu_index: 0,
            should_open_editor: None,

            autotune_config: crate::autotune::AutotuneConfig::default(),
            autotune_results: None,
            autotune_menu: AutotuneMenuState::PresetSelection,
            autotune_menu_index: 0,
            autotune_protocols_menu: AutotuneProtocolsState::Http,
            autotune_block_checks_menu: AutotuneBlockChecksState::DnsSpoof,
            autotune_preset_index: 0,
            autotune_strat_index: 0,
            autotune_results_index: 0,
            should_run_autotune: false,
            autotune_running: false,
            autotune_request_editing: false,
            autotune_request_buf: String::new(),
        };
        app.refresh_service_status();
        app
    }

    pub fn refresh_dep_status(&mut self) {
        self.nfqws_installed = crate::download::check_nfqws_installed();
        self.strategies_installed = crate::download::check_strategies_installed();
    }

    #[cfg(target_os = "windows")]
    pub fn refresh_defender_status(&mut self) {
        self.defender_status_cache = crate::defender::check_defender_exclusion().ok();
    }

    pub fn refresh_service_status(&mut self) {
        #[cfg(target_os = "linux")]
        {
            if let Some(mgr) = crate::inits::get_detected_manager() {
                self.service_installed = mgr.is_installed();
                self.service_active = mgr.is_active();
            } else {
                self.service_installed = false;
                self.service_active = false;
            }
        }
        #[cfg(target_os = "windows")]
        {
            use crate::inits::ServiceManager;
            let mgr = crate::inits::winservice::WindowsServiceManager;
            self.service_installed = mgr.is_installed();
            self.service_active = mgr.is_active();
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            self.service_installed = false;
            self.service_active = false;
        }

        let count = self.get_service_menu_count();
        if count > 0 && self.service_menu_index >= count {
            self.service_menu_index = count - 1;
        }
    }

    pub fn refresh_ipset_status(&mut self) {
        self.available_ipset_modes = crate::ipset::get_available_modes();
        let current_ipset_mode = crate::ipset::determine_current_mode();
        self.selected_ipset_mode = self.available_ipset_modes.iter().position(|m| m == &current_ipset_mode).unwrap_or(0);
    }

    fn save_current_config(&self) {
        let interface = self.interfaces.get(self.selected_interface)
            .map(|s| s.as_str())
            .unwrap_or("any");
        let strategy = self.strategies.get(self.selected_strategy)
            .map(|s| s.as_str())
            .unwrap_or("");
        #[cfg(target_os = "linux")]
        let backend = self.selected_backend.to_config();
        #[cfg(not(target_os = "linux"))]
        let backend = "nftables";
        let _ = crate::config::save_tui_state(
            interface,
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

    pub fn next_menu(&mut self) {
        self.status_message = None;
        match self.active_screen {
            ActiveScreen::Main => self.main_menu = self.main_menu.next(),
            #[cfg(target_os = "windows")]
            ActiveScreen::DefenderSubmenu => self.defender_menu = self.defender_menu.next(),
            ActiveScreen::StrategySubmenu => {
                if !self.strategies.is_empty() {
                    self.strategy_menu_index = (self.strategy_menu_index + 1) % (self.strategies.len() + 1);
                }
            }
            ActiveScreen::DownloadDepsSubmenu => self.download_deps_menu = self.download_deps_menu.next(),
            ActiveScreen::DownloadZapretSubmenu => self.download_zapret_menu = self.download_zapret_menu.next(),
            ActiveScreen::DownloadStrategiesSubmenu => self.download_strategies_menu = self.download_strategies_menu.next(),
            ActiveScreen::GamefilterSubmenu => self.gamefilter_menu = self.gamefilter_menu.next(),
            ActiveScreen::ZapretTagSelect => {
                if !self.available_nfqws_tags.is_empty() {
                    self.nfqws_tag_index = (self.nfqws_tag_index + 1) % (self.available_nfqws_tags.len() + 1);
                }
            }
            ActiveScreen::StrategyTagSelect => {
                if !self.available_strat_tags.is_empty() {
                    self.strat_tag_index = (self.strat_tag_index + 1) % (self.available_strat_tags.len() + 1);
                }
            }
            ActiveScreen::ServiceSubmenu => {
                let count = self.get_service_menu_count();
                if count > 0 {
                    self.service_menu_index = (self.service_menu_index + 1) % count;
                }
            }
            ActiveScreen::ListsEditorSubmenu => {
                let max = self.lists_files.len() + 1; // +1 for Back
                self.lists_menu_index = (self.lists_menu_index + 1) % max;
            }
            ActiveScreen::AutotuneSubmenu => {
                let count = 9;
                self.autotune_menu_index = (self.autotune_menu_index + 1) % count;
                self.autotune_menu = match self.autotune_menu_index {
                    0 => AutotuneMenuState::PresetSelection,
                    1 => AutotuneMenuState::NumRequests,
                    2 => AutotuneMenuState::Strategies,
                    3 => AutotuneMenuState::Protocols,
                    4 => AutotuneMenuState::BlockChecks,
                    5 => AutotuneMenuState::EditCustom,
                    6 => AutotuneMenuState::Results,
                    7 => AutotuneMenuState::Run,
                    _ => AutotuneMenuState::Back,
                };
            }
            ActiveScreen::AutotuneProtocolsSubmenu => {
                self.autotune_protocols_menu = match self.autotune_protocols_menu {
                    AutotuneProtocolsState::Http => AutotuneProtocolsState::Https,
                    AutotuneProtocolsState::Https => AutotuneProtocolsState::Tls12,
                    AutotuneProtocolsState::Tls12 => AutotuneProtocolsState::Tls13,
                    AutotuneProtocolsState::Tls13 => AutotuneProtocolsState::Quic,
                    AutotuneProtocolsState::Quic => AutotuneProtocolsState::Back,
                    AutotuneProtocolsState::Back => AutotuneProtocolsState::Http,
                };
            }
            ActiveScreen::AutotunePresetSelectionSubmenu => {
                let total = crate::autotune::PRESETS.len() + 1;
                if total > 0 {
                    self.autotune_preset_index = (self.autotune_preset_index + 1) % total;
                }
            }
            ActiveScreen::AutotuneBlockChecksSubmenu => {
                self.autotune_block_checks_menu = match self.autotune_block_checks_menu {
                    AutotuneBlockChecksState::DnsSpoof => AutotuneBlockChecksState::TcpRst,
                    AutotuneBlockChecksState::TcpRst => AutotuneBlockChecksState::SniBlock,
                    AutotuneBlockChecksState::SniBlock => AutotuneBlockChecksState::SiberianBlock,
                    AutotuneBlockChecksState::SiberianBlock => AutotuneBlockChecksState::QuicBlock,
                    AutotuneBlockChecksState::QuicBlock => AutotuneBlockChecksState::CidrWhitelist,
                    AutotuneBlockChecksState::CidrWhitelist => AutotuneBlockChecksState::Back,
                    AutotuneBlockChecksState::Back => AutotuneBlockChecksState::DnsSpoof,
                };
            }
            ActiveScreen::AutotuneStrategiesSubmenu => {
                let max = self.strategies.len() + 1; // +1 for Back
                if max > 0 {
                    self.autotune_strat_index = (self.autotune_strat_index + 1) % max;
                }
            }
            ActiveScreen::AutotuneResultsSubmenu => {
                let total = self.count_results_items();
                if total > 0 && self.autotune_results_index + 1 < total {
                    self.autotune_results_index += 1;
                }
            }
        }
    }

    pub fn prev_menu(&mut self) {
        self.status_message = None;
        match self.active_screen {
            ActiveScreen::Main => self.main_menu = self.main_menu.prev(),
            #[cfg(target_os = "windows")]
            ActiveScreen::DefenderSubmenu => self.defender_menu = self.defender_menu.prev(),
            ActiveScreen::StrategySubmenu => {
                if !self.strategies.is_empty() {
                    let max = self.strategies.len() + 1;
                    self.strategy_menu_index = (self.strategy_menu_index + max - 1) % max;
                }
            }
            ActiveScreen::DownloadDepsSubmenu => self.download_deps_menu = self.download_deps_menu.prev(),
            ActiveScreen::DownloadZapretSubmenu => self.download_zapret_menu = self.download_zapret_menu.prev(),
            ActiveScreen::DownloadStrategiesSubmenu => self.download_strategies_menu = self.download_strategies_menu.prev(),
            ActiveScreen::GamefilterSubmenu => self.gamefilter_menu = self.gamefilter_menu.prev(),
            ActiveScreen::ZapretTagSelect => {
                if !self.available_nfqws_tags.is_empty() {
                    let max = self.available_nfqws_tags.len() + 1;
                    self.nfqws_tag_index = (self.nfqws_tag_index + max - 1) % max;
                }
            }
            ActiveScreen::StrategyTagSelect => {
                if !self.available_strat_tags.is_empty() {
                    let max = self.available_strat_tags.len() + 1;
                    self.strat_tag_index = (self.strat_tag_index + max - 1) % max;
                }
            }
            ActiveScreen::ServiceSubmenu => {
                let count = self.get_service_menu_count();
                if count > 0 {
                    self.service_menu_index = (self.service_menu_index + count - 1) % count;
                }
            }
            ActiveScreen::ListsEditorSubmenu => {
                let max = self.lists_files.len() + 1; // +1 for Back
                self.lists_menu_index = (self.lists_menu_index + max - 1) % max;
            }
            ActiveScreen::AutotuneSubmenu => {
                let count = 9;
                self.autotune_menu_index = (self.autotune_menu_index + count - 1) % count;
                self.autotune_menu = match self.autotune_menu_index {
                    0 => AutotuneMenuState::PresetSelection,
                    1 => AutotuneMenuState::NumRequests,
                    2 => AutotuneMenuState::Strategies,
                    3 => AutotuneMenuState::Protocols,
                    4 => AutotuneMenuState::BlockChecks,
                    5 => AutotuneMenuState::EditCustom,
                    6 => AutotuneMenuState::Results,
                    7 => AutotuneMenuState::Run,
                    _ => AutotuneMenuState::Back,
                };
            }
            ActiveScreen::AutotuneProtocolsSubmenu => {
                self.autotune_protocols_menu = match self.autotune_protocols_menu {
                    AutotuneProtocolsState::Http => AutotuneProtocolsState::Back,
                    AutotuneProtocolsState::Https => AutotuneProtocolsState::Http,
                    AutotuneProtocolsState::Tls12 => AutotuneProtocolsState::Https,
                    AutotuneProtocolsState::Tls13 => AutotuneProtocolsState::Tls12,
                    AutotuneProtocolsState::Quic => AutotuneProtocolsState::Tls13,
                    AutotuneProtocolsState::Back => AutotuneProtocolsState::Quic,
                };
            }
            ActiveScreen::AutotunePresetSelectionSubmenu => {
                let total = crate::autotune::PRESETS.len() + 1;
                self.autotune_preset_index = (self.autotune_preset_index + total - 1) % total;
            }
            ActiveScreen::AutotuneBlockChecksSubmenu => {
                self.autotune_block_checks_menu = match self.autotune_block_checks_menu {
                    AutotuneBlockChecksState::DnsSpoof => AutotuneBlockChecksState::Back,
                    AutotuneBlockChecksState::TcpRst => AutotuneBlockChecksState::DnsSpoof,
                    AutotuneBlockChecksState::SniBlock => AutotuneBlockChecksState::TcpRst,
                    AutotuneBlockChecksState::SiberianBlock => AutotuneBlockChecksState::SniBlock,
                    AutotuneBlockChecksState::QuicBlock => AutotuneBlockChecksState::SiberianBlock,
                    AutotuneBlockChecksState::CidrWhitelist => AutotuneBlockChecksState::QuicBlock,
                    AutotuneBlockChecksState::Back => AutotuneBlockChecksState::CidrWhitelist,
                };
            }
            ActiveScreen::AutotuneStrategiesSubmenu => {
                let max = self.strategies.len() + 1;
                if max > 0 {
                    self.autotune_strat_index = (self.autotune_strat_index + max - 1) % max;
                }
            }
            ActiveScreen::AutotuneResultsSubmenu => {
                let total = self.count_results_items();
                if total > 0 && self.autotune_results_index > 0 {
                    self.autotune_results_index -= 1;
                }
            }
        }
    }

    pub fn toggle_current(&mut self) {
        match self.active_screen {
            ActiveScreen::Main => {
                match self.main_menu {
                    #[cfg(target_os = "windows")]
                    MainMenuState::DefenderSettings => {
                        self.active_screen = ActiveScreen::DefenderSubmenu;
                        self.refresh_defender_status();
                        self.status_message = None;
                    }
                    MainMenuState::DownloadDeps => {
                        self.active_screen = ActiveScreen::DownloadDepsSubmenu;
                        self.status_message = None;
                    }
                    MainMenuState::Interface => {
                        if !self.interfaces.is_empty() {
                            self.selected_interface = (self.selected_interface + 1) % self.interfaces.len();
                            self.save_current_config();
                        }
                    }
                    #[cfg(target_os = "linux")]
                    MainMenuState::BackendSettings => {
                        let backends = LinuxBackend::variants();
                        if !backends.is_empty() {
                            let current_idx = backends.iter().position(|b| *b == self.selected_backend).unwrap_or(0);
                            let new_idx = (current_idx + 1) % backends.len();
                            self.selected_backend = backends[new_idx];
                            self.save_current_config();
                        }
                    }
                    MainMenuState::IpsetMode => {
                        if !self.available_ipset_modes.is_empty() {
                            let old_mode = self.available_ipset_modes[self.selected_ipset_mode];
                            self.selected_ipset_mode = (self.selected_ipset_mode + 1) % self.available_ipset_modes.len();
                            let new_mode = self.available_ipset_modes[self.selected_ipset_mode];
                            crate::ipset::apply_ipset_mode(old_mode, new_mode);
                            self.available_ipset_modes = crate::ipset::get_available_modes();
                            self.selected_ipset_mode = self.available_ipset_modes.iter().position(|m| m == &new_mode).unwrap_or(0);
                        }
                    }
                    MainMenuState::Strategy => {
                        self.active_screen = ActiveScreen::StrategySubmenu;
                        self.strategy_menu_index = self.selected_strategy;
                        self.status_message = None;
                    }
                    MainMenuState::GamefilterSettings => {
                        self.active_screen = ActiveScreen::GamefilterSubmenu;
                        self.gamefilter_menu = GamefilterMenuState::Tcp;
                        self.status_message = None;
                    }
                    MainMenuState::ServiceSettings => {
                        self.active_screen = ActiveScreen::ServiceSubmenu;
                        self.service_menu_index = 0;
                        self.refresh_service_status();
                        self.status_message = None;
                    }
                    MainMenuState::ListsEditor => {
                        if !crate::download::check_strategies_installed() {
                            self.show_error(rust_i18n::t!("err_no_strats").into_owned());
                        } else {
                            self.lists_files = crate::utils::get_lists_files();
                            self.lists_menu_index = 0;
                            self.active_screen = ActiveScreen::ListsEditorSubmenu;
                            self.status_message = None;
                        }
                    }
                    MainMenuState::Autotune => {
                        self.active_screen = ActiveScreen::AutotuneSubmenu;
                        self.autotune_menu_index = 0;
                        self.autotune_menu = AutotuneMenuState::PresetSelection;
                        self.status_message = None;
                    }
                    MainMenuState::Run => {
                        if self.check_dependencies() {
                            self.should_run = true;
                        }
                    }
                    MainMenuState::Quit => self.should_quit = true,
                }
            }
            #[cfg(target_os = "windows")]
             ActiveScreen::DefenderSubmenu => {
                match self.defender_menu {
                    DefenderMenuState::Add => {
                        match crate::defender::add_defender_exclusion() {
                            Ok(_) => {
                                self.status_message = Some(rust_i18n::t!("msg_def_add_ok").into_owned());
                                self.refresh_defender_status();
                            }
                            Err(e) => self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), e)),
                        }
                    }
                    DefenderMenuState::Remove => {
                        match crate::defender::remove_defender_exclusion() {
                            Ok(_) => {
                                self.status_message = Some(rust_i18n::t!("msg_def_rm_ok").into_owned());
                                self.refresh_defender_status();
                            }
                            Err(e) => self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_err"), e)),
                        }
                    }
                    DefenderMenuState::Back => {
                        self.active_screen = ActiveScreen::Main;
                        self.status_message = None;
                    }
                }
            }
            ActiveScreen::StrategySubmenu => {
                if self.strategy_menu_index < self.strategies.len() {
                    self.selected_strategy = self.strategy_menu_index;
                    self.save_current_config();
                    self.active_screen = ActiveScreen::Main;
                    self.status_message = Some(format!("{}{}", rust_i18n::t!("msg_strat_sel"), self.strategies[self.selected_strategy]));
                } else {
                    self.active_screen = ActiveScreen::Main;
                    self.status_message = None;
                }
            }
            ActiveScreen::DownloadDepsSubmenu => {
                match self.download_deps_menu {
                    DownloadDepsMenuState::ZapretDownloader => {
                        self.active_screen = ActiveScreen::DownloadZapretSubmenu;
                        self.download_zapret_menu = DownloadSubmenuState::Version;
                        self.status_message = None;
                    }
                    DownloadDepsMenuState::StrategiesDownloader => {
                        self.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
                        self.download_strategies_menu = DownloadSubmenuState::Version;
                        self.status_message = None;
                    }
                    DownloadDepsMenuState::DownloadDefaults => {
                        self.should_download_defaults = true;
                    }
                    DownloadDepsMenuState::Back => {
                        self.active_screen = ActiveScreen::Main;
                        self.status_message = None;
                    }
                }
            }
            ActiveScreen::DownloadZapretSubmenu => {
                match self.download_zapret_menu {
                    DownloadSubmenuState::Version => {
                        self.nfqws_target = self.nfqws_target.cycle(true);
                    }
                    DownloadSubmenuState::SelectTag => {
                        self.status_message = Some(rust_i18n::t!("msg_fetch_zapret_tags").into_owned());
                        match crate::download::fetch_repo_tags("bol-van/zapret") {
                            Ok(tags) => {
                                self.available_nfqws_tags = tags;
                                self.nfqws_tag_index = 0;
                                self.active_screen = ActiveScreen::ZapretTagSelect;
                                self.status_message = None;
                            }
                            Err(e) => {
                                self.show_error(format!("{}{}", rust_i18n::t!("msg_err_fetch_tags"), e));
                            }
                        }
                    }
                    DownloadSubmenuState::Start => {
                        self.should_download_zapret = true;
                    }
                    DownloadSubmenuState::Back => {
                        self.active_screen = ActiveScreen::DownloadDepsSubmenu;
                        self.status_message = None;
                    }
                }
            }
            ActiveScreen::DownloadStrategiesSubmenu => {
                match self.download_strategies_menu {
                    DownloadSubmenuState::Version => {
                        self.strat_target = self.strat_target.cycle(true);
                    }
                    DownloadSubmenuState::SelectTag => {
                        self.status_message = Some(rust_i18n::t!("msg_fetch_strat_tags").into_owned());
                        match crate::download::fetch_repo_tags("Flowseal/zapret-discord-youtube") {
                            Ok(tags) => {
                                self.available_strat_tags = tags;
                                self.strat_tag_index = 0;
                                self.active_screen = ActiveScreen::StrategyTagSelect;
                                self.status_message = None;
                            }
                            Err(e) => {
                                self.show_error(format!("{}{}", rust_i18n::t!("msg_err_fetch_tags"), e));
                            }
                        }
                    }
                    DownloadSubmenuState::Start => {
                        self.should_download_strategies = true;
                    }
                    DownloadSubmenuState::Back => {
                        self.active_screen = ActiveScreen::DownloadDepsSubmenu;
                        self.status_message = None;
                    }
                }
            }
            ActiveScreen::ZapretTagSelect => {
                if self.nfqws_tag_index < self.available_nfqws_tags.len() {
                    let selected = self.available_nfqws_tags[self.nfqws_tag_index].clone();
                    self.nfqws_target = VersionTarget::Tag(selected);
                    self.active_screen = ActiveScreen::DownloadZapretSubmenu;
                    self.status_message = Some(rust_i18n::t!("msg_zapret_tag_sel").into_owned());
                } else {
                    self.active_screen = ActiveScreen::DownloadZapretSubmenu;
                    self.status_message = None;
                }
            }
            ActiveScreen::StrategyTagSelect => {
                if self.strat_tag_index < self.available_strat_tags.len() {
                    let selected = self.available_strat_tags[self.strat_tag_index].clone();
                    self.strat_target = VersionTarget::Tag(selected);
                    self.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
                    self.status_message = Some(rust_i18n::t!("msg_strat_tag_sel").into_owned());
                } else {
                    self.active_screen = ActiveScreen::DownloadStrategiesSubmenu;
                    self.status_message = None;
                }
            }
            ActiveScreen::GamefilterSubmenu => {
                match self.gamefilter_menu {
                    GamefilterMenuState::Tcp => {
                        self.tcp_gamefilter = !self.tcp_gamefilter;
                        self.save_current_config();
                    }
                    GamefilterMenuState::Udp => {
                        self.udp_gamefilter = !self.udp_gamefilter;
                        self.save_current_config();
                    }
                    GamefilterMenuState::Back => {
                        self.active_screen = ActiveScreen::Main;
                        self.status_message = None;
                    }
                }
            }
            ActiveScreen::ServiceSubmenu => {
                #[cfg(target_os = "linux")]
                let mgr_opt = crate::inits::get_detected_manager();
                #[cfg(target_os = "windows")]
                let mgr_opt: Option<Box<dyn crate::inits::ServiceManager>> = Some(Box::new(crate::inits::winservice::WindowsServiceManager));
                #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                let mgr_opt: Option<Box<dyn crate::inits::ServiceManager>> = None;

                if let Some(mgr) = mgr_opt {
                    let mut action_taken = true;
                    let res = if !self.service_installed {
                        match self.service_menu_index {
                            0 => {
                                if !self.check_dependencies() {
                                    action_taken = false;
                                    Ok(())
                                } else {
                                    let exe_path = std::env::current_exe().map_err(|e| e.to_string());
                                    match exe_path {
                                        Ok(p) => {
                                            let config_path = crate::config::config_path();
                                            let cache_dir = crate::config::get_cache_dir();
                                            mgr.install(&p, &config_path, &cache_dir)
                                                .and_then(|_| mgr.start())
                                        }
                                        Err(e) => Err(e),
                                    }
                                }
                            }
                            1 => {
                                self.active_screen = ActiveScreen::Main;
                                self.status_message = None;
                                action_taken = false;
                                Ok(())
                            }
                            _ => {
                                action_taken = false;
                                Ok(())
                            }
                        }
                    } else if self.service_active {
                        match self.service_menu_index {
                            0 => mgr.stop(),
                            1 => {
                                if !self.check_dependencies() {
                                    action_taken = false;
                                    Ok(())
                                } else {
                                    mgr.restart()
                                }
                            }
                            2 => mgr.uninstall(),
                            3 => {
                                self.active_screen = ActiveScreen::Main;
                                self.status_message = None;
                                action_taken = false;
                                Ok(())
                            }
                            _ => {
                                action_taken = false;
                                Ok(())
                            }
                        }
                    } else {
                        match self.service_menu_index {
                            0 => mgr.start(),
                            1 => mgr.uninstall(),
                            2 => {
                                self.active_screen = ActiveScreen::Main;
                                self.status_message = None;
                                action_taken = false;
                                Ok(())
                            }
                            _ => {
                                action_taken = false;
                                Ok(())
                            }
                        }
                    };

                    if action_taken {
                        match res {
                            Ok(_) => {
                                self.refresh_service_status();
                                self.service_menu_index = 0;
                                self.status_message = Some(rust_i18n::t!("msg_op_ok").into_owned());
                            }
                            Err(e) => {
                                self.refresh_service_status();
                                self.show_error(e);
                            }
                        }
                    }
                } else {
                    self.status_message = Some(rust_i18n::t!("msg_err_init").into_owned());
                }
            }
            ActiveScreen::ListsEditorSubmenu => {
                if self.lists_menu_index < self.lists_files.len() {
                    let file = self.lists_files[self.lists_menu_index].clone();
                    self.should_open_editor = Some(file);
                } else {
                    self.active_screen = ActiveScreen::Main;
                    self.status_message = None;
                }
            }
            ActiveScreen::AutotuneSubmenu => {
                match self.autotune_menu {
                    AutotuneMenuState::PresetSelection => {}
                    AutotuneMenuState::NumRequests => {}
                    AutotuneMenuState::Strategies => {
                        if !self.strategies.is_empty() {
                            self.active_screen = ActiveScreen::AutotuneStrategiesSubmenu;
                            self.autotune_strat_index = 0;
                            self.status_message = None;
                        } else {
                            self.show_error(rust_i18n::t!("err_no_strats").into_owned());
                        }
                    }
                    AutotuneMenuState::Protocols => {
                        self.active_screen = ActiveScreen::AutotuneProtocolsSubmenu;
                        self.autotune_protocols_menu = AutotuneProtocolsState::Http;
                        self.status_message = None;
                    }
                    AutotuneMenuState::BlockChecks => {
                        self.active_screen = ActiveScreen::AutotuneBlockChecksSubmenu;
                        self.autotune_block_checks_menu = AutotuneBlockChecksState::DnsSpoof;
                        self.status_message = None;
                    }
                    AutotuneMenuState::EditCustom => {
                        let file = crate::autotune::custom_domains_file_path().to_string_lossy().into_owned();
                        self.should_open_editor = Some(file);
                    }
                    AutotuneMenuState::Results => {
                        self.active_screen = ActiveScreen::AutotuneResultsSubmenu;
                        self.autotune_results_index = 0;
                        self.status_message = None;
                    }
                    AutotuneMenuState::Run => {
                        if crate::platform::is_nfqws_running() {
                            self.status_message = Some(rust_i18n::t!("autotune_err_nfqws_running").into_owned());
                        } else {
                            self.should_run_autotune = true;
                        }
                    }
                    AutotuneMenuState::Back => {
                        self.active_screen = ActiveScreen::Main;
                        self.status_message = None;
                    }
                }
            }
            ActiveScreen::AutotuneProtocolsSubmenu => {
                match self.autotune_protocols_menu {
                    AutotuneProtocolsState::Http => {
                        self.autotune_config.check_http = !self.autotune_config.check_http;
                    }
                    AutotuneProtocolsState::Https => {
                        self.autotune_config.check_https = !self.autotune_config.check_https;
                    }
                    AutotuneProtocolsState::Tls12 => {
                        self.autotune_config.check_tls12 = !self.autotune_config.check_tls12;
                    }
                    AutotuneProtocolsState::Tls13 => {
                        self.autotune_config.check_tls13 = !self.autotune_config.check_tls13;
                    }
                    AutotuneProtocolsState::Quic => {
                        self.autotune_config.check_quic = !self.autotune_config.check_quic;
                    }
                    AutotuneProtocolsState::Back => {
                        self.active_screen = ActiveScreen::AutotuneSubmenu;
                        self.status_message = None;
                    }
                }
            }
            ActiveScreen::AutotuneBlockChecksSubmenu => {
                match self.autotune_block_checks_menu {
                    AutotuneBlockChecksState::DnsSpoof => {
                        let mut bc = self.autotune_config.block_checks.clone();
                        bc.set(0, !bc.get(0));
                        self.autotune_config.block_checks = bc;
                    }
                    AutotuneBlockChecksState::TcpRst => {
                        let mut bc = self.autotune_config.block_checks.clone();
                        bc.set(1, !bc.get(1));
                        self.autotune_config.block_checks = bc;
                    }
                    AutotuneBlockChecksState::SniBlock => {
                        let mut bc = self.autotune_config.block_checks.clone();
                        bc.set(2, !bc.get(2));
                        self.autotune_config.block_checks = bc;
                    }
                    AutotuneBlockChecksState::SiberianBlock => {
                        let mut bc = self.autotune_config.block_checks.clone();
                        bc.set(3, !bc.get(3));
                        self.autotune_config.block_checks = bc;
                    }
                    AutotuneBlockChecksState::QuicBlock => {
                        let mut bc = self.autotune_config.block_checks.clone();
                        bc.set(4, !bc.get(4));
                        self.autotune_config.block_checks = bc;
                    }
                    AutotuneBlockChecksState::CidrWhitelist => {
                        let mut bc = self.autotune_config.block_checks.clone();
                        bc.set(5, !bc.get(5));
                        self.autotune_config.block_checks = bc;
                    }
                    AutotuneBlockChecksState::Back => {
                        self.active_screen = ActiveScreen::AutotuneSubmenu;
                        self.status_message = None;
                    }
                }
            }
            ActiveScreen::AutotunePresetSelectionSubmenu => {
                let idx = self.autotune_preset_index;
                if idx >= crate::autotune::PRESETS.len() {
                    self.active_screen = ActiveScreen::AutotuneSubmenu;
                    self.status_message = None;
                    return;
                }
                let was_selected = self.autotune_config.preset_indices.contains(&idx);
                if was_selected {
                    self.autotune_config.preset_indices.retain(|&i| i != idx);
                } else {
                    self.autotune_config.preset_indices.push(idx);
                }
                self.autotune_config.preset_indices.sort();
                self.autotune_config.preset_indices.dedup();
                let names: Vec<&str> = self.autotune_config.preset_indices.iter()
                    .filter_map(|&i| if i < crate::autotune::PRESETS.len() {
                        Some(crate::autotune::PRESETS[i].name)
                    } else { None })
                    .collect();
                let label = if names.is_empty() {
                    rust_i18n::t!("menu_autotune_preset_none").into_owned()
                } else {
                    names.join(", ")
                };
                self.status_message = Some(format!(
                    "{}: {}",
                    rust_i18n::t!("autotune_preset_sel"),
                    label
                ));
            }
            ActiveScreen::AutotuneStrategiesSubmenu => {
                let max = self.strategies.len(); // +1 for Back
                if self.autotune_strat_index < max {
                    // Toggle strategy selection
                    let idx = self.autotune_strat_index;
                    if let Some(pos) = self.autotune_config.strategy_indices.iter().position(|&i| i == idx) {
                        self.autotune_config.strategy_indices.remove(pos);
                    } else {
                        self.autotune_config.strategy_indices.push(idx);
                    }
                } else {
                    // Back
                    self.active_screen = ActiveScreen::AutotuneSubmenu;
                    self.status_message = None;
                }
            }
            ActiveScreen::AutotuneResultsSubmenu => {
                self.active_screen = ActiveScreen::AutotuneSubmenu;
                self.status_message = None;
            }
        }
    }

    fn count_results_items(&self) -> usize {
        if let Some(ref results) = self.autotune_results {
            let mut n = 10; // header + 6 net checks + blank
            for pr in &results.preset_results {
                n += 1; // preset header
                n += pr.domain_checks.len();
                if !pr.strategy_results.is_empty() {
                    n += 1; // strategy header
                    for sr in &pr.strategy_results {
                        n += 1; // strategy name line
                        n += sr.domain_checks.len();
                    }
                    let working_count = pr.strategy_results.iter().filter(|s| s.works).count();
                    if working_count > 0 {
                        n += 1; // working summary header
                        n += working_count;
                    }
                }
                n += 1; // blank
            }
            if !results.common_strategies.is_empty() {
                n += 1; // common strategies header
                n += results.common_strategies.len();
                n += 1; // blank
            }
            n += 1; // back
            n
        } else if let Some(cached) = crate::autotune::load_results_file() {
            cached.lines().count() + 1 // lines + back
        } else {
            2 // "no data" line + back
        }
    }

    pub fn cycle_current(&mut self, forward: bool) {
        match self.active_screen {
            ActiveScreen::Main => {
                match self.main_menu {
                    MainMenuState::Interface => {
                        if !self.interfaces.is_empty() {
                            let len = self.interfaces.len();
                            if forward {
                                self.selected_interface = (self.selected_interface + 1) % len;
                            } else {
                                self.selected_interface = (self.selected_interface + len - 1) % len;
                            }
                            self.save_current_config();
                        }
                    }
                    #[cfg(target_os = "linux")]
                    MainMenuState::BackendSettings => {
                        let backends = LinuxBackend::variants();
                        if !backends.is_empty() {
                            let current_idx = backends.iter().position(|b| *b == self.selected_backend).unwrap_or(0);
                            let len = backends.len();
                            let new_idx = if forward {
                                (current_idx + 1) % len
                            } else {
                                (current_idx + len - 1) % len
                            };
                            self.selected_backend = backends[new_idx];
                            self.save_current_config();
                        }
                    }
                    MainMenuState::IpsetMode => {
                        if !self.available_ipset_modes.is_empty() {
                            let len = self.available_ipset_modes.len();
                            let old_mode = self.available_ipset_modes[self.selected_ipset_mode];
                            if forward {
                                self.selected_ipset_mode = (self.selected_ipset_mode + 1) % len;
                            } else {
                                self.selected_ipset_mode = (self.selected_ipset_mode + len - 1) % len;
                            }
                            let new_mode = self.available_ipset_modes[self.selected_ipset_mode];
                            crate::ipset::apply_ipset_mode(old_mode, new_mode);
                            self.available_ipset_modes = crate::ipset::get_available_modes();
                            self.selected_ipset_mode = self.available_ipset_modes.iter().position(|m| m == &new_mode).unwrap_or(0);
                        }
                    }
                    _ => {
                        if forward {
                            self.toggle_current();
                        }
                    }
                }
            }
            ActiveScreen::DownloadZapretSubmenu => {
                match self.download_zapret_menu {
                    DownloadSubmenuState::Version => {
                        self.nfqws_target = self.nfqws_target.cycle(forward);
                    }
                    _ => {
                        if forward {
                            self.toggle_current();
                        }
                    }
                }
            }
            ActiveScreen::DownloadStrategiesSubmenu => {
                match self.download_strategies_menu {
                    DownloadSubmenuState::Version => {
                        self.strat_target = self.strat_target.cycle(forward);
                    }
                    _ => {
                        if forward {
                            self.toggle_current();
                        }
                    }
                }
            }
            ActiveScreen::GamefilterSubmenu => {
                match self.gamefilter_menu {
                    GamefilterMenuState::Tcp => {
                        self.tcp_gamefilter = !self.tcp_gamefilter;
                        self.save_current_config();
                    }
                    GamefilterMenuState::Udp => {
                        self.udp_gamefilter = !self.udp_gamefilter;
                        self.save_current_config();
                    }
                    _ => {
                        if forward {
                            self.toggle_current();
                        }
                    }
                }
            }
            ActiveScreen::AutotuneSubmenu => {
                match self.autotune_menu {
                    AutotuneMenuState::PresetSelection => {
                        self.active_screen = ActiveScreen::AutotunePresetSelectionSubmenu;
                        let count = crate::autotune::PRESETS.len();
                        if self.autotune_preset_index >= count {
                            self.autotune_preset_index = count - 1;
                        }
                        self.status_message = None;
                    }
                    AutotuneMenuState::NumRequests => {
                        if forward {
                            self.autotune_request_buf = self.autotune_config.num_requests.to_string();
                            self.autotune_request_editing = true;
                        }
                    }
                    _ => {
                        if forward {
                            self.toggle_current();
                        }
                    }
                }
            }
            ActiveScreen::AutotuneProtocolsSubmenu => {
                match self.autotune_protocols_menu {
                    AutotuneProtocolsState::Http => {
                        self.autotune_config.check_http = !self.autotune_config.check_http;
                    }
                    AutotuneProtocolsState::Https => {
                        self.autotune_config.check_https = !self.autotune_config.check_https;
                    }
                    AutotuneProtocolsState::Tls12 => {
                        self.autotune_config.check_tls12 = !self.autotune_config.check_tls12;
                    }
                    AutotuneProtocolsState::Tls13 => {
                        self.autotune_config.check_tls13 = !self.autotune_config.check_tls13;
                    }
                    AutotuneProtocolsState::Quic => {
                        self.autotune_config.check_quic = !self.autotune_config.check_quic;
                    }
                    _ => {
                        if forward {
                            self.toggle_current();
                        }
                    }
                }
            }
             ActiveScreen::AutotuneBlockChecksSubmenu => {
                match self.autotune_block_checks_menu {
                    AutotuneBlockChecksState::Back => {
                        self.active_screen = ActiveScreen::AutotuneSubmenu;
                        self.status_message = None;
                    }
                    _ => {
                        self.toggle_current();
                    }
                }
            }
            ActiveScreen::AutotunePresetSelectionSubmenu => {
                self.toggle_current();
            }
            ActiveScreen::AutotuneStrategiesSubmenu => {
                self.toggle_current();
            }
            ActiveScreen::AutotuneResultsSubmenu => {
                self.active_screen = ActiveScreen::AutotuneSubmenu;
                self.status_message = None;
            }
            _ => {
                if forward {
                    self.toggle_current();
                }
            }
        }
    }
}
