//! Screens and their per-menu cursors, plus the up/down navigation.
//!
//! Every screen owns a small state enum here. Keeping the enums in one place
//! makes the screen set easy to audit: adding a screen means adding a variant
//! here, a handler in `actions`, and a render branch in `menus`.
//!
//! Two invariants hold across the whole set:
//!
//! - A screen that ends in a "Back" row is `rows.len()` deep, not
//!   `rows.len() - 1`, so the cursor has somewhere to sit on the way out.
//! - Where a screen came from is [`AppState::history`], not a second copy of
//!   that knowledge in the back handler.

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
    ZapretTagSelect,
    StrategyTagSelect,
    GamefilterSubmenu,
    ServiceSubmenu,
    ListsEditorSubmenu,
    /// The settings that are real but not part of the everyday run: the DPI TTL
    /// and the fake payloads. They used to be two rows on the main menu, which
    /// made the menu long enough that the things you touch most were the
    /// hardest ones to find.
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
    /// What this screen is called, everywhere it is named: the breadcrumb, the
    /// download version rows, nowhere else.
    pub fn label(self) -> String {
        match self {
            Self::Main => rust_i18n::t!("tui_title_main").into_owned(),
            #[cfg(target_os = "windows")]
            Self::DefenderSubmenu => rust_i18n::t!("tui_title_defender").into_owned(),
            Self::StrategySubmenu => rust_i18n::t!("tui_title_strategy").into_owned(),
            Self::DownloadDepsSubmenu => rust_i18n::t!("tui_title_download_cat").into_owned(),
            Self::DownloadZapretSubmenu => rust_i18n::t!("tui_title_download_zapret").into_owned(),
            Self::DownloadStrategiesSubmenu => rust_i18n::t!("tui_title_download_strat").into_owned(),
            Self::GamefilterSubmenu => rust_i18n::t!("tui_title_gamefilter").into_owned(),
            Self::ExtendedSubmenu => rust_i18n::t!("tui_title_extended").into_owned(),
            Self::TtlSubmenu => rust_i18n::t!("tui_title_ttl").into_owned(),
            Self::FakesSubmenu => rust_i18n::t!("tui_title_fakes").into_owned(),
            Self::FakesSelectSubmenu => rust_i18n::t!("menu_fakes_select_title").into_owned(),
            Self::ZapretTagSelect => rust_i18n::t!("tui_title_tag_zapret").into_owned(),
            Self::StrategyTagSelect => rust_i18n::t!("tui_title_tag_strat").into_owned(),
            Self::ServiceSubmenu => rust_i18n::t!("tui_title_service").into_owned(),
            Self::ListsEditorSubmenu => rust_i18n::t!("tui_title_lists").into_owned(),
            Self::AutotuneSubmenu => rust_i18n::t!("tui_title_autotune").into_owned(),
            Self::AutotuneEditDomainsSubmenu => rust_i18n::t!("tui_title_autotune_edit_domains").into_owned(),
            Self::AutotuneProtocolsSubmenu => rust_i18n::t!("tui_title_autotune_proto").into_owned(),
            Self::AutotuneBlockChecksSubmenu => rust_i18n::t!("tui_title_autotune_bc").into_owned(),
            Self::AutotunePresetSelectionSubmenu => rust_i18n::t!("tui_title_autotune_presets").into_owned(),
            Self::AutotuneStrategiesSubmenu => rust_i18n::t!("tui_title_autotune_strat").into_owned(),
            Self::AutotuneResultsSubmenu => rust_i18n::t!("tui_title_autotune_results").into_owned(),
        }
    }
}

/// Where the user has been, so Esc can walk back out the way they walked in.
///
/// This used to be knowledge spread across the back handler: one arm per
/// screen, each one naming the screen to return to. That is a table of parents
/// pretending to be navigation, and it goes stale the moment a screen gains a
/// second parent — the autotune menu reached three levels down used to send Esc
/// straight to the top, skipping two screens on the way.
///
/// Holding the stack instead means the answer is always the screen the user was
/// on when they opened this one, however deep they are.
#[derive(Default)]
pub struct History {
    screens: Vec<ActiveScreen>,
}

impl History {
    /// Record where a screen was opened from.
    pub fn push(&mut self, from: ActiveScreen) {
        // Consecutive duplicates would make Esc appear to do nothing, which is
        // what a screen that reopens itself looks like from the keyboard.
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
    /// Read off the stack rather than a table of parents, so it is the way the
    /// user actually came. `to` is appended rather than looked for: the stack
    /// holds where they came *from*, so the current screen is never in it.
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

/// The main menu.
///
/// `Interface` is Linux-only: nftables and iptables can bind the rules to one
/// output device and WinDivert cannot, so on Windows the row does not exist
/// rather than existing and doing nothing.
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
///
/// The main menu is ordered by how often a row is touched, least first: the
/// machine gets set up once, then the network, then the service, and the
/// strategy is what you come back to change. Headings are what make a list of
/// rows read as a set of decisions rather than a list to scan.
pub struct Group {
    /// The locale key of the heading.
    pub title: &'static str,
    pub rows: &'static [MainMenuState],
}

/// A menu's cursor is a position in an ordered list of states, so up and down
/// are the same arithmetic on every screen. Writing them out per menu is how one
/// of them ends up skipping a row, and the mouse needs the same list to turn a
/// drawn row back into a state.
fn step_in<T: Copy + PartialEq>(all: &[T], current: T, forward: bool) -> T {
    let len = all.len();
    if len == 0 {
        return current;
    }
    let pos = position_in(all, current);
    let next = if forward { pos + 1 } else { pos + len - 1 } % len;
    all[next]
}

fn position_in<T: Copy + PartialEq>(all: &[T], current: T) -> usize {
    all.iter().position(|s| *s == current).unwrap_or(0)
}

impl MainMenuState {
    /// The main menu as drawn, in order.
    ///
    /// The single source of truth: the menu is rendered from this list and the
    /// cursor is this list's index, so a row cannot be added to one and not the
    /// other. Headings are not in it — they are drawn from [`Self::GROUPS`] but
    /// never land the cursor.
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
        // The two ends of every screen, not a setting: no heading, because a
        // heading above "Run" would name a group of one decision.
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
    /// happens to be on this platform. Derived rather than named, so reordering
    /// the menu cannot leave the cursor starting on a row that is not there.
    pub fn first() -> Self {
        Self::all().next().unwrap_or(Self::Quit)
    }

    pub fn next(self) -> Self {
        Self::step(self, true)
    }

    pub fn prev(self) -> Self {
        Self::step(self, false)
    }

    fn step(self, forward: bool) -> Self {
        let all: Vec<Self> = Self::all().collect();
        step_in(&all, self, forward)
    }

    /// Where this row sits among the selectable ones, which is also the row the
    /// cursor lands on. Headings are not counted.
    pub fn index(self) -> usize {
        let all: Vec<Self> = Self::all().collect();
        position_in(&all, self)
    }
}

/// The settings under the Extended submenu, plus the way back.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ExtendedMenuState {
    Ttl,
    Fakes,
    Back,
}

impl ExtendedMenuState {
    pub const ALL: [Self; 3] = [Self::Ttl, Self::Fakes, Self::Back];

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }

    pub fn index(self) -> usize {
        position_in(&Self::ALL, self)
    }
}

/// The three things one can do about the fixed DPI-TTL, plus the way back.
///
/// This used to be a single row on the main menu where the arrows changed the
/// number and Enter ran the sweep, which made "do not touch it" and "set it to
/// seven" and "go and find one" three different gestures on one control. They
/// are three different decisions, so they get three rows.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TtlMenuState {
    /// Leave the setting alone: the DPI-desync feature stays off.
    DontTouch,
    /// Pin an explicit hop count.
    SetValue,
    /// Sweep the range and use the first hop count that works.
    Autopick,
    Back,
}

impl TtlMenuState {
    pub const ALL: [Self; 4] = [Self::DontTouch, Self::SetValue, Self::Autopick, Self::Back];

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }

    pub fn index(self) -> usize {
        position_in(&Self::ALL, self)
    }
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

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum GamefilterMenuState {
    Tcp,
    Udp,
    Back,
}

impl GamefilterMenuState {
    pub const ALL: [Self; 3] = [Self::Tcp, Self::Udp, Self::Back];

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }

    pub fn index(self) -> usize {
        position_in(&Self::ALL, self)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum FakesMenuState {
    DiscordUdp,
    GameUdp,
    Back,
}

impl FakesMenuState {
    pub const ALL: [Self; 3] = [Self::DiscordUdp, Self::GameUdp, Self::Back];

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }

    pub fn index(self) -> usize {
        position_in(&Self::ALL, self)
    }
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

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
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

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }

    /// Where this row sits, which is also the row the cursor lands on.
    pub fn index(self) -> usize {
        position_in(&Self::ALL, self)
    }
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

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
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

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }

    /// Where this row sits, which is also its index into
    /// [`BlockCheckType::all`](zapret_wrapper::autotune::BlockCheckType::all).
    pub fn index(self) -> usize {
        position_in(&Self::ALL, self)
    }

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

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }
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

    pub fn next(self) -> Self {
        step_in(&Self::ALL, self, true)
    }

    pub fn prev(self) -> Self {
        step_in(&Self::ALL, self, false)
    }

    pub fn index(self) -> usize {
        position_in(&Self::ALL, self)
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

    /// Put the user back on the main menu and forget how they got here.
    ///
    /// For a job that finished and dropped the user out of a submenu tree:
    /// there is nothing left to go back to.
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
            ActiveScreen::ExtendedSubmenu => {
                self.extended_menu = if forward {
                    self.extended_menu.next()
                } else {
                    self.extended_menu.prev()
                }
            }
            ActiveScreen::TtlSubmenu => {
                self.ttl_menu = if forward {
                    self.ttl_menu.next()
                } else {
                    self.ttl_menu.prev()
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
                self.autotune_menu = if forward {
                    self.autotune_menu.next()
                } else {
                    self.autotune_menu.prev()
                };
                self.autotune_menu_index = self.autotune_menu.index();
            }
            ActiveScreen::AutotuneProtocolsSubmenu => {
                self.autotune_protocols_menu = if forward {
                    self.autotune_protocols_menu.next()
                } else {
                    self.autotune_protocols_menu.prev()
                };
            }
            ActiveScreen::AutotunePresetSelectionSubmenu => {
                let total = zapret_wrapper::domains::PRESETS.len() + 1;
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
                // The report is a scrolling view, not a menu: the offset has no
                // cursor to sit on, and `views::report` pulls it back into range
                // once it knows how tall the tables turned out to be.
                self.scroll_report(forward, 1);
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
