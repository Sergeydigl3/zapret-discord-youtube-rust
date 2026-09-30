mod date;
mod facts;

use date::timestamp;
use facts::collect_system_info;
use std::fs::{self, OpenOptions};
use std::io::Write;

pub fn log_nfqws_launch(bin_path: &str, nfqws_params: &[String], terminal_output: &[String]) {
    let log_dir = crate::paths::cache_dir().join("logs");
    let log_file = log_dir.join("zapret.log");

    if fs::create_dir_all(&log_dir).is_err() {
        return;
    }

    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_file)
    {
        Ok(f) => f,
        Err(_) => return,
    };

    let ts = timestamp();

    let _ = writeln!(file, "===== nfqws | {} =====", ts);
    let _ = writeln!(file);

    let _ = writeln!(file, "--- System Info ---");
    for line in collect_system_info() {
        let _ = writeln!(file, "{}", line);
    }
    let _ = writeln!(file);

    let config_path = crate::paths::config_path();
    let config_content = fs::read_to_string(&config_path).unwrap_or_default();
    let _ = writeln!(file, "--- Config ---");
    let _ = write!(file, "{}", config_content);
    let _ = writeln!(file);

    if !terminal_output.is_empty() {
        let _ = writeln!(file, "--- Terminal ---");
        for line in terminal_output {
            let _ = writeln!(file, "{}", line);
        }
        let _ = writeln!(file);
    }

    let _ = writeln!(file, "--- Strategy params ---");
    let _ = writeln!(file, "binary: {}", bin_path);
    for param in nfqws_params {
        let _ = writeln!(file, "{}", param);
    }
}

pub fn log_stop(stop_output: &[String]) {
    let log_file = crate::paths::cache_dir().join("logs").join("zapret.log");
    let mut file = match OpenOptions::new().create(true).append(true).open(&log_file) {
        Ok(f) => f,
        Err(_) => return,
    };

    let ts = timestamp();
    let _ = writeln!(file);
    let _ = writeln!(file, "===== stop | {} =====", ts);
    for line in stop_output {
        let _ = writeln!(file, "{}", line);
    }
}
