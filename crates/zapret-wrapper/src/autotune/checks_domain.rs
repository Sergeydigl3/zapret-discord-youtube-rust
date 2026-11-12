use std::io::{self, ErrorKind, Read};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use super::probe::{test_http, test_quic, test_tls12, test_tls13};
use super::types::{status_char, AutotuneConfig, CheckStatus, DomainCheckResult};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(4);

/// Connect to a domain, trying every address DNS returned.
fn try_tcp_connect_domain(domain: &str, port: u16) -> Result<TcpStream, io::Error> {
    let addrs = (domain, port).to_socket_addrs()?;
    let mut last_err = io::Error::other("no addresses");
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT) {
            Ok(stream) => return Ok(stream),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

/// Result to report when a per-domain check thread died before producing one.
///
/// Lives next to [`check_domain`] because this is the module that produces
/// `DomainCheckResult`; the orchestrator only has to ask for it.
pub fn domain_check_error() -> DomainCheckResult {
    DomainCheckResult {
        domain: String::new(),
        alive: CheckStatus::Error,
        http: CheckStatus::Error,
        tls12: CheckStatus::Error,
        tls13: CheckStatus::Error,
        quic: CheckStatus::Error,
        baseline_pass: false,
        detail: "check thread panicked".to_string(),
    }
}

pub fn check_domain_alive(domain: &str) -> CheckStatus {
    match try_tcp_connect_domain(domain, 443) {
        Ok(mut stream) => {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
            let mut buf = [0u8; 1];
            match stream.read(&mut buf) {
                Ok(_) => CheckStatus::Pass,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => CheckStatus::Pass,
                Err(ref e) if e.kind() == ErrorKind::ConnectionReset => CheckStatus::Fail,
                Err(_) => CheckStatus::Pass,
            }
        }
        Err(ref e) if e.kind() == ErrorKind::ConnectionReset => CheckStatus::Fail,
        Err(ref e) if e.kind() == ErrorKind::TimedOut => CheckStatus::Fail,
        Err(ref e) if e.kind() == ErrorKind::AddrNotAvailable => CheckStatus::Error,
        Err(_) => CheckStatus::Skip,
    }
}

fn status(ok: bool) -> CheckStatus {
    if ok {
        CheckStatus::Pass
    } else {
        CheckStatus::Fail
    }
}

pub fn check_domain(config: &AutotuneConfig, domain: &str) -> DomainCheckResult {
    if super::cancel::is_cancelled() {
        return domain_check_error();
    }
    let alive = check_domain_alive(domain);
    let n = config.num_requests;

    // TLS 1.3 is the baseline: a browser needs it for these sites, so it also
    // decides which domains count as blocked before a strategy is tested.
    let baseline_ok = alive == CheckStatus::Pass && test_tls13(domain, n);
    let baseline_pass = if alive == CheckStatus::Pass {
        baseline_ok
    } else {
        alive == CheckStatus::Skip
    };

    let (http, tls12, tls13, quic) = if alive == CheckStatus::Pass {
        (
            if config.check_http {
                status(test_http(domain, n))
            } else {
                CheckStatus::Skip
            },
            if config.check_tls12 {
                status(test_tls12(domain, n))
            } else {
                CheckStatus::Skip
            },
            if config.check_tls13 {
                status(baseline_ok)
            } else {
                CheckStatus::Skip
            },
            if config.check_quic {
                status(test_quic(domain, n))
            } else {
                CheckStatus::Skip
            },
        )
    } else {
        (
            CheckStatus::Skip,
            CheckStatus::Skip,
            CheckStatus::Skip,
            CheckStatus::Skip,
        )
    };

    let detail = if alive == CheckStatus::Pass {
        format!(
            "HTTP:{} TLS1.2:{} TLS1.3:{} QUIC:{}",
            status_char(&http),
            status_char(&tls12),
            status_char(&tls13),
            status_char(&quic)
        )
    } else if alive == CheckStatus::Skip {
        "Domain unreachable (skipped)".to_string()
    } else {
        "Domain appears blocked (alive check failed)".to_string()
    };

    DomainCheckResult {
        domain: domain.to_string(),
        alive,
        http,
        tls12,
        tls13,
        quic,
        baseline_pass,
        detail,
    }
}
