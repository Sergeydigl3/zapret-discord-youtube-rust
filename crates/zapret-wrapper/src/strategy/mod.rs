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

/// Accept the file stem as well as the full name.
///
/// The TUI always passes a name from [`discovery::get_strategies`] (for example
/// `discord.bat`), while a script writer naturally types `--strategy discord`.
/// The returned name is the one that [`resolve`] finds, or the input unchanged
/// when neither spelling exists (so the caller can report the original).
pub fn normalize_name(strategy_file: &str) -> String {
    if resolve(strategy_file).is_file() {
        return strategy_file.to_string();
    }
    if !strategy_file.ends_with(".bat") {
        let with_bat = format!("{}.bat", strategy_file);
        if resolve(&with_bat).is_file() {
            return with_bat;
        }
    }
    strategy_file.to_string()
}
