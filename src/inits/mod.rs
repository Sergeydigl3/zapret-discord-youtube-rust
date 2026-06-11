// Linux
#[cfg(target_os = "linux")]
pub mod init;
#[cfg(target_os = "linux")]
pub mod systemd;
#[cfg(target_os = "linux")]
pub mod openrc;
#[cfg(target_os = "linux")]
pub mod runit;
#[cfg(target_os = "linux")]
pub mod dinit;
#[cfg(target_os = "linux")]
pub mod s6;


#[cfg(target_os = "linux")]
pub use init::InitManager;
#[cfg(target_os = "linux")]
pub use systemd::SystemdManager;
#[cfg(target_os = "linux")]
pub use openrc::OpenRcManager;
#[cfg(target_os = "linux")]
pub use runit::RunitManager;
#[cfg(target_os = "linux")]
pub use dinit::DinitManager;
#[cfg(target_os = "linux")]
pub use s6::S6Manager;

// Windows
#[cfg(target_os = "windows")]
pub mod winservice;

#[cfg(target_os = "windows")]
pub use winservice::WindowsServiceManager;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitType {
    #[cfg(target_os = "linux")]
    Systemd,
    #[cfg(target_os = "linux")]
    OpenRc,
    #[cfg(target_os = "linux")]
    Runit,
    #[cfg(target_os = "linux")]
    Dinit,
    #[cfg(target_os = "linux")]
    S6,
    #[cfg(target_os = "linux")]
    Init,
    #[cfg(target_os = "windows")]
    Windows,
}

impl InitType {
    pub fn as_str(&self) -> &'static str {
        match self {
            #[cfg(target_os = "linux")]
            Self::Systemd => "systemd",
            #[cfg(target_os = "linux")]
            Self::OpenRc => "openrc",
            #[cfg(target_os = "linux")]
            Self::Runit => "runit",
            #[cfg(target_os = "linux")]
            Self::Dinit => "dinit",
            #[cfg(target_os = "linux")]
            Self::S6 => "s6",
            #[cfg(target_os = "linux")]
            Self::Init => "init",
            #[cfg(target_os = "windows")]
            Self::Windows => "windows",
        }
    }
}

/// Detect the active init system of the running OS.
pub fn detect_init_system() -> Option<InitType> {
    #[cfg(target_os = "windows")]
    {
        return Some(InitType::Windows);
    }

    #[cfg(target_os = "linux")]
    {
        // 1. Check systemd (standard directory /run/systemd/system)
        if Path::new("/run/systemd/system").exists() {
            return Some(InitType::Systemd);
        }

        // 2. Check OpenRC
        if Path::new("/run/openrc").exists() || Path::new("/sbin/openrc-run").exists() {
            return Some(InitType::OpenRc);
        }

        // 3. Check dinit (usually dinitctl exists or checking process name/etc)
        if Path::new("/etc/dinit.d").exists() {
            if std::process::Command::new("which").arg("dinitctl").output().map(|o| o.status.success()).unwrap_or(false) {
                return Some(InitType::Dinit);
            }
        }

        // 4. Check runit (directory /etc/runit or command runsvdir/sv)
        if Path::new("/etc/runit").exists() || Path::new("/var/service").exists() {
            if std::process::Command::new("which").arg("sv").output().map(|o| o.status.success()).unwrap_or(false) {
                return Some(InitType::Runit);
            }
        }

        // 5. Check s6
        if Path::new("/etc/s6").exists() {
            if std::process::Command::new("which").arg("s6-svstat").output().map(|o| o.status.success()).unwrap_or(false) {
                return Some(InitType::S6);
            }
        }

        // 6. Check classic SysV Init
        if Path::new("/etc/init.d").exists() {
            return Some(InitType::Init);
        }

        None
    }
}

/// Factory function to get the ServiceManager for the detected init system.
pub fn get_detected_manager() -> Option<Box<dyn ServiceManager>> {
    detect_init_system().map(|t| get_manager(t))
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
