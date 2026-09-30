//! List files shipped with the strategies repository.
//!
//! The directory is resolved by [`crate::paths::exe_relative_lists_dir`], which
//! deliberately ignores `ZAPRET_CACHE_DIR`. See the note there before "fixing"
//! that: it is the current behaviour, kept as is by the restructuring.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum IpsetMode {
    None,
    Any,
    Loaded,
    Custom,
}

impl std::fmt::Display for IpsetMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            IpsetMode::None => rust_i18n::t!("ipset_none").into_owned(),
            IpsetMode::Any => rust_i18n::t!("ipset_any").into_owned(),
            IpsetMode::Loaded => rust_i18n::t!("ipset_loaded").into_owned(),
            IpsetMode::Custom => rust_i18n::t!("ipset_custom").into_owned(),
        };
        write!(f, "{}", s)
    }
}

pub fn get_ipset_dir() -> PathBuf {
    crate::paths::exe_relative_lists_dir()
}

pub fn get_ipset_all_path() -> PathBuf {
    get_ipset_dir().join("ipset-all.txt")
}

pub fn get_ipset_backup_path() -> PathBuf {
    get_ipset_dir().join("ipset-all.txt.backup")
}

pub fn get_ipset_custom_path() -> PathBuf {
    get_ipset_dir().join("ipset-all.txt.custom")
}

/// True when `dir` is the `lists/` subdirectory rather than the repository root.
///
/// [`crate::paths::exe_relative_lists_dir`] returns one of the two, and the
/// listing rules differ: inside `lists/` every file counts, in the repository
/// root only `*.txt` does.
fn is_lists_subdir(dir: &Path) -> bool {
    dir.file_name().is_some_and(|n| n == "lists")
        && dir
            .parent()
            .and_then(|p| p.file_name())
            .is_some_and(|n| n == crate::paths::REPO_DIR_NAME)
}

/// Collect every list file the editor may open.
pub fn get_lists_files() -> Vec<String> {
    let mut files = Vec::new();

    let dir = get_ipset_dir();
    if dir.exists() && dir.is_dir() {
        let txt_only = !is_lists_subdir(&dir);
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                if txt_only && path.extension().is_none_or(|e| e != "txt") {
                    continue;
                }
                files.push(path.to_string_lossy().into_owned());
            }
        }
    }

    files.sort();
    files
}

/// Inspect the `ipset-all.txt` file and report which mode it represents.
pub fn determine_current_mode() -> IpsetMode {
    let path = get_ipset_all_path();
    if !path.exists() {
        // If file doesn't exist, we can treat it as Any (empty) or Custom.
        // Let's treat it as Any since it's effectively empty.
        return IpsetMode::Any;
    }

    let content = fs::read_to_string(&path).unwrap_or_default().trim().to_string();

    if content == "203.0.113.113/32" {
        return IpsetMode::None;
    }

    if content.is_empty() {
        return IpsetMode::Any;
    }

    let backup_path = get_ipset_backup_path();
    if backup_path.exists() {
        let backup_content = fs::read_to_string(&backup_path).unwrap_or_default().trim().to_string();
        if content == backup_content {
            return IpsetMode::Loaded;
        }
    }

    IpsetMode::Custom
}

pub fn get_available_modes() -> Vec<IpsetMode> {
    if !crate::paths::strategies_installed() {
        return vec![IpsetMode::None];
    }

    let mut modes = vec![IpsetMode::None, IpsetMode::Any, IpsetMode::Loaded];
    let custom_path = get_ipset_custom_path();

    if custom_path.exists() || determine_current_mode() == IpsetMode::Custom {
        modes.push(IpsetMode::Custom);
    }

    modes
}

pub fn apply_ipset_mode(old_mode: IpsetMode, new_mode: IpsetMode) {
    let path = get_ipset_all_path();
    let dir = get_ipset_dir();

    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }

    // Save custom mode if leaving it
    if old_mode == IpsetMode::Custom && new_mode != IpsetMode::Custom && path.exists() {
        let custom_path = get_ipset_custom_path();
        let _ = fs::copy(&path, &custom_path);
    }

    match new_mode {
        IpsetMode::None => {
            let _ = fs::write(&path, "203.0.113.113/32\n");
        }
        IpsetMode::Any => {
            let _ = fs::write(&path, "");
        }
        IpsetMode::Loaded => {
            let backup_path = get_ipset_backup_path();
            if backup_path.exists() {
                let _ = fs::copy(&backup_path, &path);
            } else {
                let _ = fs::write(&path, "");
            }
        }
        IpsetMode::Custom => {
            let custom_path = get_ipset_custom_path();
            if custom_path.exists() {
                let _ = fs::copy(&custom_path, &path);
            }
        }
    }
}
