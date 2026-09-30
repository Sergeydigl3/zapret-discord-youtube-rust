#![cfg(target_os = "windows")]

use crate::service::ServiceManager;
use std::path::Path;
use std::thread;
use std::time::Duration;

// We will use standard windows-service API when compiling on Windows
use windows_service::{
    service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState, ServiceType},
    service_manager::{ServiceManager as Scm, ServiceManagerAccess},
};

pub struct WindowsServiceManager;

impl WindowsServiceManager {
    /// Name of the service in the SCM. The service *runtime* lives in the
    /// binary (`src/daemon.rs`) and registers under this name, so it is part of
    /// the contract between the two.
    pub const SERVICE_NAME: &'static str = "zapret-rust";

    fn connect_scm() -> Result<Scm, String> {
        Scm::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_service"), e))
    }

    fn open_service(&self, access: ServiceAccess) -> Result<windows_service::service::Service, String> {
        let manager = Self::connect_scm()?;
        manager
            .open_service(Self::SERVICE_NAME, access)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_service"), e))
    }
}

impl ServiceManager for WindowsServiceManager {
    fn is_installed(&self) -> bool {
        self.open_service(ServiceAccess::QUERY_STATUS).is_ok()
    }

    fn is_active(&self) -> bool {
        match self.open_service(ServiceAccess::QUERY_STATUS) {
            Ok(svc) => svc
                .query_status()
                .map(|s| s.current_state == ServiceState::Running)
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String> {
        let config_str = config_path
            .to_str()
            .ok_or(rust_i18n::t!("err_invalid_cfg").into_owned())?;
        let cache_str = cache_dir
            .to_str()
            .ok_or(rust_i18n::t!("err_invalid_cache").into_owned())?;

        let manager = Scm::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_service"), e))?;

        let service_info = ServiceInfo {
            name: Self::SERVICE_NAME.into(),
            display_name: Self::SERVICE_NAME.into(),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: exe_path.to_path_buf(),
            launch_arguments: vec![
                "--service".into(),
                "--config".into(),
                config_str.into(),
                "--cache-dir".into(),
                cache_str.into(),
            ],
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };

        manager
            .create_service(&service_info, ServiceAccess::QUERY_STATUS)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_service"), e))?;

        Ok(())
    }

    fn uninstall(&self) -> Result<(), String> {
        // Stop service first (ignore errors)
        let _ = self.stop();
        self.open_service(ServiceAccess::DELETE)?
            .delete()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_service"), e))?;
        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        let svc = self.open_service(ServiceAccess::START | ServiceAccess::QUERY_STATUS)?;
        svc.start(&[] as &[&str])
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_service"), e))?;
        // Wait until the service actually reports RUNNING (like `sc start` did).
        for _ in 0..50 {
            if let Ok(status) = svc.query_status() {
                match status.current_state {
                    ServiceState::Running => return Ok(()),
                    ServiceState::Stopped => {
                        return Err(rust_i18n::t!("err_service_start_failed").into_owned());
                    }
                    _ => {}
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    fn stop(&self) -> Result<(), String> {
        let svc = self.open_service(ServiceAccess::STOP | ServiceAccess::QUERY_STATUS)?;
        svc.stop()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_service"), e))?;
        // `stop` returns as soon as the control is accepted; wait until fully stopped.
        for _ in 0..50 {
            match svc.query_status() {
                Ok(status) if status.current_state == ServiceState::Stopped => return Ok(()),
                _ => {}
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    fn restart(&self) -> Result<(), String> {
        let _ = self.stop();
        self.start()
    }
}
