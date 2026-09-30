//! Windows service runtime.
//!
//! The binary's second entry point: when the SCM starts the process with
//! `--service`, this hands control to the dispatcher and then boots the whole
//! application (config load, WinDivert backend, zapret loop) instead of the
//! TUI. It lives in the binary rather than in `zapret-core`'s service layer
//! because it is a process entry point, not a service manager - the SCM client
//! in `zapret_core::service::windows` is the other half of the same service.

#![cfg(target_os = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use windows_service::{
    define_windows_service,
    service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType},
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
};
use zapret_core::service::windows::WindowsServiceManager;

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
    let cfg = match zapret_core::config::load_config(&config_file) {
        Ok(c) => c,
        Err(_) => {
            report_stopped(&status_handle, 2);
            return;
        }
    };

    // Run Zapret background loop
    let backend = zapret_core::firewall::windivert::WinDivertBackend;

    let req = zapret_core::plan::RunRequest::new(&cfg.strategy, &cfg.interface, cfg.gamefilter_tcp, cfg.gamefilter_udp);
    zapret_core::run::run_foreground(&req, &backend);

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

    zapret_core::run::stop(&backend);

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
