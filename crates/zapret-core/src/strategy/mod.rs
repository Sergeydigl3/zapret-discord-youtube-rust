pub mod args;
pub mod assets;
pub mod discovery;
pub mod parser;

use std::path::PathBuf;

pub use assets::ensure_custom_strategies;
pub use discovery::get_strategies;
#[allow(unused_imports)]
pub use parser::ParsedStrategy;
pub use parser::{parse_bat_file, GameFilterPorts};

/// Resolve a strategy file name to the file that should actually be parsed.
///
/// A custom strategy in the cache directory wins over one shipped in the
/// repository, so user overrides survive a repository re-download. This is the
/// single place that precedence is decided: discovery lists the names, the
/// sweeps list the names, and both end up here.
pub fn resolve(strategy_file: &str) -> PathBuf {
    let repo_dir = crate::paths::repo_dir();

    let cache_custom = crate::paths::custom_strategies_dir().join(strategy_file);
    if cache_custom.exists() {
        return cache_custom;
    }
    let repo_custom = repo_dir.join("custom-strategies").join(strategy_file);
    if repo_custom.exists() {
        repo_custom
    } else {
        repo_dir.join(strategy_file)
    }
}
