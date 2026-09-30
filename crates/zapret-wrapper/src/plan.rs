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
    /// Network interface, or `any`.
    pub interface: String,
    pub gamefilter_tcp: bool,
    pub gamefilter_udp: bool,
    /// A fixed DPI TTL for this run. `None` takes whatever the configuration
    /// file says, which is how the ordinary run and the sweeps differ: the TTL
    /// sweep passes `Some(n)`, everything else defers to the saved value.
    pub ttl_override: Option<u8>,
}

impl RunRequest {
    /// A request with no fixed TTL: the saved value is used.
    pub fn new(strategy: &str, interface: &str, gamefilter_tcp: bool, gamefilter_udp: bool) -> Self {
        Self {
            strategy: strategy.to_string(),
            interface: interface.to_string(),
            gamefilter_tcp,
            gamefilter_udp,
            ttl_override: None,
        }
    }

    /// The same request, pinned to one DPI TTL.
    pub fn with_ttl(mut self, ttl: u8) -> Self {
        self.ttl_override = Some(ttl);
        self
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

    let ttl = req.ttl_override.or_else(crate::config::load_ttl);

    Ok(LaunchPlan {
        binary: crate::paths::binary_path(),
        work_dir: crate::paths::repo_dir(),
        args: build_args(&parsed, ttl),
        tcp_ports: parsed.tcp_ports,
        udp_ports: parsed.udp_ports,
    })
}
