#![cfg(target_os = "linux")]

use std::os::unix::process::CommandExt;

/// Ensures that the current process is running with root privileges.
/// If not, it attempts to escalate privileges using pkexec or sudo.
pub fn ensure_admin() {
    let not_root = !is_elevated::is_elevated();

    if not_root {
        println!("{}", rust_i18n::t!("root_req"));

        // Try pkexec first via exec().
        // exec() replaces the current process. It only returns if it fails to start the binary.
        let _err1 = std::process::Command::new("pkexec")
            .arg(std::env::current_exe().unwrap_or_default())
            .args(std::env::args().skip(1))
            .exec();

        // If we reach here, pkexec is not installed or failed to execute. Fall back to sudo.
        let err2 = std::process::Command::new("sudo")
            .arg(std::env::current_exe().unwrap_or_default())
            .args(std::env::args().skip(1))
            .exec();

        eprintln!("{} ({})", rust_i18n::t!("root_err_sudo"), err2);
        std::process::exit(1);
    }
}

pub fn is_nfqws_running() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let is_running = sys.processes_by_exact_name(std::ffi::OsStr::new("nfqws")).next().is_some();
    is_running
}
