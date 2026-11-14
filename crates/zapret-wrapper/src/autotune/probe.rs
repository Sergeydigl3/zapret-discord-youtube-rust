//! HTTP and TLS probes.
//!
//! The sweep asks the same question a browser does: "does this site answer?"
//! It does that with `ureq` over the same TLS stack the rest of the program
//! uses, rather than by spawning `curl`. That keeps the check inside Rust:
//! there is no external program to install, no output to parse, and
//! cancellation only has to wait for the current request instead of killing a
//! child process.
//!
//! TLS 1.2 and TLS 1.3 are tested separately, because a strategy may lift one
//! and not the other. Each version gets its own client configuration with only
//! that protocol version enabled.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use ureq::rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use ureq::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use ureq::rustls::{
    version, ClientConfig, DigitallySignedStruct, Error as TlsError, RootCertStore, SignatureScheme,
    SupportedProtocolVersion,
};

use super::dns::resolve_domain;
use super::quic;

const TIMEOUT: Duration = Duration::from_secs(4);

/// Certificate verification off: the probe asks whether the DPI lets the
/// connection through, not whether the site's certificate is valid. Some test
/// domains serve a wildcard certificate that does not match the bare hostname,
/// and a filtering page may present its own certificate; both still mean the
/// connection works.
#[derive(Debug)]
struct AcceptAnyCertificate;

impl ServerCertVerifier for AcceptAnyCertificate {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, TlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, TlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        ureq::rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn agent_with(versions: &[&'static SupportedProtocolVersion]) -> ureq::Agent {
    let mut tls = ClientConfig::builder_with_protocol_versions(versions)
        .with_root_certificates(RootCertStore::empty())
        .with_no_client_auth();
    tls.dangerous().set_certificate_verifier(Arc::new(AcceptAnyCertificate));

    ureq::AgentBuilder::new()
        .timeout(TIMEOUT)
        // A new connection for every request: a reused one would report the
        // first success again without asking the network anything.
        .max_idle_connections(0)
        .redirects(0)
        .tls_config(Arc::new(tls))
        .build()
}

fn plain_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(TIMEOUT)
        .max_idle_connections(0)
        .redirects(0)
        .build()
}

/// Ask one URL `num_requests` times.
///
/// Any HTTP response counts as success: the probe is testing whether the
/// request gets through the DPI, not whether the page exists. Only a transport
/// error (DNS, connect, TLS) means the site is blocked.
fn get_ok(agent: &ureq::Agent, url: &str, num_requests: usize) -> bool {
    if num_requests == 0 || super::cancel::is_cancelled() {
        return false;
    }
    for _ in 0..num_requests {
        if super::cancel::is_cancelled() {
            return false;
        }
        match agent.get(url).call() {
            Ok(_) => {}
            Err(ureq::Error::Status(_, _)) => {}
            Err(_) => return false,
        }
    }
    true
}

pub fn test_http(domain: &str, num_requests: usize) -> bool {
    get_ok(&plain_agent(), &format!("http://{domain}/"), num_requests)
}

pub fn test_tls12(domain: &str, num_requests: usize) -> bool {
    get_ok(
        &agent_with(&[&version::TLS12]),
        &format!("https://{domain}/"),
        num_requests,
    )
}

pub fn test_tls13(domain: &str, num_requests: usize) -> bool {
    get_ok(
        &agent_with(&[&version::TLS13]),
        &format!("https://{domain}/"),
        num_requests,
    )
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
