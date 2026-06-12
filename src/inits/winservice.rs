#![cfg(target_os = "windows")]

use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use crate::inits::ServiceManager;

// We will use standard windows-service API when compiling on Windows
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher,
};

pub struct WindowsServiceManager;

impl WindowsServiceManager {
    const SERVICE_NAME: &'static str = "zapret-rust";

    fn run_sc<S: AsRef<std::ffi::OsStr>>(&self, args: &[S]) -> Result<(), String> {
        let output = Command::new("sc")
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute sc: {}", e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let cmd_str = args.iter()
                .map(|a| a.as_ref().to_string_lossy().into_owned())
                .collect::<Vec<String>>()
                .join(" ");
            Err(format!(
                "sc {} failed: {}",
                cmd_str,
                if stderr.is_empty() { stdout } else { stderr }
            ))
        }
    }
}

impl ServiceManager for WindowsServiceManager {
    fn is_installed(&self) -> bool {
        let output = Command::new("sc")
            .arg("query")
            .arg(Self::SERVICE_NAME)
            .output();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // If the service doesn't exist, sc query fails (often code 1060) and doesn't contain the service name
                out.status.success() || stdout.contains("SERVICE_NAME")
            }
            Err(_) => false,
        }
    }

    fn is_active(&self) -> bool {
        let output = Command::new("sc")
            .arg("query")
            .arg(Self::SERVICE_NAME)
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains("RUNNING")
            }
            _ => false,
        }
    }

    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String> {
        let exe_str = exe_path.to_str().ok_or("Invalid executable path")?;
        let config_str = config_path.to_str().ok_or("Invalid config path")?;
        let cache_str = cache_dir.to_str().ok_or("Invalid cache directory path")?;

        // Format binPath with correct arguments. SCM expects space after 'binPath=' and 'start='
        let bin_path_arg = format!(
            "\"{}\" --service --config \"{}\" --cache-dir \"{}\"",
            exe_str, config_str, cache_str
        );

        self.run_sc(&[
            "create",
            Self::SERVICE_NAME,
            "binPath=",
            &bin_path_arg,
            "start=",
            "auto",
        ])?;

        Ok(())
    }

    fn uninstall(&self) -> Result<(), String> {
        // Stop service first (ignore errors)
        let _ = Command::new("sc").arg("stop").arg(Self::SERVICE_NAME).output();
        thread::sleep(Duration::from_millis(500));

        self.run_sc(&["delete", Self::SERVICE_NAME])?;
        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        self.run_sc(&["start", Self::SERVICE_NAME])
    }

    fn stop(&self) -> Result<(), String> {
        self.run_sc(&["stop", Self::SERVICE_NAME])
    }

    fn restart(&self) -> Result<(), String> {
        let _ = self.stop();
        thread::sleep(Duration::from_secs(1));
        self.start()
    }
}

// Windows Service Runtime Implementation
static RUNNING: AtomicBool = AtomicBool::new(true);

define_windows_service!(ffi_service_main, my_service_main);

pub fn run_service() -> Result<(), String> {
    service_dispatcher::start(WindowsServiceManager::SERVICE_NAME, ffi_service_main)
        .map_err(|e| format!("Failed to start service dispatcher: {}", e))
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
