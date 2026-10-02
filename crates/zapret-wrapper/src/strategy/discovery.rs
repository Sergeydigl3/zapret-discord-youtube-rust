use std::fs;
use std::path::Path;

/// Offered when neither the cache nor the repository holds a single strategy.
const FALLBACK_STRATEGY: &str = "discord.bat";

/// Names of the `*.bat` files directly inside `dir`, ignoring unreadable entries.
fn bat_names(dir: &Path) -> Vec<String> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| n.ends_with(".bat"))
        .collect()
}

/// Collect all available strategy file names: the custom-strategies directory
/// next to the executable, the one inside the repository, and the `general*` /
/// `discord*` files in the repository root. Sorted and deduplicated.
pub fn get_strategies() -> Vec<String> {
    let _ = crate::strategy::assets::ensure_custom_strategies();
    let repo = crate::paths::repo_dir();

    let mut strats = bat_names(&crate::paths::custom_strategies_dir());
    strats.extend(bat_names(&repo.join("custom-strategies")));
    strats.extend(
        bat_names(&repo)
            .into_iter()
            .filter(|n| n.starts_with("general") || n.starts_with("discord")),
    );

    strats.sort();
    strats.dedup();

    if strats.is_empty() {
        strats.push(FALLBACK_STRATEGY.to_string());
    }

    strats
}
