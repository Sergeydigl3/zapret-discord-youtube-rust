//! Screens and their per-menu cursors, plus the up/down navigation.
//!
//! Every screen owns a small state enum here. Keeping the enums in one place
//! makes the screen set easy to audit: adding a screen means adding a variant
//! here, a handler in `actions`, and a render branch in `menus`.

use super::AppState;

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
    FakesSubmenu,
    FakesSelectSubmenu,
    ZapretTagSelect,
    StrategyTagSelect,
    ServiceSubmenu,
    ListsEditorSubmenu,
    AutotuneSubmenu,
    AutotuneEditDomainsSubmenu,
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
    TtlAutopick,
    FakesSettings,
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
            Self::Autotune => Self::TtlAutopick,
            Self::TtlAutopick => Self::FakesSettings,
            Self::FakesSettings => Self::ServiceSettings,
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
            Self::TtlAutopick => Self::Autotune,
            Self::FakesSettings => Self::TtlAutopick,
            Self::ServiceSettings => Self::FakesSettings,
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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FakesMenuState {
    DiscordUdp,
    GameUdp,
    Back,
}

impl FakesMenuState {
    pub fn next(self) -> Self {
        match self {
            Self::DiscordUdp => Self::GameUdp,
            Self::GameUdp => Self::Back,
            Self::Back => Self::DiscordUdp,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::DiscordUdp => Self::Back,
            Self::GameUdp => Self::DiscordUdp,
            Self::Back => Self::GameUdp,
        }
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FakesSelectTarget {
    DiscordUdp,
    GameUdp,
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
    EditDomains,
    Results,
    Run,
    Back,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AutotuneProtocolsState {
    Http,
    Tls12,
    Tls13,
    Quic,
    Back,
}

impl AutotuneProtocolsState {
    pub fn next(self) -> Self {
        match self {
            Self::Http => Self::Tls12,
            Self::Tls12 => Self::Tls13,
            Self::Tls13 => Self::Quic,
            Self::Quic => Self::Back,
            Self::Back => Self::Http,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Http => Self::Back,
            Self::Tls12 => Self::Http,
            Self::Tls13 => Self::Tls12,
            Self::Quic => Self::Tls13,
            Self::Back => Self::Quic,
        }
    }
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

impl AutotuneBlockChecksState {
    pub fn next(self) -> Self {
        match self {
            Self::DnsSpoof => Self::TcpRst,
            Self::TcpRst => Self::SniBlock,
            Self::SniBlock => Self::SiberianBlock,
            Self::SiberianBlock => Self::QuicBlock,
            Self::QuicBlock => Self::CidrWhitelist,
            Self::CidrWhitelist => Self::Back,
            Self::Back => Self::DnsSpoof,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::DnsSpoof => Self::Back,
            Self::TcpRst => Self::DnsSpoof,
            Self::SniBlock => Self::TcpRst,
            Self::SiberianBlock => Self::SniBlock,
            Self::QuicBlock => Self::SiberianBlock,
            Self::CidrWhitelist => Self::QuicBlock,
            Self::Back => Self::CidrWhitelist,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::DnsSpoof => 0,
            Self::TcpRst => 1,
            Self::SniBlock => 2,
            Self::SiberianBlock => 3,
            Self::QuicBlock => 4,
            Self::CidrWhitelist => 5,
            Self::Back => unreachable!("Back has no block-check index"),
        }
    }
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
                Self::Latest | Self::Tag(_) => Self::Recommended,
            }
        } else {
            match self {
                Self::Recommended | Self::Tag(_) => Self::Latest,
                Self::Latest => Self::Recommended,
            }
        }
    }
}

impl AppState {
    pub fn next_menu(&mut self) {
        self.move_menu(true);
    }

    pub fn prev_menu(&mut self) {
        self.move_menu(false);
    }

    /// Move the cursor of the current screen up or down.
    ///
    /// A screen whose cursor is a plain `usize` counts a trailing "Back" row, so
    /// the bound is `len + 1` there and `len` for the enum-based menus.
    pub fn move_menu(&mut self, forward: bool) {
        self.status_message = None;
        match self.active_screen {
            ActiveScreen::Main => {
                self.main_menu = if forward {
                    self.main_menu.next()
                } else {
                    self.main_menu.prev()
                }
            }
            #[cfg(target_os = "windows")]
            ActiveScreen::DefenderSubmenu => {
                self.defender_menu = if forward {
                    self.defender_menu.next()
                } else {
                    self.defender_menu.prev()
                };
            }
            ActiveScreen::StrategySubmenu => {
                if !self.strategies.is_empty() {
                    let max = self.strategies.len() + 1;
                    self.strategy_menu_index = Self::cycle_index(self.strategy_menu_index, max, forward);
                }
            }
            ActiveScreen::DownloadDepsSubmenu => {
                self.download_deps_menu = if forward {
                    self.download_deps_menu.next()
                } else {
                    self.download_deps_menu.prev()
                };
            }
            ActiveScreen::DownloadZapretSubmenu => {
                self.download_zapret_menu = if forward {
                    self.download_zapret_menu.next()
                } else {
                    self.download_zapret_menu.prev()
                };
            }
            ActiveScreen::DownloadStrategiesSubmenu => {
                self.download_strategies_menu = if forward {
                    self.download_strategies_menu.next()
                } else {
                    self.download_strategies_menu.prev()
                };
            }
            ActiveScreen::GamefilterSubmenu => {
                self.gamefilter_menu = if forward {
                    self.gamefilter_menu.next()
                } else {
                    self.gamefilter_menu.prev()
                };
            }
            ActiveScreen::FakesSubmenu => {
                self.fakes_menu = if forward {
                    self.fakes_menu.next()
                } else {
                    self.fakes_menu.prev()
                }
            }
            ActiveScreen::FakesSelectSubmenu => {
                let max = self.fakes_state.available.len() + 2;
                if max > 0 {
                    self.fakes_select_index = Self::cycle_index(self.fakes_select_index, max, forward);
                }
            }
            ActiveScreen::ZapretTagSelect => {
                if !self.available_nfqws_tags.is_empty() {
                    let max = self.available_nfqws_tags.len() + 1;
                    self.nfqws_tag_index = Self::cycle_index(self.nfqws_tag_index, max, forward);
                }
            }
            ActiveScreen::StrategyTagSelect => {
                if !self.available_strat_tags.is_empty() {
                    let max = self.available_strat_tags.len() + 1;
                    self.strat_tag_index = Self::cycle_index(self.strat_tag_index, max, forward);
                }
            }
            ActiveScreen::ServiceSubmenu => {
                let count = self.get_service_menu_count();
                if count > 0 {
                    self.service_menu_index = Self::cycle_index(self.service_menu_index, count, forward);
                }
            }
            ActiveScreen::ListsEditorSubmenu => {
                let max = self.lists_files.len() + 1; // +1 for Back
                self.lists_menu_index = Self::cycle_index(self.lists_menu_index, max, forward);
            }
            ActiveScreen::AutotuneEditDomainsSubmenu => {
                let max = self.domain_files.len() + 1; // +1 for Back
                self.domain_files_index = Self::cycle_index(self.domain_files_index, max, forward);
            }
            ActiveScreen::AutotuneSubmenu => {
                let count = 9;
                self.set_autotune_menu_index(Self::cycle_index(self.autotune_menu_index, count, forward));
            }
            ActiveScreen::AutotuneProtocolsSubmenu => {
                self.autotune_protocols_menu = if forward {
                    self.autotune_protocols_menu.next()
                } else {
                    self.autotune_protocols_menu.prev()
                };
            }
            ActiveScreen::AutotunePresetSelectionSubmenu => {
                let total = zapret_core::domains::PRESETS.len() + 1;
                if total > 0 {
                    self.autotune_preset_index = Self::cycle_index(self.autotune_preset_index, total, forward);
                }
            }
            ActiveScreen::AutotuneBlockChecksSubmenu => {
                self.autotune_block_checks_menu = if forward {
                    self.autotune_block_checks_menu.next()
                } else {
                    self.autotune_block_checks_menu.prev()
                };
            }
            ActiveScreen::AutotuneStrategiesSubmenu => {
                let max = self.strategies.len() + 1; // +1 for Back
                if max > 0 {
                    self.autotune_strat_index = Self::cycle_index(self.autotune_strat_index, max, forward);
                }
            }
            ActiveScreen::AutotuneResultsSubmenu => {
                let total = self.count_results_items();
                if total > 0 {
                    if forward {
                        if self.autotune_results_index + 1 < total {
                            self.autotune_results_index += 1;
                        }
                    } else if self.autotune_results_index > 0 {
                        self.autotune_results_index -= 1;
                    }
                }
            }
        }
    }

    pub(crate) fn cycle_index(current: usize, max: usize, forward: bool) -> usize {
        if max == 0 {
            return current;
        }
        if forward {
            (current + 1) % max
        } else {
            (current + max - 1) % max
        }
    }

    pub(crate) fn set_autotune_menu_index(&mut self, index: usize) {
        self.autotune_menu_index = index % 9;
        self.autotune_menu = match self.autotune_menu_index {
            0 => AutotuneMenuState::PresetSelection,
            1 => AutotuneMenuState::NumRequests,
            2 => AutotuneMenuState::Strategies,
            3 => AutotuneMenuState::Protocols,
            4 => AutotuneMenuState::BlockChecks,
            5 => AutotuneMenuState::EditDomains,
            6 => AutotuneMenuState::Results,
            7 => AutotuneMenuState::Run,
            _ => AutotuneMenuState::Back,
        };
    }

    pub fn is_ttl_autopick_selected(&self) -> bool {
        self.active_screen == ActiveScreen::Main && self.main_menu == MainMenuState::TtlAutopick
    }
}
