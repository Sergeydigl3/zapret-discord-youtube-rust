//! List files shipped with the strategies repository, resolved relative to the
//! executable by [`crate::paths::exe_relative_lists_dir`].

use std::fs;
use std::path::{Path, PathBuf};

/// `ipset-all.txt` holding the sentinel that disables ipset entirely.
const NONE_MARKER: &str = "203.0.113.113/32";

/// Files a user edits by hand; they live in the downloaded repository rather
/// than in the cache because that is where the strategies read them from.
const USER_LIST_FILES: &[&str] = &[
    "list-general-user.txt",
    "list-exclude-user.txt",
    "ipset-exclude-user.txt",
];

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
        write!(f, "{s}")
    }
}

pub fn get_ipset_dir() -> PathBuf {
    crate::paths::exe_relative_lists_dir()
}

/// Create the three user list files the original shell scripts create empty
/// before every run, so a strategy that references them does not fail.
pub fn ensure_user_lists() {
    let lists_dir = crate::paths::repo_lists_dir();
    for name in USER_LIST_FILES {
        let path = lists_dir.join(name);
        if !path.exists() {
            let _ = fs::write(&path, "");
        }
    }
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
/// The two differ in what counts: inside `lists/` every file qualifies, in the
/// repository root only `*.txt` does.
fn is_lists_subdir(dir: &Path) -> bool {
    dir.file_name().is_some_and(|n| n == "lists")
        && dir
            .parent()
            .and_then(|p| p.file_name())
            .is_some_and(|n| n == crate::paths::REPO_DIR_NAME)
}

/// Collect every list file the editor may open.
pub fn get_lists_files() -> Vec<String> {
    let dir = get_ipset_dir();
    let txt_only = !is_lists_subdir(&dir);
    let mut files: Vec<String> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .filter(|p| !txt_only || p.extension().is_some_and(|e| e == "txt"))
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    files.sort();
    files
}

/// Inspect the `ipset-all.txt` file and report which mode it represents.
pub fn determine_current_mode() -> IpsetMode {
    let path = get_ipset_all_path();
    if !path.exists() {
        // A missing file is effectively an empty (Any) ipset.
        return IpsetMode::Any;
    }

    let raw = fs::read_to_string(&path).unwrap_or_default();
    let content = raw.trim();

    if content.is_empty() {
        return IpsetMode::Any;
    }
    if content == NONE_MARKER {
        return IpsetMode::None;
    }
    if fs::read_to_string(get_ipset_backup_path()).unwrap_or_default().trim() == content {
        return IpsetMode::Loaded;
    }

    IpsetMode::Custom
}

pub fn get_available_modes() -> Vec<IpsetMode> {
    if !crate::paths::strategies_installed() {
        return vec![IpsetMode::None];
    }

    let mut modes = vec![IpsetMode::None, IpsetMode::Any, IpsetMode::Loaded];
    if get_ipset_custom_path().exists() || determine_current_mode() == IpsetMode::Custom {
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

    // Leaving custom mode saves the current file as the custom one.
    if old_mode == IpsetMode::Custom && new_mode != IpsetMode::Custom && path.exists() {
        let _ = fs::copy(&path, get_ipset_custom_path());
    }

    match new_mode {
        IpsetMode::None => {
            let _ = fs::write(&path, format!("{NONE_MARKER}\n"));
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
