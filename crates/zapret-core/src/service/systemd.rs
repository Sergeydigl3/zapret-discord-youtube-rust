use super::script::daemon_paths;
use crate::service::ServiceManager;
use std::fs;
use std::path::Path;
use std::process::Command;

pub struct SystemdManager;

impl SystemdManager {
    const SERVICE_NAME: &'static str = "zapret-rust";
    const SERVICE_PATH: &'static str = "/etc/systemd/system/zapret-rust.service";

    fn run_command(&self, args: &[&str]) -> Result<(), String> {
        let output = Command::new("systemctl")
            .args(args)
            .output()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_systemctl"), e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!(
                "systemctl {} failed: {}",
                args.join(" "),
                if stderr.is_empty() {
                    format!("exit code {:?}", output.status.code())
                } else {
                    stderr
                }
            ))
        }
    }
}

impl ServiceManager for SystemdManager {
    fn is_installed(&self) -> bool {
        Path::new(Self::SERVICE_PATH).exists()
    }

    fn is_active(&self) -> bool {
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg(Self::SERVICE_NAME)
            .output();
        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }

    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String> {
        let paths = daemon_paths(exe_path, config_path, cache_dir)?;

        let service_content = format!(
            r#"[Unit]
Description=Zapret Discord Youtube Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart={}
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
"#,
            paths.command()
        );

        fs::write(Self::SERVICE_PATH, service_content)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_write_svc"), e))?;

        self.run_command(&["daemon-reload"])?;
        self.run_command(&["enable", Self::SERVICE_NAME])?;

        Ok(())
    }

    fn uninstall(&self) -> Result<(), String> {
        // Stop service first (ignore errors if it's not running or doesn't exist)
        let _ = self.run_command(&["stop", Self::SERVICE_NAME]);
        let _ = self.run_command(&["disable", Self::SERVICE_NAME]);

        if Path::new(Self::SERVICE_PATH).exists() {
            fs::remove_file(Self::SERVICE_PATH).map_err(|e| format!("{}{}", rust_i18n::t!("err_rm_svc"), e))?;
        }

        self.run_command(&["daemon-reload"])?;
        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        self.run_command(&["start", Self::SERVICE_NAME])
    }

    fn stop(&self) -> Result<(), String> {
        self.run_command(&["stop", Self::SERVICE_NAME])
    }

    fn restart(&self) -> Result<(), String> {
        self.run_command(&["restart", Self::SERVICE_NAME])
    }
}
