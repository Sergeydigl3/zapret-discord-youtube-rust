#![cfg(target_os = "linux")]

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
        println!("Для работы требуются права root. Попытка инициализации pkexec...");
        let mut status = std::process::Command::new("pkexec")
            .arg(std::env::current_exe().unwrap_or_default())
            .args(std::env::args().skip(1))
            .status();

        let pkexec_failed = match &status {
            Ok(st) => !st.success(),
            Err(_) => true,
        };

        if pkexec_failed {
            if let Ok(ref st) = status {
                eprintln!("pkexec завершился с ошибкой (код: {})", st);
            } else if let Err(ref e) = status {
                eprintln!("Не удалось запустить pkexec: {}", e);
            }
            println!("Попытка инициализации sudo...");
            status = std::process::Command::new("sudo")
                .arg(std::env::current_exe().unwrap_or_default())
                .args(std::env::args().skip(1))
                .status();
        }

        match status {
            Ok(st) => {
                std::process::exit(st.code().unwrap_or(1));
            }
            Err(e) => {
                eprintln!("Ошибка запуска sudo/pkexec: {}. Попробуйте запустить вручную с правами root.", e);
                std::process::exit(1);
            }
        }
    }
}
