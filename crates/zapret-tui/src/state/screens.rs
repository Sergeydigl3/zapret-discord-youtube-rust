//! The screen set: every screen's state enum, its cursor arithmetic, and the
//! up/down navigation between them.
//!
//! Two invariants hold across the set: a screen that ends in a "Back" row is
//! `rows.len()` deep rather than `rows.len() - 1`, and where a screen was opened
//! from is [`AppState::history`], not a second copy in the back handler.

use super::AppState;

/// Up/down over a menu's `ALL` list. The cursor is a position in an ordered list
/// of states, so every enum-based screen wants exactly this arithmetic, and the
/// mouse needs the same list to turn a drawn row back into a state.
macro_rules! cursor {
    () => {
        pub fn next(self) -> Self {
            step_in(&Self::ALL, self, true)
        }

        pub fn prev(self) -> Self {
            step_in(&Self::ALL, self, false)
        }
    };
    (index) => {
        cursor!();
        pub fn index(self) -> usize {
            position_in(&Self::ALL, self)
        }
    };
}

fn step_in<T: Copy + PartialEq>(all: &[T], current: T, forward: bool) -> T {
    if all.is_empty() {
        return current;
    }
    let len = all.len();
    let pos = position_in(all, current);
    all[(if forward { pos + 1 } else { pos + len - 1 }) % len]
}

fn position_in<T: Copy + PartialEq>(all: &[T], current: T) -> usize {
    all.iter().position(|s| *s == current).unwrap_or(0)
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActiveScreen {
    Main,
    #[cfg(target_os = "windows")]
    DefenderSubmenu,
    StrategySubmenu,
    DownloadDepsSubmenu,
    DownloadZapretSubmenu,
    DownloadStrategiesSubmenu,
    ZapretTagSelect,
    StrategyTagSelect,
    GamefilterSubmenu,
    ServiceSubmenu,
    ListsEditorSubmenu,
    /// The settings that are real but not part of the everyday run.
    ExtendedSubmenu,
    TtlSubmenu,
    FakesSubmenu,
    FakesSelectSubmenu,
    AutotuneSubmenu,
    AutotuneEditDomainsSubmenu,
    AutotuneProtocolsSubmenu,
    AutotuneBlockChecksSubmenu,
    AutotunePresetSelectionSubmenu,
    AutotuneStrategiesSubmenu,
    AutotuneResultsSubmenu,
}

impl ActiveScreen {
    /// What this screen is called, everywhere it is named.
    pub fn label(self) -> String {
        let key = match self {
            Self::Main => "tui_title_main",
            #[cfg(target_os = "windows")]
            Self::DefenderSubmenu => "tui_title_defender",
            Self::StrategySubmenu => "tui_title_strategy",
            Self::DownloadDepsSubmenu => "tui_title_download_cat",
            Self::DownloadZapretSubmenu => "tui_title_download_zapret",
            Self::DownloadStrategiesSubmenu => "tui_title_download_strat",
            Self::GamefilterSubmenu => "tui_title_gamefilter",
            Self::ExtendedSubmenu => "tui_title_extended",
            Self::TtlSubmenu => "tui_title_ttl",
            Self::FakesSubmenu => "tui_title_fakes",
            Self::FakesSelectSubmenu => "menu_fakes_select_title",
            Self::ZapretTagSelect => "tui_title_tag_zapret",
            Self::StrategyTagSelect => "tui_title_tag_strat",
            Self::ServiceSubmenu => "tui_title_service",
            Self::ListsEditorSubmenu => "tui_title_lists",
            Self::AutotuneSubmenu => "tui_title_autotune",
            Self::AutotuneEditDomainsSubmenu => "tui_title_autotune_edit_domains",
            Self::AutotuneProtocolsSubmenu => "tui_title_autotune_proto",
            Self::AutotuneBlockChecksSubmenu => "tui_title_autotune_bc",
            Self::AutotunePresetSelectionSubmenu => "tui_title_autotune_presets",
            Self::AutotuneStrategiesSubmenu => "tui_title_autotune_strat",
            Self::AutotuneResultsSubmenu => "tui_title_autotune_results",
        };
        rust_i18n::t!(key).into_owned()
    }
}

/// Where the user has been, so Esc can walk back out the way they walked in.
#[derive(Default)]
pub struct History {
    screens: Vec<ActiveScreen>,
}

impl History {
    /// Record where a screen was opened from. Consecutive duplicates would make
    /// Esc appear to do nothing, so they are dropped.
    pub fn push(&mut self, from: ActiveScreen) {
        if self.screens.last() != Some(&from) {
            self.screens.push(from);
        }
    }

    /// The screen to go back to, or `None` at the main menu.
    pub fn pop(&mut self) -> Option<ActiveScreen> {
        self.screens.pop()
    }

    pub fn clear(&mut self) {
        self.screens.clear();
    }

    /// The route from the main menu down to `to`, oldest first.
    ///
    /// `to` is appended rather than looked for: the stack holds where the user
    /// came *from*, so the current screen is never in it.
    pub fn trail(&self, to: ActiveScreen) -> Vec<ActiveScreen> {
        if to == ActiveScreen::Main {
            return vec![to];
        }
        let mut trail: Vec<ActiveScreen> = self.screens.iter().copied().filter(|s| *s != to).collect();
        if trail.first() != Some(&ActiveScreen::Main) {
            trail.insert(0, ActiveScreen::Main);
        }
        trail.push(to);
        trail
    }
}

/// The main menu. `Interface`, `BackendSettings` and `Router` are Linux-only:
/// nftables and iptables can bind the rules to one output device and WinDivert
/// cannot, so on Windows those rows do not exist rather than exist and do
/// nothing.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MainMenuState {
    #[cfg(target_os = "windows")]
    DefenderSettings,
    DownloadDeps,
    #[cfg(target_os = "linux")]
    Interface,
    Strategy,
    GamefilterSettings,
    #[cfg(target_os = "linux")]
    BackendSettings,
    IpsetMode,
    ListsEditor,
    Autotune,
    Extended,
    ServiceSettings,
    Run,
    Quit,
}

/// A named group of main-menu rows.
pub struct Group {
    /// The locale key of the heading.
    pub title: &'static str,
    pub rows: &'static [MainMenuState],
}

impl MainMenuState {
    /// The single source of truth: the menu is rendered from this list and the
    /// cursor is this list's index. Headings are drawn from it but never land
    /// the cursor.
    pub const GROUPS: &'static [Group] = &[
        Group {
            title: "menu_group_setup",
            rows: &[Self::DownloadDeps, Self::ListsEditor],
        },
        Group {
            title: "menu_group_network",
            rows: &[
                #[cfg(target_os = "linux")]
                Self::Interface,
                Self::GamefilterSettings,
                #[cfg(target_os = "linux")]
                Self::BackendSettings,
                Self::IpsetMode,
                Self::Extended,
            ],
        },
        Group {
            title: "menu_group_system",
            rows: &[
                #[cfg(target_os = "windows")]
                Self::DefenderSettings,
                Self::ServiceSettings,
            ],
        },
        Group {
            title: "menu_group_strategy",
            rows: &[Self::Strategy, Self::Autotune],
        },
        // No heading: a heading above "Run" would name a group of one decision.
        Group {
            title: "",
            rows: &[Self::Run, Self::Quit],
        },
    ];

    /// Every selectable row, in the order the menu draws them.
    pub fn all() -> impl Iterator<Item = Self> {
        Self::GROUPS.iter().flat_map(|group| group.rows.iter().copied())
    }

    /// The row the cursor starts on: the top of the menu, whichever row that
    /// happens to be on this platform.
    pub fn first() -> Self {
        Self::all().next().unwrap_or(Self::Quit)
    }

    pub fn next(self) -> Self {
        let all: Vec<Self> = Self::all().collect();
        step_in(&all, self, true)
    }

    pub fn prev(self) -> Self {
        let all: Vec<Self> = Self::all().collect();
        step_in(&all, self, false)
    }
}

/// The settings under the Extended submenu, plus the way back.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ExtendedMenuState {
    Ttl,
    Fakes,
    /// Make this machine a gateway for another device. Linux-only: the
    /// forwarding and masquerade rules only exist there.
    #[cfg(target_os = "linux")]
    Router,
    Back,
}

impl ExtendedMenuState {
    #[cfg(target_os = "linux")]
    pub const ALL: [Self; 4] = [Self::Ttl, Self::Fakes, Self::Router, Self::Back];
    #[cfg(not(target_os = "linux"))]
    pub const ALL: [Self; 3] = [Self::Ttl, Self::Fakes, Self::Back];

    cursor!(index);
}

/// Leave the DPI-TTL alone, pin an explicit hop count, or sweep for one.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TtlMenuState {
    DontTouch,
    SetValue,
    Autopick,
    Back,
}

impl TtlMenuState {
    pub const ALL: [Self; 4] = [Self::DontTouch, Self::SetValue, Self::Autopick, Self::Back];

    cursor!(index);
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AutotuneReportTab {
    /// Which strategy to pick.
    Summary,
    /// Every preset, every domain, every protocol.
    Details,
}

impl AutotuneReportTab {
    pub const ALL: [Self; 2] = [Self::Summary, Self::Details];

    pub fn label(self) -> String {
        match self {
            Self::Summary => rust_i18n::t!("atv_tab_summary").to_string(),
            Self::Details => rust_i18n::t!("atv_tab_details").to_string(),
        }
    }

    cursor!();
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GamefilterMenuState {
    Tcp,
    Udp,
    Back,
}

impl GamefilterMenuState {
    pub const ALL: [Self; 3] = [Self::Tcp, Self::Udp, Self::Back];

    cursor!(index);
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FakesMenuState {
    DiscordUdp,
    GameUdp,
    Back,
}

impl FakesMenuState {
    pub const ALL: [Self; 3] = [Self::DiscordUdp, Self::GameUdp, Self::Back];

    cursor!(index);
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FakesSelectTarget {
    DiscordUdp,
    GameUdp,
}

#[cfg(target_os = "windows")]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DefenderMenuState {
    Add,
    Remove,
    Back,
}

#[cfg(target_os = "windows")]
impl DefenderMenuState {
    pub const ALL: [Self; 3] = [Self::Add, Self::Remove, Self::Back];

    cursor!(index);
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

impl AutotuneMenuState {
    pub const ALL: [Self; 9] = [
        Self::PresetSelection,
        Self::NumRequests,
        Self::Strategies,
        Self::Protocols,
        Self::BlockChecks,
        Self::EditDomains,
        Self::Results,
        Self::Run,
        Self::Back,
    ];

    cursor!(index);
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
    pub const ALL: [Self; 5] = [Self::Http, Self::Tls12, Self::Tls13, Self::Quic, Self::Back];

    cursor!(index);
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
    /// The six checks, in the order they are drawn. `Back` is not one: it is a
    /// row below them, not a seventh check.
    pub const CHECKS: [Self; 6] = [
        Self::DnsSpoof,
        Self::TcpRst,
        Self::SniBlock,
        Self::SiberianBlock,
        Self::QuicBlock,
        Self::CidrWhitelist,
    ];
    pub const ALL: [Self; 7] = [
        Self::DnsSpoof,
        Self::TcpRst,
        Self::SniBlock,
        Self::SiberianBlock,
        Self::QuicBlock,
        Self::CidrWhitelist,
        Self::Back,
    ];

    cursor!(index);

    /// Which check this row toggles, or `None` for the way back.
    pub fn check_index(self) -> Option<usize> {
        Self::CHECKS.iter().position(|c| *c == self)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DownloadDepsMenuState {
    ZapretDownloader,
    StrategiesDownloader,
    DownloadDefaults,
    Back,
}

impl DownloadDepsMenuState {
    pub const ALL: [Self; 4] = [
        Self::ZapretDownloader,
        Self::StrategiesDownloader,
        Self::DownloadDefaults,
        Self::Back,
    ];

    cursor!(index);
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DownloadSubmenuState {
    Version,
    SelectTag,
    Start,
    Back,
}

impl DownloadSubmenuState {
    pub const ALL: [Self; 4] = [Self::Version, Self::SelectTag, Self::Start, Self::Back];

    cursor!(index);
}

#[derive(PartialEq, Clone)]
pub enum VersionTarget {
    Recommended,
    Latest,
    Tag(String),
}

impl VersionTarget {
    /// Recommended and Latest swap on either arrow; a tag is only left, and the
    /// direction decides whether leaving it lands on Recommended or on Latest.
    pub fn cycle(&self, forward: bool) -> Self {
        match self {
            Self::Recommended => Self::Latest,
            Self::Latest => Self::Recommended,
            Self::Tag(_) if !forward => Self::Latest,
            Self::Tag(_) => Self::Recommended,
        }
    }
}

impl AppState {
    /// Go one step down into `screen`, remembering where we came from.
    pub fn open(&mut self, screen: ActiveScreen) {
        self.history.push(self.active_screen);
        self.active_screen = screen;
        self.status_message = None;
    }

    /// Go one step up. False at the main menu, which is where the back stack
    /// runs out and Esc means "leave".
    pub fn back(&mut self) -> bool {
        match self.history.pop() {
            Some(previous) => {
                self.active_screen = previous;
                self.status_message = None;
                true
            }
            None => false,
        }
    }

    /// Put the user back on the main menu and forget how they got here: for a
    /// job that finished and dropped the user out of a submenu tree there is
    /// nothing left to go back to.
    pub fn home(&mut self) {
        self.history.clear();
        self.active_screen = ActiveScreen::Main;
    }

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
        macro_rules! enum_step {
            ($field:ident) => {{
                self.$field = if forward {
                    self.$field.next()
                } else {
                    self.$field.prev()
                };
            }};
        }

        self.status_message = None;
        match self.active_screen {
            ActiveScreen::Main => enum_step!(main_menu),
            #[cfg(target_os = "windows")]
            ActiveScreen::DefenderSubmenu => enum_step!(defender_menu),
            ActiveScreen::DownloadDepsSubmenu => enum_step!(download_deps_menu),
            ActiveScreen::DownloadZapretSubmenu => enum_step!(download_zapret_menu),
            ActiveScreen::DownloadStrategiesSubmenu => enum_step!(download_strategies_menu),
            ActiveScreen::GamefilterSubmenu => enum_step!(gamefilter_menu),
            ActiveScreen::ExtendedSubmenu => enum_step!(extended_menu),
            ActiveScreen::TtlSubmenu => enum_step!(ttl_menu),
            ActiveScreen::FakesSubmenu => enum_step!(fakes_menu),
            ActiveScreen::AutotuneProtocolsSubmenu => enum_step!(autotune_protocols_menu),
            ActiveScreen::AutotuneBlockChecksSubmenu => enum_step!(autotune_block_checks_menu),
            ActiveScreen::AutotuneSubmenu => {
                enum_step!(autotune_menu);
                self.autotune_menu_index = self.autotune_menu.index();
            }
            ActiveScreen::StrategySubmenu if !self.strategies.is_empty() => {
                self.strategy_menu_index =
                    Self::cycle_index(self.strategy_menu_index, self.strategies.len() + 1, forward);
            }
            ActiveScreen::ZapretTagSelect if !self.available_nfqws_tags.is_empty() => {
                self.nfqws_tag_index =
                    Self::cycle_index(self.nfqws_tag_index, self.available_nfqws_tags.len() + 1, forward);
            }
            ActiveScreen::StrategyTagSelect if !self.available_strat_tags.is_empty() => {
                self.strat_tag_index =
                    Self::cycle_index(self.strat_tag_index, self.available_strat_tags.len() + 1, forward);
            }
            ActiveScreen::ServiceSubmenu => {
                let count = self.get_service_menu_count();
                if count > 0 {
                    self.service_menu_index = Self::cycle_index(self.service_menu_index, count, forward);
                }
            }
            ActiveScreen::FakesSelectSubmenu => {
                self.fakes_select_index =
                    Self::cycle_index(self.fakes_select_index, self.fakes_state.available.len() + 2, forward);
            }
            ActiveScreen::ListsEditorSubmenu => {
                // +1 for Back
                self.lists_menu_index = Self::cycle_index(self.lists_menu_index, self.lists_files.len() + 1, forward);
            }
            ActiveScreen::AutotuneEditDomainsSubmenu => {
                // +1 for Back
                self.domain_files_index =
                    Self::cycle_index(self.domain_files_index, self.domain_files.len() + 1, forward);
            }
            ActiveScreen::AutotuneStrategiesSubmenu => {
                // +1 for Back
                self.autotune_strat_index =
                    Self::cycle_index(self.autotune_strat_index, self.strategies.len() + 1, forward);
            }
            ActiveScreen::AutotunePresetSelectionSubmenu => {
                self.autotune_preset_index = Self::cycle_index(
                    self.autotune_preset_index,
                    zapret_wrapper::domains::PRESETS.len() + 1,
                    forward,
                );
            }
            ActiveScreen::AutotuneResultsSubmenu => {
                // The report is a scrolling view, not a menu: the offset has no
                // cursor to sit on, and `views::report` pulls it back into range
                // once it knows how tall the tables turned out to be.
                self.scroll_report(forward, 1);
            }
            // An empty list has no cursor to move, so the guarded arms above
            // match nothing and these three screens have nothing to do.
            ActiveScreen::StrategySubmenu | ActiveScreen::ZapretTagSelect | ActiveScreen::StrategyTagSelect => {}
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

    /// Scroll the autotune report by `lines`, never above the top.
    pub fn scroll_report(&mut self, forward: bool, lines: usize) {
        self.autotune_results_index = if forward {
            self.autotune_results_index.saturating_add(lines)
        } else {
            self.autotune_results_index.saturating_sub(lines)
        };
    }

    /// Switch the autotune report between its short and long form.
    ///
    /// The offset is dropped rather than kept: the two tabs have nothing in
    /// common vertically, so landing halfway down a freshly opened one would
    /// just look broken.
    pub fn switch_report_tab(&mut self, forward: bool) {
        self.autotune_report_tab = if forward {
            self.autotune_report_tab.next()
        } else {
            self.autotune_report_tab.prev()
        };
        self.autotune_results_index = 0;
    }
}
