//! Resolving a request into something the daemon can be started with.
//!
//! This is where the knowledge of zapret file formats stops and the plain
//! process lifecycle in `zapret_core::daemon` begins. A [`RunRequest`] names a
//! strategy and a few switches; a [`LaunchPlan`] is a binary, a working
//! directory, an argument vector and two port ranges, and nothing else.

use crate::strategy;
use crate::strategy::args::{build_args, game_filter};
use zapret_core::daemon::LaunchPlan;

/// What the user asked to run.
#[derive(Clone, Debug)]
pub struct RunRequest {
    /// Strategy file name as listed by `strategy::get_strategies`, e.g.
    /// `discord.bat`. Resolution happens in `plan`.
    pub strategy: String,
    /// Network interface, or `any`. Linux only: nftables and iptables can bind
    /// rules to one output device and WinDivert cannot, so elsewhere there is
    /// nothing here to ask for.
    #[cfg(target_os = "linux")]
    pub interface: String,
    pub gamefilter_tcp: bool,
    pub gamefilter_udp: bool,
    /// A fixed DPI TTL for this run. `None` takes whatever the configuration
    /// file says, which is how the ordinary run and the sweeps differ: the TTL
    /// sweep passes `Some(n)`, everything else defers to the saved value.
    pub ttl_override: Option<u8>,
    /// The TTL read from an explicit `--config` file. Outer `None` means "no
    /// file was given, ask the default config"; inner `None` means the file
    /// itself asks for automatic TTL and must not fall back to the default.
    pub ttl_from_config: Option<Option<u8>>,
    /// Router mode from an explicit `--config` file, when one was given.
    #[cfg(target_os = "linux")]
    pub router: Option<bool>,
}

impl RunRequest {
    /// A request with no fixed TTL: the saved value is used.
    pub fn new(strategy: &str, gamefilter_tcp: bool, gamefilter_udp: bool) -> Self {
        Self {
            strategy: strategy.to_string(),
            #[cfg(target_os = "linux")]
            interface: "any".to_string(),
            gamefilter_tcp,
            gamefilter_udp,
            ttl_override: None,
            ttl_from_config: None,
            #[cfg(target_os = "linux")]
            router: None,
        }
    }

    /// The same request, bound to one network interface.
    #[cfg(target_os = "linux")]
    pub fn with_interface(mut self, interface: &str) -> Self {
        self.interface = interface.to_string();
        self
    }

    /// The same request, pinned to one DPI TTL.
    pub fn with_ttl(mut self, ttl: u8) -> Self {
        self.ttl_override = Some(ttl);
        self
    }

    /// Use the TTL exactly as the given configuration file states it.
    ///
    /// `None` means automatic: unlike leaving the field unset, it must not
    /// fall back to whatever the default `conf.env` happens to contain.
    pub fn with_config_ttl(mut self, ttl: Option<u8>) -> Self {
        self.ttl_from_config = Some(ttl);
        self
    }

    /// Use the router mode stated by the given configuration file.
    #[cfg(target_os = "linux")]
    pub fn with_router(mut self, router: bool) -> Self {
        self.router = Some(router);
        self
    }
}

/// Pick the TTL for a run: a sweep's explicit value first, then an explicit
/// `--config` file (including its "auto" answer), then the saved default.
fn resolve_ttl(
    ttl_override: Option<u8>,
    ttl_from_config: Option<Option<u8>>,
    saved: impl FnOnce() -> Option<u8>,
) -> Option<u8> {
    match ttl_override {
        Some(ttl) => Some(ttl),
        None => ttl_from_config.unwrap_or_else(saved),
    }
}

/// Read the strategy, expand it and produce the launch plan.
///
/// Fails when the strategy cannot be read or does not contain exactly one
/// `--wf-tcp` and one `--wf-udp` block, which is what the firewall backend
/// needs and what the parser insists on.
pub fn plan(req: &RunRequest) -> Result<LaunchPlan, String> {
    let path = strategy::resolve(&req.strategy);
    let parsed = strategy::parse_bat_file(
        path.to_str().ok_or("invalid strategy path")?,
        game_filter(req.gamefilter_tcp, req.gamefilter_udp).as_ref(),
    )?;

    let ttl = resolve_ttl(req.ttl_override, req.ttl_from_config, crate::config::load_ttl);

    Ok(LaunchPlan {
        binary: crate::paths::binary_path(),
        work_dir: crate::paths::repo_dir(),
        args: build_args(&parsed, ttl),
        tcp_ports: parsed.tcp_ports,
        udp_ports: parsed.udp_ports,
    })
}

#[cfg(test)]
mod tests {
    use super::resolve_ttl;

    #[test]
    fn a_sweep_value_beats_both_files() {
        assert_eq!(resolve_ttl(Some(4), Some(Some(8)), || Some(9)), Some(4));
    }

    #[test]
    fn an_explicit_config_beats_the_default_config() {
        assert_eq!(resolve_ttl(None, Some(Some(8)), || Some(9)), Some(8));
    }

    #[test]
    fn an_explicit_auto_in_the_config_does_not_fall_back_to_the_default() {
        assert_eq!(resolve_ttl(None, Some(None), || Some(9)), None);
    }

    #[test]
    fn without_a_config_file_the_saved_value_is_used() {
        assert_eq!(resolve_ttl(None, None, || Some(9)), Some(9));
    }
}
