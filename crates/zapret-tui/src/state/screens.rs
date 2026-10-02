//! The screen set: which screen is on, where it was opened from, and what it is
//! called.
//!
//! Everything else a screen needs to remember — where its cursor is, which row
//! it is on — belongs to the component in [`crate::screens`]. What is left here
//! is the two invariants that hold *across* the set: where a screen was opened
//! from is [`History`], not a second copy in the back handler, and every screen
//! has a name in exactly one place.

use super::AppState;

/// The screens a submenu can be, and the main menu itself.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
    AutotuneNumRequests,
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
            Self::AutotuneNumRequests => "menu_autotune_requests",
            Self::AutotuneEditDomainsSubmenu => "tui_title_autotune_edit_domains",
            Self::AutotuneProtocolsSubmenu => "tui_title_autotune_proto",
            Self::AutotuneBlockChecksSubmenu => "tui_title_autotune_bc",
            Self::AutotunePresetSelectionSubmenu => "tui_title_autotune_presets",
            Self::AutotuneStrategiesSubmenu => "tui_title_autotune_strat",
            Self::AutotuneResultsSubmenu => "tui_title_autotune_results",
        };
        rust_i18n::t!(key).into_owned()
    }

    /// Whether the screen shows what is installed under the menu, because on
    /// those screens it changes what the rows mean.
    pub fn shows_status_line(self) -> bool {
        matches!(
            self,
            Self::Main
                | Self::ServiceSubmenu
                | Self::ExtendedSubmenu
                | Self::DownloadDepsSubmenu
                | Self::DownloadZapretSubmenu
                | Self::DownloadStrategiesSubmenu
                | Self::ZapretTagSelect
                | Self::StrategyTagSelect
        )
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

/// Which version of the autotune report is open.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
        Self::ALL[(Self::ALL.iter().position(|t| *t == self).unwrap_or(0) + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let len = Self::ALL.len();
        Self::ALL[(Self::ALL.iter().position(|t| *t == self).unwrap_or(0) + len - 1) % len]
    }
}

/// Which downloader a version picker is for. The two flows are the same four
/// rows and deliberately do not share their version mapping.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DownloadTarget {
    Zapret,
    Strategies,
}

/// Which fake payload a `.bin` list is being chosen for.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FakesSelectTarget {
    DiscordUdp,
    GameUdp,
}

/// Which tag list a tag picker is showing.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TagTarget {
    Zapret,
    Strategies,
}

/// The version a downloader should fetch.
#[derive(PartialEq, Eq, Clone, Debug)]
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

    /// Go one step up. `None` at the main menu, which is where the back stack
    /// runs out and Esc means "leave".
    pub fn back(&mut self) -> Option<ActiveScreen> {
        let previous = self.history.pop()?;
        self.active_screen = previous;
        self.status_message = None;
        Some(previous)
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
}
