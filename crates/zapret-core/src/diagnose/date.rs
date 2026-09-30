use std::process::Command;

/// Timestamp header for a log section.
///
/// Prefers the platform `date` / `cmd echo` so the output matches the local
/// timezone; the hand-rolled civil-date conversion below is only the fallback
/// for when that shell-out fails, and it is therefore always UTC.
pub(crate) fn timestamp() -> String {
    #[cfg(target_os = "linux")]
    if let Ok(output) = Command::new("date").args(["+%Y-%m-%d %H:%M:%S"]).output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            let t = s.trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }

    #[cfg(target_os = "windows")]
    if let Ok(output) = Command::new("cmd").args(["/c", "echo %DATE% %TIME%"]).output() {
        if let Ok(s) = String::from_utf8(output.stdout) {
            let t = s.trim().to_string();
            if !t.is_empty() {
                return t;
            }
        }
    }

    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();

    let days = total_secs / 86400;
    let time = total_secs % 86400;
    let hours = time / 3600;
    let minutes = (time % 3600) / 60;
    let seconds = time % 60;

    fn is_leap(year: u64) -> bool {
        (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
    }

    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y as u64) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let month_days: &[i64] = if is_leap(y as u64) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0u32;
    for (i, &days_in_m) in month_days.iter().enumerate() {
        if remaining < days_in_m {
            m = (i + 1) as u32;
            break;
        }
        remaining -= days_in_m;
    }
    let d = remaining + 1;

    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, m, d, hours, minutes, seconds)
}
