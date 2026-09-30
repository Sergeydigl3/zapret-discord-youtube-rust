//! The auto-tuning feature: pick a strategy and DPI parameters that work on
//! this network.
//!
//! Built entirely on `crate::net` for measurements and `crate::domains` for the
//! domain lists, so the feature layer contains no probing or file handling of
//! its own.

pub mod orchestrator;
pub mod storage;
pub mod types;

pub use orchestrator::{domain_check_error, run_all};
pub use storage::{load_results_file, save_results_file, RESULTS_FILE};
pub use types::{
    status_char, status_str_file, AutotuneConfig, AutotuneResults, BlockCheckType, BlockChecks, CheckResult,
    CheckStatus, DomainCheckResult, DomainProtocolCheck, PresetResult, StrategyCheckResult,
};
