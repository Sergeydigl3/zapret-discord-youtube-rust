use std::fs;
use std::path::Path;
use std::process::Command;
use crate::inits::ServiceManager;

pub struct OpenRcManager;

impl OpenRcManager {
    const SERVICE_NAME: &'static str = "zapret-rust";
    const SCRIPT_PATH: &'static str = "/etc/init.d/zapret-rust";

    fn run_rc_service(&self, action: &str) -> Result<(), String> {
        let output = Command::new("rc-service")
            .arg(Self::SERVICE_NAME)
            .arg(action)
            .output()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_rc_svc"), e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!(
                "rc-service {} {} failed: {}",
                Self::SERVICE_NAME,
                action,
                if stderr.is_empty() { format!("exit code {:?}", output.status.code()) } else { stderr }
            ))
        }
    }

    fn run_rc_update(&self, action: &str, runlevel: &str) -> Result<(), String> {
        let output = Command::new("rc-update")
            .arg(action)
            .arg(Self::SERVICE_NAME)
            .arg(runlevel)
            .output()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_rc_update"), e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!(
                "rc-update {} {} {} failed: {}",
                action,
                Self::SERVICE_NAME,
                runlevel,
                if stderr.is_empty() { format!("exit code {:?}", output.status.code()) } else { stderr }
            ))
        }
    }
}

impl ServiceManager for OpenRcManager {
    fn is_installed(&self) -> bool {
        Path::new(Self::SCRIPT_PATH).exists()
    }

    fn is_active(&self) -> bool {
        let output = Command::new("rc-service")
            .arg(Self::SERVICE_NAME)
            .arg("status")
            .output();
        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String> {
        let exe_str = exe_path.to_str().ok_or(rust_i18n::t!("err_invalid_exe").into_owned())?;
        let config_str = config_path.to_str().ok_or(rust_i18n::t!("err_invalid_cfg").into_owned())?;
        let cache_str = cache_dir.to_str().ok_or(rust_i18n::t!("err_invalid_cache").into_owned())?;

        let script_content = format!(
            r#"#!/sbin/openrc-run

description="Zapret Discord Youtube Service"
supervisor="supervise-daemon"
respawn_delay=5
respawn_max=10

command="{}"
command_args="--config {} --cache-dir {}"

depend() {{
    need net
    after firewall
}}
"#,
            exe_str, config_str, cache_str
        );

        fs::write(Self::SCRIPT_PATH, script_content)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_write_openrc"), e))?;

        #[cfg(unix)]
        {{
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(Self::SCRIPT_PATH, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_chmod_openrc"), e))?;
        }}

        self.run_rc_update("add", "default")?;

        Ok(())
    }

    fn uninstall(&self) -> Result<(), String> {
        // Stop service first (ignore errors)
        let _ = self.run_rc_service("stop");
        let _ = self.run_rc_update("del", "default");

        if Path::new(Self::SCRIPT_PATH).exists() {
            fs::remove_file(Self::SCRIPT_PATH)
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_rm_openrc"), e))?;
        }

        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        self.run_rc_service("start")
    }

    fn stop(&self) -> Result<(), String> {
        self.run_rc_service("stop")
    }

    fn restart(&self) -> Result<(), String> {
        self.run_rc_service("restart")
    }
}
