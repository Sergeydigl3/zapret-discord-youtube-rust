//! Domain list files.
//!
//! This is the shared layer that both the autotune feature and the TTL sweep
//! depend on. It used to be a cycle: `ttl` loaded its domains through
//! `autotune::load_domain_file` while `autotune` created the TTL file through
//! `ttl::ensure_ttl_file`. Owning the files here means neither feature has to
//! know about the other.

pub mod presets;
pub mod ttl;

use std::path::{Path, PathBuf};

pub use presets::{DomainPreset, PRESETS, PRESET_FILES};

pub const CUSTOM_DOMAINS_FILE: &str = "autotune_custom.txt";

/// Path of the domain list file backing the given preset.
pub fn preset_domains_file_path(preset_idx: usize) -> PathBuf {
    let name = PRESET_FILES.get(preset_idx).copied().unwrap_or(CUSTOM_DOMAINS_FILE);
    crate::paths::cache_dir().join(name)
}

/// Read a domain list file. One domain per line, lines starting with `#` are
/// treated as comments and skipped.
pub fn load_domain_file(path: &Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Domains for a preset come entirely from its text file. If the file is
/// missing or empty, the built-in defaults are used as a fallback.
pub fn get_domains_for_preset(preset_idx: usize) -> Vec<String> {
    if preset_idx >= PRESETS.len() {
        return Vec::new();
    }
    let file_domains = load_domain_file(&preset_domains_file_path(preset_idx));
    if !file_domains.is_empty() {
        return file_domains;
    }
    PRESETS[preset_idx].domains.iter().map(|s| s.to_string()).collect()
}

/// Create/refresh the per-preset domain list files (and the TTL list) with the
/// full built-in domain list so the user can add/remove domains freely.
pub fn ensure_domain_files() -> Result<(), String> {
    for (idx, preset) in PRESETS.iter().enumerate() {
        let path = preset_domains_file_path(idx);
        let header = rust_i18n::t!("domain_file_header_full").replace("{}", preset.name);
        ensure_domain_file(&path, &header, preset.domains)?;
    }
    ttl::ensure_ttl_file()
}

fn ensure_domain_file(path: &Path, header: &str, defaults: &[&str]) -> Result<(), String> {
    if path.exists() && !load_domain_file(path).is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create directory '{}': {}", parent.display(), e))?;
    }
    let mut content = format!("# {}\n", header);
    for d in defaults {
        content.push_str(d);
        content.push('\n');
    }
    std::fs::write(path, content).map_err(|e| format!("Cannot write '{}': {}", path.display(), e))
}
