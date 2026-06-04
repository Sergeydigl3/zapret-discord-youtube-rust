#[cfg(target_os = "windows")]
pub mod admin;

#[cfg(target_os = "windows")]
pub use admin::ensure_admin;
