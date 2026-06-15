use std::fs;
use std::path::{Path, PathBuf};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum IpsetMode {
    None,
    Any,
    Loaded,
    Custom,
}

impl IpsetMode {
    pub fn to_string(&self) -> String {
        match self {
            IpsetMode::None => rust_i18n::t!("ipset_none").into_owned(),
            IpsetMode::Any => rust_i18n::t!("ipset_any").into_owned(),
            IpsetMode::Loaded => rust_i18n::t!("ipset_loaded").into_owned(),
            IpsetMode::Custom => rust_i18n::t!("ipset_custom").into_owned(),
        }
    }
}

pub fn get_ipset_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .map(|p| p.parent().unwrap().to_path_buf())
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
        
    let base_dir = exe_dir.join("zapret-discord-youtube-linux");
    let lists_dir = base_dir.join("lists");

    if lists_dir.exists() && lists_dir.is_dir() {
        lists_dir
    } else if base_dir.exists() && base_dir.is_dir() {
        base_dir
    } else {
        // Fallback to current directory dev mode
        let local_base = Path::new("zapret-discord-youtube-linux");
        let local_lists = local_base.join("lists");
        if local_lists.exists() && local_lists.is_dir() {
            local_lists
        } else {
            local_base.to_path_buf()
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
    if old_mode == IpsetMode::Custom && new_mode != IpsetMode::Custom {
        if path.exists() {
            let custom_path = get_ipset_custom_path();
            let _ = fs::copy(&path, &custom_path);
        }
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
