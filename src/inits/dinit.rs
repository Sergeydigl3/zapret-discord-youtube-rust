use std::fs;
use std::path::Path;
use std::process::Command;
use crate::inits::ServiceManager;

pub struct DinitManager;

impl DinitManager {
    const SERVICE_PATH: &'static str = "/etc/dinit.d/zapret-rust";
    const BOOT_DIR: &'static str = "/etc/dinit.d/boot.d";
    const BOOT_LINK: &'static str = "/etc/dinit.d/boot.d/zapret-rust";
    const SERVICE_NAME: &'static str = "zapret-rust";

    fn run_dinitctl(&self, action: &str) -> Result<(), String> {
        let output = Command::new("dinitctl")
            .arg(action)
            .arg(Self::SERVICE_NAME)
            .output()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_dinit"), e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!(
                "dinitctl {} {} failed: {}",
                action,
                Self::SERVICE_NAME,
                if stderr.is_empty() { format!("exit code {:?}", output.status.code()) } else { stderr }
            ))
        }
    }
}

impl ServiceManager for DinitManager {
    fn is_installed(&self) -> bool {
        Path::new(Self::SERVICE_PATH).exists()
    }

    fn is_active(&self) -> bool {
        let output = Command::new("dinitctl")
            .arg("status")
            .arg(Self::SERVICE_NAME)
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains("Status: started") || stdout.contains("Status: running")
            }
            _ => false,
        }
    }

    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String> {
        let exe_str = exe_path.to_str().ok_or(rust_i18n::t!("err_invalid_exe").into_owned())?;
        let config_str = config_path.to_str().ok_or(rust_i18n::t!("err_invalid_cfg").into_owned())?;
        let cache_str = cache_dir.to_str().ok_or(rust_i18n::t!("err_invalid_cache").into_owned())?;

        let service_content = format!(
            r#"type = simple
command = {} --config {} --cache-dir {}
restart = true
restart-delay = 5
"#,
            exe_str, config_str, cache_str
        );

        fs::write(Self::SERVICE_PATH, service_content)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_write_dinit"), e))?;

        // Try to enable it by symlinking in boot.d
        if let Err(e) = fs::create_dir_all(Self::BOOT_DIR) {
            println!("  (could not create boot.d directory, auto-start might not work: {})", e);
        } else {
            if Path::new(Self::BOOT_LINK).exists() || fs::symlink_metadata(Self::BOOT_LINK).is_ok() {
                let _ = fs::remove_file(Self::BOOT_LINK);
            }
            #[cfg(unix)]
            {
                let _ = std::os::unix::fs::symlink("../zapret-rust", Self::BOOT_LINK);
            }
        }

        Ok(())
    }

    fn uninstall(&self) -> Result<(), String> {
        // Stop service first (ignore errors)
        let _ = self.run_dinitctl("stop");

        // Remove boot.d symlink
        if Path::new(Self::BOOT_LINK).exists() || fs::symlink_metadata(Self::BOOT_LINK).is_ok() {
            let _ = fs::remove_file(Self::BOOT_LINK);
        }

        // Remove service file
        if Path::new(Self::SERVICE_PATH).exists() {
            fs::remove_file(Self::SERVICE_PATH)
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_rm_dinit"), e))?;
        }

        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        self.run_dinitctl("start")
    }

    fn stop(&self) -> Result<(), String> {
        self.run_dinitctl("stop")
    }

    fn restart(&self) -> Result<(), String> {
        self.run_dinitctl("restart")
    }
}
