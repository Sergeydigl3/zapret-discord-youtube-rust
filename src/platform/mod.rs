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
