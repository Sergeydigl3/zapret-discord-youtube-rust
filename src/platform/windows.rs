#![cfg(target_os = "windows")]

/// Ensures that the current process is running with elevated (Administrator) privileges.
/// If not, it requests UAC elevation by spawning a new PowerShell process and exits the current process.
pub fn ensure_admin() {
    let output = std::process::Command::new("fsutil")
        .arg("dirty")
        .arg("query")
        .arg(std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string()))
        .output();

    let is_elevated = match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    };

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
    std::process::Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq nfqws.exe", "/NH"])
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            out.contains("nfqws.exe")
        })
        .unwrap_or(false)
}
