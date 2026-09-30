use std::net::SocketAddr;
use std::time::Duration;

use super::dns::resolve_domain;
use super::quic;

fn null_device() -> &'static str {
    if cfg!(target_os = "windows") {
        "NUL"
    } else {
        "/dev/null"
    }
}

fn http_ok(code: &str) -> bool {
    !code.is_empty() && code != "000"
}

pub fn curl_test(url: &str, extra_args: &[&str], num_requests: usize, ok: impl Fn(&str) -> bool) -> bool {
    if num_requests == 0 || super::cancel::is_cancelled() {
        return false;
    }
    for _ in 0..num_requests {
        if super::cancel::is_cancelled() {
            return false;
        }
        let out = std::process::Command::new("curl")
            .arg("-s")
            .arg("-k")
            .args(extra_args)
            .args(["--connect-timeout", "4", "--max-time", "4", "-o", null_device(), "-w"])
            .arg("%{http_code}")
            .arg(url)
            .output();
        if super::cancel::is_cancelled() {
            return false;
        }
        let code = out
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if !ok(&code) {
            return false;
        }
    }
    true
}

pub fn test_tls(domain: &str, tls_flag: &str, num_requests: usize) -> bool {
    curl_test(&format!("https://{}", domain), &[tls_flag], num_requests, http_ok)
}

pub fn test_quic(domain: &str, num_requests: usize) -> bool {
    let ips = resolve_domain(domain);
    if ips.is_empty() {
        return false;
    }
    ips.iter().take(2).any(|&ip| {
        let addr = SocketAddr::new(ip, 443);
        quic::probe_quic(addr, domain, num_requests.max(1), Duration::from_secs(2))
    })
}

pub fn test_http(domain: &str, num_requests: usize) -> bool {
    curl_test(&format!("http://{}", domain), &[], num_requests, http_ok)
}
