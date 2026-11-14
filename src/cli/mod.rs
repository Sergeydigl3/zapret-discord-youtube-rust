//! Command-line surface of zapret-rust.
//!
//! The program is a TUI by default. Every menu action has a flag equivalent
//! here, so the same things can be configured and run from a script without a
//! terminal: run, downloads, interface/strategy/gamefilter/backend/ipset/TTL/
//! fakes/router, autotune, the service, Defender exclusions and the list files.
//!
//! The flags themselves live in [`args`], grouped by the part of the program
//! they belong to. This module only answers the questions the rest of the
//! binary asks: which one action was requested, does it need root, and is this
//! a TUI run or not.

mod action;
mod args;
mod help;

pub use args::Cli;

pub use action::{Action, DefenderAction, ServiceAction};
pub use help::show_help;

impl Cli {
    /// True when at least one flag asks for something other than the TUI.
    pub fn has_non_tui_request(&self) -> bool {
        if self.run.no_tui || self.run.run || self.has_settings() {
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
        self.settings.has_any()
    }

    /// The operation requested by the flags, or `None` for the normal TUI.
    ///
    /// `--no-tui` is a mode rather than an action: with an explicit action it
    /// just says "do not fall back to the TUI", and on its own it means `--run`.
    pub fn action(&self) -> Result<Option<Action>, String> {
        let mut actions: Vec<Action> = Vec::new();

        if self.run.run {
            actions.push(Action::Run);
        }
        if let Some(version) = &self.downloads.download_zapret {
            actions.push(Action::DownloadZapret(version.clone()));
        }
        if let Some(version) = &self.downloads.download_strategies {
            actions.push(Action::DownloadStrategies(version.clone()));
        }
        if self.downloads.download_defaults {
            actions.push(Action::DownloadDefaults);
        }
        #[cfg(target_os = "linux")]
        if self.inspect.list_interfaces {
            actions.push(Action::ListInterfaces);
        }
        if self.inspect.list_strategies {
            actions.push(Action::ListStrategies);
        }
        if self.inspect.list_fakes {
            actions.push(Action::ListFakes);
        }
        if self.inspect.list_lists {
            actions.push(Action::ListLists);
        }
        if self.inspect.list_domains {
            actions.push(Action::ListDomains);
        }
        if self.inspect.list_tags_zapret {
            actions.push(Action::ListTagsZapret);
        }
        if self.inspect.list_tags_strategies {
            actions.push(Action::ListTagsStrategies);
        }
        if self.inspect.show_config {
            actions.push(Action::ShowConfig);
        }
        if self.inspect.status {
            actions.push(Action::Status);
        }
        if self.inspect.autotune_results {
            actions.push(Action::ShowAutotuneResults);
        }
        if let Some(file) = &self.edit.edit_list {
            actions.push(Action::EditList(file.clone()));
        }
        if let Some(preset) = &self.edit.edit_domains {
            actions.push(Action::EditDomains(preset.clone()));
        }
        if self.autotune.autotune {
            actions.push(Action::Autotune);
        }
        if self.autotune.ttl_autopick {
            actions.push(Action::TtlAutopick);
        }
        if self.service.service_install {
            actions.push(Action::Service(ServiceAction::Install));
        }
        if self.service.service_start {
            actions.push(Action::Service(ServiceAction::Start));
        }
        if self.service.service_stop {
            actions.push(Action::Service(ServiceAction::Stop));
        }
        if self.service.service_restart {
            actions.push(Action::Service(ServiceAction::Restart));
        }
        if self.service.service_uninstall {
            actions.push(Action::Service(ServiceAction::Uninstall));
        }
        if self.service.service_status {
            actions.push(Action::Service(ServiceAction::Status));
        }
        if self.defender.defender_add {
            actions.push(Action::Defender(DefenderAction::Add));
        }
        if self.defender.defender_remove {
            actions.push(Action::Defender(DefenderAction::Remove));
        }
        if self.defender.defender_status {
            actions.push(Action::Defender(DefenderAction::Status));
        }

        if actions.len() > 1 {
            let names: Vec<&str> = actions.iter().map(Action::label).collect();
            return Err(format!("{} {}", rust_i18n::t!("cli_err_conflict"), names.join(", ")));
        }

        if self.autotune.has_options() && !self.autotune.autotune {
            return Err(rust_i18n::t!("cli_err_autotune_opts").into_owned());
        }

        // Run parameters are only useful for a launch. Supplying one means the
        // user asked for that launch, not for the menu.
        #[cfg(target_os = "linux")]
        let has_run_options = self.run.config.is_some()
            || self.run.strategy.is_some()
            || self.run.interface.is_some()
            || self.run.gamefiltertcp
            || self.run.gamefilterudp;
        #[cfg(not(target_os = "linux"))]
        let has_run_options = self.run.config.is_some()
            || self.run.strategy.is_some()
            || self.run.gamefiltertcp
            || self.run.gamefilterudp;

        Ok(actions
            .pop()
            .or_else(|| (self.run.no_tui || has_run_options).then_some(Action::Run)))
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
        if self.has_settings() || self.run.no_tui || self.run.run {
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
pub(super) fn parse_toggle(value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "enable" | "enabled" => Ok(true),
        "0" | "false" | "no" | "off" | "disable" | "disabled" => Ok(false),
        other => Err(format!("expected on/off, got '{other}'")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use action::Action;
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
        assert_eq!(cli.settings.set_gamefilter_tcp, Some(true));

        let cli = parse(&["zapret-rust", "--set-gamefilter-tcp", "off"]);
        assert_eq!(cli.settings.set_gamefilter_tcp, Some(false));

        let cli = parse(&["zapret-rust", "--set-gamefilter-udp=on"]);
        assert_eq!(cli.settings.set_gamefilter_udp, Some(true));
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
}
