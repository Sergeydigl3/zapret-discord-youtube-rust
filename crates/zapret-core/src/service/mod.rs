//! Service management: one `ServiceManager` per init system, plus the Windows
//! SCM client.
//!
//! The managers are deliberately dumb - they write the unit file and shell out
//! to the init system's own tooling. `detect` decides which one applies,
//! `script` holds the command line they all share.

mod detect;
#[cfg(target_os = "linux")]
mod script;

// Linux
#[cfg(target_os = "linux")]
pub mod dinit;
#[cfg(target_os = "linux")]
pub mod openrc;
#[cfg(target_os = "linux")]
pub mod runit;
#[cfg(target_os = "linux")]
pub mod s6;
#[cfg(target_os = "linux")]
pub mod systemd;
#[cfg(target_os = "linux")]
pub mod sysv;

#[cfg(target_os = "linux")]
pub use dinit::DinitManager;
#[cfg(target_os = "linux")]
pub use openrc::OpenRcManager;
#[cfg(target_os = "linux")]
pub use runit::RunitManager;
#[cfg(target_os = "linux")]
pub use s6::S6Manager;
#[cfg(target_os = "linux")]
pub use systemd::SystemdManager;
#[cfg(target_os = "linux")]
pub use sysv::InitManager;

// Windows
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "windows")]
pub use windows::WindowsServiceManager;

pub use detect::{detect_init_system, InitType};

use std::path::Path;

pub trait ServiceManager: Send + Sync {
    fn is_installed(&self) -> bool;
    fn is_active(&self) -> bool;
    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String>;
    fn uninstall(&self) -> Result<(), String>;
    fn start(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn restart(&self) -> Result<(), String>;
}

/// Factory function to get the ServiceManager for the detected init system.
pub fn get_detected_manager() -> Option<Box<dyn ServiceManager>> {
    detect_init_system().map(get_manager)
}

/// Factory function to get the ServiceManager for a specific InitType.
pub fn get_manager(init_type: InitType) -> Box<dyn ServiceManager> {
    match init_type {
        #[cfg(target_os = "linux")]
        InitType::Systemd => Box::new(SystemdManager),
        #[cfg(target_os = "linux")]
        InitType::OpenRc => Box::new(OpenRcManager),
        #[cfg(target_os = "linux")]
        InitType::Runit => Box::new(RunitManager),
        #[cfg(target_os = "linux")]
        InitType::Dinit => Box::new(DinitManager),
        #[cfg(target_os = "linux")]
        InitType::S6 => Box::new(S6Manager),
        #[cfg(target_os = "linux")]
        InitType::Init => Box::new(InitManager),

        #[cfg(target_os = "windows")]
        InitType::Windows => Box::new(WindowsServiceManager),
    }
}
