#![cfg(target_os = "windows")]

use crate::inits::ServiceManager;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

// We will use standard windows-service API when compiling on Windows
use windows_service::{
    define_windows_service,
    service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode, ServiceInfo,
        ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
    service_manager::{ServiceManager as Scm, ServiceManagerAccess},
};

pub struct WindowsServiceManager;

impl WindowsServiceManager {
    const SERVICE_NAME: &'static str = "zapret-rust";

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

// Windows Service Runtime Implementation
static RUNNING: AtomicBool = AtomicBool::new(true);

define_windows_service!(ffi_service_main, my_service_main);

pub fn run_service() -> Result<(), String> {
    service_dispatcher::start(WindowsServiceManager::SERVICE_NAME, ffi_service_main)
        .map_err(|e| format!("{}{}", rust_i18n::t!("err_start_dispatcher"), e))
}

fn my_service_main(_arguments: Vec<std::ffi::OsString>) {
    let status_handle = match service_control_handler::register(
        WindowsServiceManager::SERVICE_NAME,
        move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop => {
                    RUNNING.store(false, Ordering::SeqCst);
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        },
    ) {
        Ok(h) => h,
        Err(_) => return,
    };

    // Report Running state
    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    });

    // Parse options from standard arguments (since binPath arguments are passed to the process)
    let args: Vec<String> = std::env::args().collect();
    let mut config_path = None;
    let mut cache_dir = None;

    for i in 0..args.len() {
        if args[i] == "--config" && i + 1 < args.len() {
            config_path = Some(args[i + 1].clone());
        } else if args[i] == "--cache-dir" && i + 1 < args.len() {
            cache_dir = Some(args[i + 1].clone());
        }
    }

    let config_file = match config_path {
        Some(c) => c,
        None => {
            report_stopped(&status_handle, 1);
            return;
        }
    };

    if let Some(ref d) = cache_dir {
        std::env::set_var("ZAPRET_CACHE_DIR", d);
    }

    // Load Configuration
    let cfg = match crate::config::load_config(&config_file) {
        Ok(c) => c,
        Err(_) => {
            report_stopped(&status_handle, 2);
            return;
        }
    };

    // Run Zapret background loop
    let backend = crate::firewalls::windivert::WinDivertBackend;

    crate::runner::run_zapret(
        &cfg.strategy,
        &cfg.interface,
        cfg.gamefilter_tcp,
        cfg.gamefilter_udp,
        &backend,
    );

    // Main service loop
    while RUNNING.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(100));
    }

    // Cleanup and stop
    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StopPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(5),
        process_id: None,
    });

    crate::runner::stop_zapret(&backend);

    report_stopped(&status_handle, 0);
}

fn report_stopped(status_handle: &ServiceStatusHandle, exit_code: u32) {
    let _ = status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: if exit_code == 0 {
            ServiceExitCode::Win32(0)
        } else {
            ServiceExitCode::Win32(exit_code)
        },
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    });
}
