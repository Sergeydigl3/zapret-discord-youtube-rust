#![cfg(target_os = "linux")]

use std::os::unix::process::CommandExt;

/// Ensures that the current process is running with root privileges.
/// If not, it attempts to escalate privileges using pkexec or sudo.
pub fn ensure_admin() {
    let not_root = std::process::Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim() != "0")
        .unwrap_or(true);

    if not_root {
        println!("Для работы требуются права root. Перезапуск с повышенными привилегиями...");

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

        eprintln!("Ошибка повышения привилегий: sudo не удалось запустить ({})", err2);
        std::process::exit(1);
    }
}
