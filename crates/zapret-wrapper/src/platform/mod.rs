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

/// True when a zapret daemon that this program did not start is running.
///
/// The only question in the program that is answered by looking at the machine:
/// the daemon this program started is a child handle, and a process found by
/// name belongs to somebody else. `run::queue_in_use` is the one caller-facing
/// form of the question.
pub fn is_nfqws_running() -> bool {
    zapret_core::process::is_daemon_running()
}

/// The network interfaces a firewall rule can be bound to.
///
/// Only Linux has this choice: nftables and iptables match on the output
/// device, while WinDivert filters the whole system. Everywhere else the list
/// is empty rather than a single meaningless `"any"` entry, so nothing
/// downstream can offer a setting that would do nothing.
#[cfg(target_os = "linux")]
pub fn get_interfaces() -> Vec<String> {
    let mut interfaces = vec!["any".to_string()];
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                interfaces.push(name);
            }
        }
    }
    interfaces
}

#[cfg(not(target_os = "linux"))]
pub fn get_interfaces() -> Vec<String> {
    Vec::new()
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
