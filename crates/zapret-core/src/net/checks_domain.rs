use std::io::{ErrorKind, Read, Write};
use std::net::SocketAddr;
use std::time::Duration;

use crate::autotune::types::{status_char, AutotuneConfig, CheckStatus, DomainCheckResult};

use super::checks_network::try_tcp_connect_domain;
use super::dns::resolve_domain;
use super::probe::test_tls;
use super::quic;

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

pub fn check_domain_http(domain: &str, num_req: usize) -> (CheckStatus, usize) {
    let mut success = 0;
    for _ in 0..num_req {
        match try_tcp_connect_domain(domain, 80) {
            Ok(mut stream) => {
                let req = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", domain);
                let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                if stream.write(req.as_bytes()).is_ok() {
                    let mut buf = [0u8; 16];
                    match stream.read(&mut buf) {
                        Ok(n) if n > 0 => success += 1,
                        _ => {
                            return (CheckStatus::Fail, success);
                        }
                    }
                } else {
                    return (CheckStatus::Fail, success);
                }
            }
            Err(_) => {
                return (CheckStatus::Fail, success);
            }
        }
    }
    let status = if success > 0 {
        CheckStatus::Pass
    } else {
        CheckStatus::Fail
    };
    (status, success)
}

pub fn check_domain_tls(domain: &str, num_req: usize) -> CheckStatus {
    for _ in 0..num_req {
        match try_tcp_connect_domain(domain, 443) {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut buf = [0u8; 1];
                let _ = stream.read(&mut buf);
            }
            Err(_) => {
                return CheckStatus::Fail;
            }
        }
    }
    CheckStatus::Pass
}

pub fn check_domain_quic(domain: &str, num_req: usize) -> (CheckStatus, usize) {
    let ips = resolve_domain(domain);
    if ips.is_empty() {
        return (CheckStatus::Skip, 0);
    }
    let mut success = 0;
    for &ip in ips.iter().take(2) {
        let addr = SocketAddr::new(ip, 443);
        if quic::probe_quic(addr, domain, num_req.max(1), Duration::from_secs(2)) {
            success += 1;
            break;
        }
    }
    let status = if success > 0 {
        CheckStatus::Pass
    } else {
        CheckStatus::Fail
    };
    (status, success)
}

pub fn check_domain(config: &AutotuneConfig, domain: &str) -> DomainCheckResult {
    if super::cancel::is_cancelled() {
        return crate::autotune::orchestrator::domain_check_error();
    }
    let alive = check_domain_alive(domain);
    let detail;

    let (http, tls12, tls13, quic, http_count, quic_count) = if alive == CheckStatus::Pass {
        let mut parts = Vec::new();

        let (http, hc) = if config.check_http {
            let (s, c) = check_domain_http(domain, config.num_requests);
            parts.push(format!("HTTP:{} ({}/{})", status_char(&s), c, config.num_requests));
            (s, c)
        } else {
            (CheckStatus::Skip, 0)
        };

        let tls = if config.check_tls12 || config.check_tls13 {
            check_domain_tls(domain, config.num_requests)
        } else {
            CheckStatus::Skip
        };

        let tls12 = if config.check_tls12 {
            parts.push(format!("TLS1.2:{}", status_char(&tls)));
            tls.clone()
        } else {
            CheckStatus::Skip
        };

        let tls13 = if config.check_tls13 {
            parts.push(format!("TLS1.3:{}", status_char(&tls)));
            tls.clone()
        } else {
            CheckStatus::Skip
        };

        let (quic, qc) = if config.check_quic {
            let (s, c) = check_domain_quic(domain, config.num_requests);
            parts.push(format!("QUIC:{} ({}/{})", status_char(&s), c, config.num_requests));
            (s, c)
        } else {
            (CheckStatus::Skip, 0)
        };

        detail = parts.join(" ");
        (http, tls12, tls13, quic, hc, qc)
    } else if alive == CheckStatus::Skip {
        detail = "Domain unreachable (skipped)".to_string();
        (
            CheckStatus::Skip,
            CheckStatus::Skip,
            CheckStatus::Skip,
            CheckStatus::Skip,
            0,
            0,
        )
    } else {
        detail = "Domain appears blocked (alive check failed)".to_string();
        (
            CheckStatus::Skip,
            CheckStatus::Skip,
            CheckStatus::Skip,
            CheckStatus::Skip,
            0,
            0,
        )
    };

    // Baseline TLS 1.3 test: real TLS handshake + HTTP request
    let baseline_pass = if alive == CheckStatus::Pass {
        test_tls(domain, "--tlsv1.3", config.num_requests)
    } else {
        alive == CheckStatus::Skip
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
        http_count,
        quic_count,
    }
}
