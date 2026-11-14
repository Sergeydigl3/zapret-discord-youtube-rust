//! The operations a command line can ask for.
//!
//! Settings are not actions: they are applied first and then either an action
//! runs or the program exits.

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
    pub(super) fn label(self) -> &'static str {
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
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Add => "--defender-add",
            Self::Remove => "--defender-remove",
            Self::Status => "--defender-status",
        }
    }
}

/// One non-interactive operation.
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

    pub(super) fn label(&self) -> &'static str {
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
