//! Fixed-DPI-TTL sweep.
//!
//! Depends on `domains` for the domain list file only. The autotune feature is
//! not involved, which is what keeps the two features from forming a cycle.

use std::time::Duration;

use zapret_core::firewall::FirewallBackend;

/// TTL sweep range (DPI hop numbers are typically 3-20).
pub const TTL_MIN: u8 = 3;
pub const TTL_MAX: u8 = 20;

const TEST_DOMAINS: &[&str] = &[
    "discord.com",
    "youtube.com",
    "cdn.discordapp.com",
    "googlevideo.com",
    "discord.media",
];
const EXTRA_DOMAINS_FILE: &str = "ttl_domains.txt";

pub fn ttl_domains_file_path() -> std::path::PathBuf {
    crate::paths::cache_dir().join(EXTRA_DOMAINS_FILE)
}

/// Create/refresh the TTL domain file with the built-in test domains so the
/// user can add/remove domains freely.
pub fn ensure_ttl_file() -> Result<(), String> {
    let path = ttl_domains_file_path();
    if path.exists() && !super::load_domain_file(&path).is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create directory '{}': {}", parent.display(), e))?;
    }
    let mut content = format!("# {}\n", rust_i18n::t!("domain_file_header_ttl"));
    for d in TEST_DOMAINS {
        content.push_str(d);
        content.push('\n');
    }
    std::fs::write(&path, content).map_err(|e| format!("Cannot write '{}': {}", path.display(), e))
}

/// Test domains come entirely from `ttl_domains.txt`; if the file is missing or
/// empty, the built-in defaults are used as a fallback.
fn get_test_domains() -> Vec<String> {
    let from_file = super::load_domain_file(&ttl_domains_file_path());
    if !from_file.is_empty() {
        return from_file;
    }
    TEST_DOMAINS.iter().map(|s| s.to_string()).collect()
}

/// Check that the domain is reachable over TLS 1.3.
///
/// `-k` skips certificate verification: the probe only checks that the TCP/TLS
/// connection gets through the DPI, and `googlevideo.com` (apex of YouTube's
/// video CDN) serves a wildcard cert that does not match the bare hostname.
///
/// The autotune probes run their own curl checks with different arguments and
/// different timeouts, so this one is deliberately not shared with them.
fn curl_tls_ok(domain: &str) -> bool {
    std::process::Command::new("curl")
        .arg("-s")
        .arg("-k")
        .args([
            "--tlsv1.3",
            "--connect-timeout",
            "3",
            "--max-time",
            "3",
            "-o",
            crate::platform::null_device(),
        ])
        .arg(format!("https://{}", domain))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Wait until the daemon this program just started is up.
///
/// The child handle is the answer: the sweep launched it, and nothing else can
/// be mistaken for it.
fn wait_for_nfqws(timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    let mut running = false;
    while std::time::Instant::now() < deadline {
        if zapret_core::daemon::is_running() {
            running = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    if running {
        // Let the daemon bind its socket before probing.
        std::thread::sleep(Duration::from_millis(500));
    }
    running
}

/// One thing the sweep did, so the caller can draw it instead of holding the
/// terminal and reading a console log.
#[derive(Debug, Clone, PartialEq)]
pub enum TtlEvent {
    /// About to probe every test domain with this hop count.
    Trying(u8),
    /// The daemon said something and left the queue empty. The text is what
    /// winws printed, which is the only explanation for a launch that died.
    Refused(u8, String),
    /// One domain's verdict under the current hop count.
    Probed { domain: String, ok: bool },
    /// Every domain came through at this hop count — this is the answer.
    Found(u8),
}

/// Sweep TTL from 1 to 20, running winws with a fixed TTL each time and
/// probing real domains. Returns the first (minimum) working TTL.
///
/// `on_event` is asked after every event; returning `false` stops the sweep
/// with the network put back the way it was.
pub fn autopick_ttl(
    strategy_file: &str,
    interface: &str,
    backend: &dyn FirewallBackend,
    on_event: &mut dyn FnMut(TtlEvent) -> bool,
) -> Result<u8, String> {
    if crate::run::queue_in_use() {
        return Err(rust_i18n::t!("ttl_err_running").into_owned());
    }
    let domains = get_test_domains();
    if domains.is_empty() {
        return Err(rust_i18n::t!("ttl_err_none").into_owned());
    }
    let capture = crate::paths::nfqws_output_log();

    for ttl in TTL_MIN..=TTL_MAX {
        if !on_event(TtlEvent::Trying(ttl)) {
            crate::run::stop_quiet(backend);
            return Err(rust_i18n::t!("ttl_err_cancelled").into_owned());
        }

        let req = crate::plan::RunRequest::new(strategy_file, interface, false, false).with_ttl(ttl);
        if let Err(e) = crate::run::run_quiet(&req, backend, &capture) {
            let line = format!("{}: {}", strategy_file, e);
            if !on_event(TtlEvent::Refused(ttl, line)) {
                crate::run::stop_quiet(backend);
                return Err(rust_i18n::t!("ttl_err_cancelled").into_owned());
            }
            continue;
        }

        if !wait_for_nfqws(Duration::from_secs(3)) {
            // Whatever winws printed on its way out is the answer to why this
            // hop count was skipped, so it is reported rather than dropped.
            let said = crate::run::read_launch_output(&capture);
            crate::run::stop_quiet(backend);
            if !on_event(TtlEvent::Refused(ttl, said)) {
                return Err(rust_i18n::t!("ttl_err_cancelled").into_owned());
            }
            continue;
        }

        let mut all_ok = true;
        for domain in &domains {
            let ok = curl_tls_ok(domain);
            if !ok {
                all_ok = false;
            }
            if !on_event(TtlEvent::Probed {
                domain: domain.clone(),
                ok,
            }) {
                crate::run::stop_quiet(backend);
                return Err(rust_i18n::t!("ttl_err_cancelled").into_owned());
            }
        }

        crate::run::stop_quiet(backend);

        if all_ok {
            on_event(TtlEvent::Found(ttl));
            return Ok(ttl);
        }
    }

    Err(rust_i18n::t!("ttl_err_none").into_owned())
}
