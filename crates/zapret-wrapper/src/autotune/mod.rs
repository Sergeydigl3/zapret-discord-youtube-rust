//! The auto-tuning feature: find a strategy and DPI parameters that work on this
//! network.
//!
//! Everything the sweep needs lives in this zone, because nothing outside the
//! sweep uses any of it. The pieces, in the order the sweep runs them:
//!
//! ```text
//!   types             what a check result looks like
//!   cancel            abort flag, and how the running probes are stopped
//!   dns / quic        raw reachability: resolving, TCP, HTTP/3
//!   probe             raw reachability: the curl transports (HTTP, TLS, QUIC)
//!   checks_network    what the network is blocking right now (DNS, RST, SNI, ...)
//!   checks_domain     per-domain reachability through and without the tunnel
//!   progress          the event stream the sweep reports through
//!   orchestrator      the sweep itself: strategy x domain x protocol
//!   storage           the results file
//! ```
//!
//! The probes deliberately know nothing about strategies and the orchestrator
//! knows nothing about sockets, but they are one feature, not two: the probes
//! report in this feature's result types and exist to serve this sweep.
//!
//! Domain *list files* are the one thing that does not live here - they are
//! shared with the TTL sweep, so they are in [`crate::domains`].

pub mod cancel;
mod checks_domain;
mod checks_network;
mod dns;
mod probe;
mod quic;

pub mod orchestrator;
pub mod progress;
pub mod storage;
pub mod types;

pub use checks_domain::domain_check_error;
pub use orchestrator::run_all;
pub use progress::{LogLevel, SweepEvent};
pub use storage::{load_results_file, save_results_file, RESULTS_FILE};
pub use types::{
    status_char, status_str_file, AutotuneConfig, AutotuneResults, BlockCheckType, BlockChecks, CheckResult,
    CheckStatus, DomainCheckResult, DomainProtocolCheck, PresetResult, StrategyCheckResult,
};
