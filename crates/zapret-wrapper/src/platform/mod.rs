/// Talking to the operating system: privilege escalation, process detection,
/// interface enumeration, and the platform-specific spellings of the paths that
/// a child process needs.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::ensure_admin;

#[cfg(target_os = "linux")]
pub use linux::ensure_admin;

// Заглушка на остальные системы (BSD, MacOS)
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn ensure_admin() {}

#[cfg(target_os = "windows")]
pub use windows::is_nfqws_running;

#[cfg(target_os = "linux")]
pub use linux::is_nfqws_running;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn is_nfqws_running() -> bool {
    false
}

/// Return available network interfaces.
/// On Windows and macOS there is no `/sys/class/net`, so only "any" is returned.
pub fn get_interfaces() -> Vec<String> {
    #[allow(unused_mut)]
    let mut interfaces = vec!["any".to_string()];

    #[cfg(target_os = "linux")]
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                interfaces.push(name);
            }
        }
    }

    interfaces
}

/// Device that swallows output, spelled per platform.
///
/// The single owner of this name: the network probes redirect a child's stdout
/// into it, and the name differs on Windows.
pub fn null_device() -> &'static str {
    if cfg!(target_os = "windows") {
        "NUL"
    } else {
        "/dev/null"
    }
}
