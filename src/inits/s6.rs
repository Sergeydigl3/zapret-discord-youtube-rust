use std::fs;
use std::path::Path;
use std::process::Command;
use crate::inits::ServiceManager;

pub struct S6Manager;

impl S6Manager {
    const SERVICE_DIR: &'static str = "/etc/s6/services/zapret-rust";

    fn run_s6_svc(&self, flag: &str) -> Result<(), String> {
        let output = Command::new("s6-svc")
            .arg(flag)
            .arg(Self::SERVICE_DIR)
            .output()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_s6"), e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!(
                "s6-svc {} {} failed: {}",
                flag,
                Self::SERVICE_DIR,
                if stderr.is_empty() { format!("exit code {:?}", output.status.code()) } else { stderr }
            ))
        }
    }
}

impl ServiceManager for S6Manager {
    fn is_installed(&self) -> bool {
        Path::new(Self::SERVICE_DIR).exists()
    }

    fn is_active(&self) -> bool {
        let output = Command::new("s6-svstat")
            .arg(Self::SERVICE_DIR)
            .output();
        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains("up (pid")
            }
            _ => false,
        }
    }

    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String> {
        let exe_str = exe_path.to_str().ok_or(rust_i18n::t!("err_invalid_exe").into_owned())?;
        let config_str = config_path.to_str().ok_or(rust_i18n::t!("err_invalid_cfg").into_owned())?;
        let cache_str = cache_dir.to_str().ok_or(rust_i18n::t!("err_invalid_cache").into_owned())?;

        // 1. Create S6 dir
        fs::create_dir_all(Self::SERVICE_DIR)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_mkdir_s6"), e))?;

        // 2. Write run file
        let run_path = Path::new(Self::SERVICE_DIR).join("run");
        let run_content = format!(
            r#"#!/bin/sh
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

        Ok(())
    }

    fn uninstall(&self) -> Result<(), String> {
        // Stop service first (ignore errors)
        let _ = self.run_s6_svc("-d");

        // Remove service directory
        if Path::new(Self::SERVICE_DIR).exists() {
            fs::remove_dir_all(Self::SERVICE_DIR)
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_rm_s6"), e))?;
        }

        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        self.run_s6_svc("-u")
    }

    fn stop(&self) -> Result<(), String> {
        self.run_s6_svc("-d")
    }

    fn restart(&self) -> Result<(), String> {
        self.run_s6_svc("-r")
    }
}
