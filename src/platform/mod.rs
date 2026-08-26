#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub use windows::ensure_admin;

#[cfg(target_os = "linux")]
pub use linux::ensure_admin;

// Заглушка на остальные системы (BSD, MacOS)
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn ensure_admin() {}

#[cfg(target_os = "windows")]
pub use windows::is_nfqws_running;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn is_nfqws_running() -> bool {
    false
}

#[cfg(target_os = "linux")]
pub use linux::{disable_ip_forward, enable_ip_forward, is_nfqws_running};

#[cfg(not(target_os = "linux"))]
pub fn enable_ip_forward() {}

#[cfg(not(target_os = "linux"))]
pub fn disable_ip_forward() {}
