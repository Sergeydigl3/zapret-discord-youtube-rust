//! Init-system detection.
//!
//! Kept apart from the manager facade: detection only decides *which* manager
//! applies, it never touches one, so the two can be read independently.

#[cfg(target_os = "linux")]
use std::path::Path;

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
        Some(InitType::Windows)
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
            if std::process::Command::new("which")
                .arg("dinitctl")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return Some(InitType::Dinit);
            }
        }

        // 4. Check runit (directory /etc/runit or command runsvdir/sv)
        if Path::new("/etc/runit").exists() || Path::new("/var/service").exists() {
            if std::process::Command::new("which")
                .arg("sv")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return Some(InitType::Runit);
            }
        }

        // 5. Check s6
        if Path::new("/etc/s6").exists() {
            if std::process::Command::new("which")
                .arg("s6-svstat")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
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
