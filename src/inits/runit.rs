use std::fs;
use std::path::Path;
use std::process::Command;
use crate::inits::ServiceManager;

pub struct RunitManager;

impl RunitManager {
    const SV_DIR: &'static str = "/etc/sv/zapret-rust";
    const SERVICE_NAME: &'static str = "zapret-rust";

    fn get_link_path(&self) -> &'static str {
        if Path::new("/service").exists() && !Path::new("/var/service").exists() {
            "/service/zapret-rust"
        } else {
            "/var/service/zapret-rust"
        }
    }

    fn run_sv(&self, action: &str) -> Result<(), String> {
        let output = Command::new("sv")
            .arg(action)
            .arg(Self::SERVICE_NAME)
            .output()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_sv"), e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!(
                "sv {} {} failed: {}",
                action,
                Self::SERVICE_NAME,
                if stderr.is_empty() { format!("exit code {:?}", output.status.code()) } else { stderr }
            ))
        }
    }
}

impl ServiceManager for RunitManager {
    fn is_installed(&self) -> bool {
        Path::new(Self::SV_DIR).exists()
    }

    fn is_active(&self) -> bool {
        let output = Command::new("sv")
            .arg("status")
            .arg(Self::SERVICE_NAME)
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.trim().starts_with("run:")
            }
            _ => false,
        }
    }

    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String> {
        let exe_str = exe_path.to_str().ok_or(rust_i18n::t!("err_invalid_exe").into_owned())?;
        let config_str = config_path.to_str().ok_or(rust_i18n::t!("err_invalid_cfg").into_owned())?;
        let cache_str = cache_dir.to_str().ok_or(rust_i18n::t!("err_invalid_cache").into_owned())?;

        // 1. Create SV dir
        fs::create_dir_all(Self::SV_DIR)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_mkdir_runit"), e))?;

        // 2. Write run file
        let run_path = Path::new(Self::SV_DIR).join("run");
        let run_content = format!(
            r#"#!/bin/sh
exec 2>&1
exec {} --config {} --cache-dir {}
"#,
            exe_str, config_str, cache_str
        );
        fs::write(&run_path, run_content)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_write_run"), e))?;

        // 3. Make run script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&run_path, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_chmod_run"), e))?;
        }

        // 4. Link service
        let link_path = self.get_link_path();
        if Path::new(link_path).exists() || fs::symlink_metadata(link_path).is_ok() {
            let _ = fs::remove_file(link_path);
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(Self::SV_DIR, link_path)
                .map_err(|e| format!("{}{}: {}", rust_i18n::t!("err_symlink"), link_path, e))?;
        }

        Ok(())
    }

    fn uninstall(&self) -> Result<(), String> {
        // Stop the service
        let _ = self.run_sv("stop");

        // Remove symlink
        let link_path = self.get_link_path();
        if Path::new(link_path).exists() || fs::symlink_metadata(link_path).is_ok() {
            fs::remove_file(link_path)
                .map_err(|e| format!("{}{}: {}", rust_i18n::t!("err_rm_symlink"), link_path, e))?;
        }

        // Remove sv directory
        if Path::new(Self::SV_DIR).exists() {
            fs::remove_dir_all(Self::SV_DIR)
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_rm_runit"), e))?;
        }

        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        self.run_sv("start")
    }

    fn stop(&self) -> Result<(), String> {
        self.run_sv("stop")
    }

    fn restart(&self) -> Result<(), String> {
        self.run_sv("restart")
    }
}
