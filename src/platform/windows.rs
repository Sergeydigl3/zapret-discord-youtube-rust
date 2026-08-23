#![cfg(target_os = "windows")]

/// Ensures that the current process is running with elevated (Administrator) privileges.
/// If not, it requests UAC elevation by spawning a new PowerShell process and exits the current process.
pub fn ensure_admin() {
    let is_elevated = is_elevated::is_elevated();

    if !is_elevated {
        println!("Requesting Administrator privileges...");
        let exe_path = std::env::current_exe().unwrap();
        let status = std::process::Command::new("powershell")
            .arg("-Command")
            .arg(format!(
                "Start-Process -FilePath \"{}\" -Verb RunAs",
                exe_path.display()
            ))
            .status();

        if let Ok(st) = status {
            if st.success() {
                std::process::exit(0);
            }
        }

        eprintln!("Failed to elevate privileges. Please run as Administrator.");
        std::process::exit(1);
    }
}

pub fn is_nfqws_running() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let is_running = sys.processes_by_exact_name(std::ffi::OsStr::new("winws.exe")).next().is_some();
    is_running
}
