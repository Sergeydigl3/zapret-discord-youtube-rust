use std::env;
use std::fs;
use std::path::Path;

/// Directory that contains the strategies repo (bat/sh scripts + lists).
fn repo_dir() -> String {
    env::var("REPO_DIR").unwrap_or_else(|_| {
        crate::config::get_cache_dir()
            .join("zapret-discord-youtube-linux")
            .to_string_lossy()
            .into_owned()
    })
}

/// Collect all available strategy file names from the strategies repository.
///
/// Files are looked up in two locations:
/// - `<repo>/custom-strategies/*.bat`
/// - `<repo>/*.bat` (only filenames starting with `general` or `discord`)
///
/// The list is sorted and deduplicated. A fallback of `["discord.bat"]` is
/// returned when no files are found at all.
pub fn get_strategies() -> Vec<String> {
    let repo = repo_dir();
    let mut strats = Vec::new();

    // custom-strategies subfolder
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
                if name.ends_with(".bat")
                    && (name.starts_with("general") || name.starts_with("discord"))
                {
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
