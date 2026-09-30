use std::process::Command;

/// Timestamp header for a log section.
///
/// Prefers the platform `date` / `cmd echo` so the output matches the local
/// timezone; the hand-rolled civil-date conversion in [`utc_timestamp`] is only
/// the fallback for when that shell-out fails, and it is therefore always UTC.
pub(crate) fn timestamp() -> String {
    local_timestamp().unwrap_or_else(utc_timestamp)
}

/// The local wall clock, spelled the way each platform spells it.
fn local_timestamp() -> Option<String> {
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/c", "echo %DATE% %TIME%"]).output()
    } else {
        Command::new("date").args(["+%Y-%m-%d %H:%M:%S"]).output()
    }
    .ok()?;

    let t = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}

/// UTC, computed from the system clock, for when the platform command fails.
fn utc_timestamp() -> String {
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
