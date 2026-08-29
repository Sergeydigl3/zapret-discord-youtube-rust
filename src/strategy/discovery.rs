use std::fs;
use std::path::Path;

/// Directory that contains the strategies repo (bat/sh scripts + lists).
fn repo_dir() -> std::path::PathBuf {
    crate::strategy::embedded::repo_dir()
}

/// Collect all available strategy file names from the strategies repository.
///
/// Bundled custom strategies are ensured to exist first, then files are looked
/// up in three locations:
/// - `<cache>/custom-strategies/*.bat` (user/bundled custom strategies)
/// - `<repo>/custom-strategies/*.bat`
/// - `<repo>/*.bat` (only filenames starting with `general` or `discord`)
///
/// The list is sorted and deduplicated. A fallback of `["discord.bat"]` is
/// returned when no files are found at all.
pub fn get_strategies() -> Vec<String> {
    let _ = crate::strategy::embedded::ensure_custom_strategies();
    let repo = repo_dir();
    let mut strats = Vec::new();

    // custom-strategies folder next to the executable
    if let Ok(entries) = fs::read_dir(crate::config::get_cache_dir().join("custom-strategies")) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".bat") {
                    strats.push(name);
                }
            }
        }
    }

    // custom-strategies subfolder inside the repo
    if let Ok(entries) = fs::read_dir(Path::new(&repo).join("custom-strategies")) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".bat") {
                    strats.push(name);
                }
            }
        }
    }

    // root of the repo
    if let Ok(entries) = fs::read_dir(&repo) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                if name.ends_with(".bat") && (name.starts_with("general") || name.starts_with("discord")) {
                    strats.push(name);
                }
            }
        }
    }

    strats.sort();
    strats.dedup();

    if strats.is_empty() {
        strats.push("discord.bat".to_string());
    }

    strats
}
