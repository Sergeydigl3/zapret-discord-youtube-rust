#![cfg(target_os = "windows")]

use std::env;
use std::path::PathBuf;
use std::process::Command;

#[link(name = "kernel32")]
extern "system" {
    fn SetConsoleCP(wCodePageID: u32) -> i32;
    fn SetConsoleOutputCP(wCodePageID: u32) -> i32;
}

/// Prepare the Windows console for a colorful, UTF-8 TUI:
/// - switch it to UTF-8 (codepage 65001) so emoji and localized text emitted
///   by Rust's UTF-8 stdout are decoded correctly instead of being garbled by
///   the OEM codepage;
/// - explicitly enable ANSI/VT processing so ratatui's color output is
///   rendered even before crossterm happens to enable it lazily.
pub fn setup_console() {
    unsafe {
        SetConsoleCP(65001);
        SetConsoleOutputCP(65001);
    }
    let _ = crossterm::ansi_support::supports_ansi();
}

/// Ensures that the current process is running with elevated (Administrator) privileges.
/// If not, it requests UAC elevation and exits the current process.
///
/// When possible the elevated process is started inside a Windows Terminal
/// window: the legacy console host (conhost) renders everything monochrome -
/// both ANSI colors and emoji - while Windows Terminal honors them, matching
/// the colorful Linux terminal experience.
pub fn ensure_admin() {
    let elevated = is_elevated();

    if elevated && in_windows_terminal() {
        return;
    }

    let exe_path = env::current_exe().unwrap();
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if wt_available() {
        let verb = if elevated { "" } else { " -Verb RunAs" };
        let ps = format!(
            "Start-Process -FilePath 'wt'{} -ArgumentList @('-d', '\"{}\"', '\"{}\"')",
            verb,
            cwd.display(),
            exe_path.display()
        );
        if run_powershell(&ps) {
            std::process::exit(0);
        }
        if elevated {
            // Already admin but could not hop into Windows Terminal: keep
            // running in the current console.
            return;
        }
        eprintln!("Failed to elevate privileges. Please run as Administrator.");
        std::process::exit(1);
    }

    // Not elevated and Windows Terminal is unavailable: relaunch with UAC in
    // the plain console as a fallback.
    let ps = format!("Start-Process -FilePath \"{}\" -Verb RunAs", exe_path.display());
    if run_powershell(&ps) {
        std::process::exit(0);
    }

    eprintln!("Failed to elevate privileges. Please run as Administrator.");
    std::process::exit(1);
}

/// True if the current process is running inside Windows Terminal.
fn in_windows_terminal() -> bool {
    env::var("WT_SESSION").is_ok()
}

/// True if Windows Terminal (`wt.exe`) is installed and can be launched.
///
/// Must not spawn `wt` itself: `wt --version` – and even plain `wt` – opens a
/// Windows Terminal window instead of being harmless, which would pop up an
/// unwanted window next to the real one.
fn wt_available() -> bool {
    let on_path = Command::new("where.exe")
        .arg("wt")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Fallback: App Execution Alias location for Store-installed Terminal.
    on_path
        || env::var_os("LOCALAPPDATA").map_or(false, |p| {
            PathBuf::from(p)
                .join("Microsoft")
                .join("WindowsApps")
                .join("wt.exe")
                .exists()
        })
}

fn run_powershell(script: &str) -> bool {
    Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn is_elevated() -> bool {
    Command::new("fsutil")
        .arg("dirty")
        .arg("query")
        .arg(env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string()))
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

pub fn is_nfqws_running() -> bool {
    let out = Command::new("tasklist")
        .args(["/FI", "IMAGENAME eq winws.exe", "/NH"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_default();
    out.contains("winws.exe")
}
