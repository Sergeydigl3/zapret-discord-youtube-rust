use std::fs;
use std::path::Path;
use std::process::Command;
use crate::inits::ServiceManager;

pub struct InitManager;

impl InitManager {
    const SCRIPT_PATH: &'static str = "/etc/init.d/zapret-rust";
    const SERVICE_NAME: &'static str = "zapret-rust";

    fn run_init_script(&self, action: &str) -> Result<(), String> {
        let output = Command::new(Self::SCRIPT_PATH)
            .arg(action)
            .output()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_init"), e))?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(format!(
                "{} {} failed: {}",
                Self::SCRIPT_PATH,
                action,
                if stderr.is_empty() { format!("exit code {:?}", output.status.code()) } else { stderr }
            ))
        }
    }

    fn register_service(&self) -> Result<(), String> {
        // Try update-rc.d first (Debian/Ubuntu/Devuan)
        if Command::new("which").arg("update-rc.d").output().map(|o| o.status.success()).unwrap_or(false) {
            let output = Command::new("update-rc.d")
                .arg(Self::SERVICE_NAME)
                .arg("defaults")
                .output()
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_update_rc"), e))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(format!("update-rc.d failed: {}", stderr));
            }
        }
        // Fallback to chkconfig (RedHat/CentOS/openSUSE)
        else if Command::new("which").arg("chkconfig").output().map(|o| o.status.success()).unwrap_or(false) {
            let output = Command::new("chkconfig")
                .arg("--add")
                .arg(Self::SERVICE_NAME)
                .output()
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_exec_chkconfig"), e))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(format!("chkconfig --add failed: {}", stderr));
            }
        }
        Ok(())
    }

    fn unregister_service(&self) -> Result<(), String> {
        if Command::new("which").arg("update-rc.d").output().map(|o| o.status.success()).unwrap_or(false) {
            let _ = Command::new("update-rc.d")
                .args(["-f", Self::SERVICE_NAME, "remove"])
                .output();
        } else if Command::new("which").arg("chkconfig").output().map(|o| o.status.success()).unwrap_or(false) {
            let _ = Command::new("chkconfig")
                .arg("--del")
                .arg(Self::SERVICE_NAME)
                .output();
        }
        Ok(())
    }
}

impl ServiceManager for InitManager {
    fn is_installed(&self) -> bool {
        Path::new(Self::SCRIPT_PATH).exists()
    }

    fn is_active(&self) -> bool {
        if !self.is_installed() {
            return false;
        }
        let output = Command::new(Self::SCRIPT_PATH)
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
            r#"#!/bin/sh
### BEGIN INIT INFO
# Provides:          zapret-rust
# Required-Start:    $network $local_fs
# Required-Stop:     $network $local_fs
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Short-Description: Zapret Discord Youtube Service
### END INIT INFO

DESC="Zapret Discord Youtube Service"
NAME="zapret-rust"
DAEMON="{}"
DAEMON_ARGS="--config {} --cache-dir {}"
PIDFILE="/var/run/$NAME.pid"

case "$1" in
    start)
        echo "Starting $DESC"
        start-stop-daemon --start --background --make-pidfile --pidfile "$PIDFILE" --exec "$DAEMON" -- $DAEMON_ARGS
        ;;
    stop)
        echo "Stopping $DESC"
        start-stop-daemon --stop --pidfile "$PIDFILE" --retry 5
        ;;
    restart)
        $0 stop
        $0 start
        ;;
    status)
        if [ -f "$PIDFILE" ] && kill -0 $(cat "$PIDFILE") 2>/dev/null; then
            echo "$DESC is running"
            exit 0
        else
            echo "$DESC is stopped"
            exit 3
        fi
        ;;
    *)
        echo "Usage: $0 {{start|stop|restart|status}}"
        exit 1
        ;;
esac
"#,
            exe_str, config_str, cache_str
        );

        fs::write(Self::SCRIPT_PATH, script_content)
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_write_sysv"), e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(Self::SCRIPT_PATH, fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_chmod_sysv"), e))?;
        }

        self.register_service()?;

        Ok(())
    }

    fn uninstall(&self) -> Result<(), String> {
        // Stop service first (ignore errors)
        let _ = self.run_init_script("stop");
        let _ = self.unregister_service();

        if Path::new(Self::SCRIPT_PATH).exists() {
            fs::remove_file(Self::SCRIPT_PATH)
                .map_err(|e| format!("{}{}", rust_i18n::t!("err_rm_sysv"), e))?;
        }

        Ok(())
    }

    fn start(&self) -> Result<(), String> {
        self.run_init_script("start")
    }

    fn stop(&self) -> Result<(), String> {
        self.run_init_script("stop")
    }

    fn restart(&self) -> Result<(), String> {
        self.run_init_script("restart")
    }
}
