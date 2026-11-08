use std::fs;

/// Collect all available strategy file names.
///
/// Bundled custom strategies are ensured to exist first, then `.txt` files are
/// looked up in two locations:
/// - `<cache>/custom-strategies/*.txt` (user/bundled custom strategies)
/// - `<repo>/custom-strategies/*.txt`
///
/// The strategy repository root is no longer scanned: it is the lists provider
/// now, and the strategy files it ships describe a foreign engine. The list is
/// sorted and deduplicated. A fallback of `["general.txt"]` is returned when no
/// files are found at all.
pub fn get_strategies() -> Vec<String> {
    let _ = crate::strategy::assets::ensure_custom_strategies();
    let mut strats = Vec::new();

    for dir in [
        crate::paths::custom_strategies_dir(),
        crate::paths::repo_dir().join("custom-strategies"),
    ] {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.ends_with(".txt") {
                        strats.push(name);
                    }
                }
            }
        }
    }

    strats.sort();
    strats.dedup();

    if strats.is_empty() {
        strats.push("general.txt".to_string());
    }

    strats
}
