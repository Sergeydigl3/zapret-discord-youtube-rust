use std::io::{self, ErrorKind, Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs, UdpSocket};
use std::path::Path;
use std::time::Duration;

use crate::firewalls::FirewallBackend;

const TIMEOUT: Duration = Duration::from_secs(4);
const CUSTOM_DOMAINS_FILE: &str = "autotune_custom.txt";

#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Pass,
    Fail,
    Skip,
    Error,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub status: CheckStatus,
    #[allow(dead_code)]
    pub detail: String,
}

impl CheckResult {
    fn pass(detail: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Pass,
            detail: detail.into(),
        }
    }
    fn fail(detail: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Fail,
            detail: detail.into(),
        }
    }
    fn skip(detail: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Skip,
            detail: detail.into(),
        }
    }
}

mod quic;
pub mod presets;
pub use presets::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockCheckType {
    DnsSpoof,
    TcpRst,
    SniBlock,
    SiberianBlock,
    QuicBlock,
    CidrWhitelist,
}

impl BlockCheckType {
    pub fn all() -> &'static [Self] {
        &[
            Self::DnsSpoof,
            Self::TcpRst,
            Self::SniBlock,
            Self::SiberianBlock,
            Self::QuicBlock,
            Self::CidrWhitelist,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::DnsSpoof => "DNS",
            Self::TcpRst => "TCP RST",
            Self::SniBlock => "SNI",
            Self::SiberianBlock => "SIBERIAN",
            Self::QuicBlock => "QUIC",
            Self::CidrWhitelist => "CIDR",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockChecks {
    pub enabled: Vec<bool>, // same order as BlockCheckType::all()
}

impl BlockChecks {
    pub fn all_enabled() -> Self {
        Self { enabled: vec![true; 6] }
    }

    pub fn get(&self, idx: usize) -> bool {
        self.enabled.get(idx).copied().unwrap_or(false)
    }

    pub fn set(&mut self, idx: usize, val: bool) {
        if let Some(e) = self.enabled.get_mut(idx) {
            *e = val;
        }
    }

    pub fn count_enabled(&self) -> usize {
        self.enabled.iter().filter(|&&e| e).count()
    }
}

#[derive(Debug, Clone)]
pub struct AutotuneConfig {
    pub preset_indices: Vec<usize>,
    pub num_requests: usize,
    pub check_http: bool,
    pub check_tls12: bool,
    pub check_tls13: bool,
    pub check_quic: bool,
    pub strategy_indices: Vec<usize>,
    pub block_checks: BlockChecks,
}

impl Default for AutotuneConfig {
    fn default() -> Self {
        Self {
            preset_indices: vec![0],
            num_requests: 3,
            check_http: true,
            check_tls12: true,
            check_tls13: true,
            check_quic: true,
            strategy_indices: Vec::new(),
            block_checks: BlockChecks::all_enabled(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomainCheckResult {
    pub domain: String,
    pub alive: CheckStatus,
    pub http: CheckStatus,
    pub tls12: CheckStatus,
    pub tls13: CheckStatus,
    pub quic: CheckStatus,
    pub baseline_pass: bool,
    #[allow(dead_code)]
    pub detail: String,
    pub http_count: usize,
    pub quic_count: usize,
}

#[derive(Debug, Clone)]
pub struct StrategyCheckResult {
    pub strategy_name: String,
    pub domains_pass: Vec<String>,
    pub domains_fail: Vec<String>,
    pub works: bool,
    pub protocols_working: Vec<String>,
    pub domain_checks: Vec<DomainProtocolCheck>,
}

#[derive(Debug, Clone)]
pub struct DomainProtocolCheck {
    pub domain: String,
    pub http: bool,
    pub tls12: bool,
    pub tls13: bool,
    pub quic: bool,
}

impl StrategyCheckResult {
    pub fn total(&self) -> usize {
        self.domains_pass.len() + self.domains_fail.len()
    }
    pub fn score(&self) -> usize {
        self.domains_pass.len()
    }
    fn failed(name: &str, blocked_domains: &[String]) -> Self {
        Self {
            strategy_name: name.to_string(),
            domains_pass: Vec::new(),
            domains_fail: blocked_domains.to_vec(),
            works: false,
            protocols_working: Vec::new(),
            domain_checks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PresetResult {
    pub preset_name: String,
    pub domain_checks: Vec<DomainCheckResult>,
    pub strategy_results: Vec<StrategyCheckResult>,
}

#[derive(Debug, Clone)]
pub struct AutotuneResults {
    pub block_results: Vec<CheckResult>, // DNS, TCP RST, SNI, SIBERIAN, QUIC, CIDR
    pub preset_results: Vec<PresetResult>,
    pub common_strategies: Vec<String>, // strategies that work across ALL selected presets
}

#[allow(dead_code)]
impl AutotuneResults {
    pub fn dns_spoof(&self) -> &CheckResult {
        &self.block_results[0]
    }
    pub fn tcp_rst(&self) -> &CheckResult {
        &self.block_results[1]
    }
    pub fn sni_block(&self) -> &CheckResult {
        &self.block_results[2]
    }
    pub fn siberian_block(&self) -> &CheckResult {
        &self.block_results[3]
    }
    pub fn quic_block(&self) -> &CheckResult {
        &self.block_results[4]
    }
    pub fn cidr_whitelist(&self) -> &CheckResult {
        &self.block_results[5]
    }
}

const RESULTS_FILE: &str = "autotune_results.txt";

pub fn status_str_file(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pass => "OK",
        CheckStatus::Fail => "BLOCKED",
        CheckStatus::Skip => "SKIP",
        CheckStatus::Error => "ERROR",
    }
}

pub fn save_results_file(results: &AutotuneResults) {
    use std::io::Write;
    let path = crate::config::get_cache_dir().join(RESULTS_FILE);
    let mut file = match std::fs::File::create(&path) {
        Ok(f) => f,
        Err(e) => {
            println!("  [save_results_file] Failed to create {}: {}", path.display(), e);
            return;
        }
    };
    println!(
        "  {}",
        rust_i18n::t!("autotune_saving_results").replace("{}", &path.display().to_string())
    );

    let check_names = ["DNS", "TCP RST", "SNI", "SIBERIAN", "QUIC", "CIDR"];
    let _ = writeln!(file, "--- {} ---", rust_i18n::t!("autotune_net_results"));
    for (name, check) in check_names.iter().zip(&results.block_results) {
        let _ = writeln!(file, "  {}: {}", name, status_str_file(&check.status));
    }
    let _ = writeln!(file);

    for pr in &results.preset_results {
        let _ = writeln!(
            file,
            "--- {} [{}] ---",
            rust_i18n::t!("autotune_domain_results"),
            pr.preset_name
        );
        for dc in &pr.domain_checks {
            let _ = writeln!(
                file,
                "  {}: alive={} HTTP:{}({}) TLS1.2:{} TLS1.3:{} QUIC:{}({}) baseline={}",
                dc.domain,
                status_str_file(&dc.alive),
                status_str_file(&dc.http),
                dc.http_count,
                status_str_file(&dc.tls12),
                status_str_file(&dc.tls13),
                status_str_file(&dc.quic),
                dc.quic_count,
                status_str_file(if dc.baseline_pass {
                    &CheckStatus::Pass
                } else {
                    &CheckStatus::Fail
                }),
            );
        }
        let _ = writeln!(file);
    }

    if !results.preset_results.is_empty() {
        let _ = writeln!(file, "--- {} ---", rust_i18n::t!("autotune_strat_results"));
        for pr in &results.preset_results {
            let _ = writeln!(file, "  [{}]", pr.preset_name);
            for sr in &pr.strategy_results {
                let s = if sr.works { "WORKS" } else { "FAILS" };
                let protos = if sr.protocols_working.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", sr.protocols_working.join(", "))
                };
                let _ = writeln!(
                    file,
                    "    {}: {} ({}/{}){}",
                    sr.strategy_name,
                    s,
                    sr.score(),
                    sr.total(),
                    protos
                );
                for dc in &sr.domain_checks {
                    let _ = writeln!(
                        file,
                        "      {} HTTP:{} T12:{} T13:{} Q:{}",
                        dc.domain,
                        if dc.http { "✅" } else { "❌" },
                        if dc.tls12 { "✅" } else { "❌" },
                        if dc.tls13 { "✅" } else { "❌" },
                        if dc.quic { "✅" } else { "❌" },
                    );
                }
            }
            let _ = writeln!(file);
        }

        if !results.common_strategies.is_empty() {
            let _ = writeln!(
                file,
                "--- {} ({}) ---",
                rust_i18n::t!("autotune_common_strats"),
                results.common_strategies.len()
            );
            for name in &results.common_strategies {
                let _ = writeln!(file, "  ✅ {}", name);
            }
            let _ = writeln!(file);
        }
    }
}

pub fn load_results_file() -> Option<String> {
    let path = crate::config::get_cache_dir().join(RESULTS_FILE);
    if path.exists() {
        std::fs::read_to_string(&path).ok()
    } else {
        None
    }
}

const TEST_DOMAINS: &[&str] = &["discord.com", "youtube.com", "cdn.discordapp.com"];

const CLEAN_DOMAIN: &str = "google.com";

const KNOWN_IPS: &[(&str, &[&str])] = &[
    (
        "discord.com",
        &["162.159.128.233", "162.159.135.232", "162.159.136.232"],
    ),
    ("youtube.com", &["142.250.150.46", "216.58.209.46", "142.250.185.78"]),
    ("google.com", &["142.250.185.78", "216.58.215.14"]),
];

pub fn preset_domains_file_path(preset_idx: usize) -> std::path::PathBuf {
    let name = PRESET_FILES.get(preset_idx).copied().unwrap_or(CUSTOM_DOMAINS_FILE);
    crate::config::get_cache_dir().join(name)
}

/// Read a domain list file. One domain per line, lines starting with `#` are
/// treated as comments and skipped.
pub fn load_domain_file(path: &std::path::Path) -> Vec<String> {
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Domains for a preset come entirely from its text file. If the file is
/// missing or empty, the built-in defaults are used as a fallback.
pub fn get_domains_for_preset(preset_idx: usize) -> Vec<String> {
    if preset_idx >= PRESETS.len() {
        return Vec::new();
    }
    let file_domains = load_domain_file(&preset_domains_file_path(preset_idx));
    if !file_domains.is_empty() {
        return file_domains;
    }
    PRESETS[preset_idx].domains.iter().map(|s| s.to_string()).collect()
}

/// Create/refresh the per-preset domain list files (and the TTL list) with the
/// full built-in domain list so the user can add/remove domains freely.
pub fn ensure_domain_files() -> Result<(), String> {
    for (idx, preset) in PRESETS.iter().enumerate() {
        let path = preset_domains_file_path(idx);
        let header = rust_i18n::t!("domain_file_header_full").replace("{}", preset.name);
        ensure_domain_file(&path, &header, preset.domains)?;
    }
    crate::ttl::ensure_ttl_file()
}

fn ensure_domain_file(path: &std::path::Path, header: &str, defaults: &[&str]) -> Result<(), String> {
    if path.exists() && !load_domain_file(path).is_empty() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Cannot create directory '{}': {}", parent.display(), e))?;
    }
    let mut content = format!("# {}\n", header);
    for d in defaults {
        content.push_str(d);
        content.push('\n');
    }
    std::fs::write(path, content).map_err(|e| format!("Cannot write '{}': {}", path.display(), e))
}

fn resolve_domain(domain: &str) -> Vec<IpAddr> {
    (domain, 0)
        .to_socket_addrs()
        .map(|addrs| addrs.map(|a| a.ip()).collect())
        .unwrap_or_default()
}

fn is_sinkhole(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4 == Ipv4Addr::UNSPECIFIED || v4.is_loopback() || v4.is_private() || v4 == Ipv4Addr::new(0, 0, 0, 0)
        }
        IpAddr::V6(_) => false,
    }
}

fn try_tcp_connect(addr: &str, port: u16) -> Result<TcpStream, io::Error> {
    let socket_addr: SocketAddr = format!("{}:{}", addr, port)
        .parse()
        .map_err(|_| io::Error::new(ErrorKind::InvalidInput, "invalid address"))?;
    TcpStream::connect_timeout(&socket_addr, TIMEOUT)
}

fn try_tcp_connect_domain(domain: &str, port: u16) -> Result<TcpStream, io::Error> {
    let addrs = (domain, port).to_socket_addrs()?;
    let mut last_err = io::Error::other("no addresses");
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, TIMEOUT) {
            Ok(stream) => return Ok(stream),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

pub fn check_dns_spoof() -> CheckResult {
    let mut results: Vec<String> = Vec::new();

    for &domain in TEST_DOMAINS {
        let sys_ips = resolve_domain(domain);
        if sys_ips.is_empty() {
            results.push(format!("{}: not resolved", domain));
            continue;
        }

        let suspect: Vec<IpAddr> = sys_ips.iter().copied().filter(|&ip| is_sinkhole(ip)).collect();
        if !suspect.is_empty() {
            return CheckResult::fail(format!("{} resolved to sinkhole IPs: {:?}", domain, suspect));
        }

        if let Some(&(_, known_ips)) = KNOWN_IPS.iter().find(|(d, _)| *d == domain) {
            let known_addrs: Vec<IpAddr> = known_ips.iter().filter_map(|s| s.parse().ok()).collect();
            let any_match = sys_ips.iter().any(|ip| known_addrs.contains(ip));
            if !any_match {
                results.push(format!(
                    "{} resolved to {:?} (unexpected vs known {:?})",
                    domain, sys_ips, known_ips
                ));
            } else {
                results.push(format!("{} OK", domain));
            }
        }
    }

    let clean_ips = resolve_domain(CLEAN_DOMAIN);
    if clean_ips.is_empty() {
        return CheckResult::skip("google.com: not resolved (possible Internet issue)");
    }

    if results.is_empty() || results.iter().all(|r| r.contains("OK")) {
        CheckResult::pass("DNS responses look legitimate")
    } else {
        let fails: Vec<&str> = results
            .iter()
            .filter(|r| !r.contains("OK"))
            .map(|s| s.as_str())
            .collect();
        CheckResult::fail(format!("Possible DNS spoofing: {}", fails.join("; ")))
    }
}

pub fn check_tcp_rst() -> CheckResult {
    let mut domain_success = 0;
    let mut domain_fail_rst = 0;
    let mut details: Vec<String> = Vec::new();

    for &domain in TEST_DOMAINS {
        match try_tcp_connect_domain(domain, 443) {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut buf = [0u8; 1];
                match stream.read_exact(&mut buf) {
                    Ok(_) => {
                        domain_success += 1;
                        details.push(format!("{}: connected", domain));
                    }
                    Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                        domain_fail_rst += 1;
                        details.push(format!("{}: RST after connect", domain));
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                        domain_success += 1;
                        details.push(format!("{}: connected (idle)", domain));
                    }
                    Err(e) => {
                        details.push(format!("{}: {} after connect", domain, e));
                    }
                }
            }
            Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                domain_fail_rst += 1;
                details.push(format!("{}: RST on connect", domain));
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                domain_fail_rst += 1;
                details.push(format!("{}: timeout (possible DPI drop)", domain));
            }
            Err(e) => {
                details.push(format!("{}: {}", domain, e));
            }
        }
    }

    if try_tcp_connect_domain(CLEAN_DOMAIN, 443).is_err() {
        return CheckResult::skip("Internet connectivity issue (google.com unreachable)");
    }

    if domain_success > 0 && domain_fail_rst == 0 {
        CheckResult::pass("TCP connections successful, no RST detected")
    } else if domain_fail_rst > 0 {
        CheckResult::fail(format!(
            "TCP RST/blocking detected ({}/{} domains affected): {}",
            domain_fail_rst,
            TEST_DOMAINS.len(),
            details.join("; ")
        ))
    } else {
        CheckResult::skip(format!("Mixed results: {}", details.join("; ")))
    }
}

pub fn check_sni_block() -> CheckResult {
    let mut ip_ok = 0;
    let mut domain_fail = 0;
    let mut ip_fail = 0;
    let mut details: Vec<String> = Vec::new();

    for &(domain, ips) in KNOWN_IPS {
        if domain == CLEAN_DOMAIN {
            continue;
        }

        let domain_ok = try_tcp_connect_domain(domain, 443).is_ok();
        if !domain_ok {
            domain_fail += 1;
        }

        for &ip in ips {
            match try_tcp_connect(ip, 443) {
                Ok(mut stream) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
                    let mut buf = [0u8; 1];
                    match stream.read(&mut buf) {
                        Ok(_) => {
                            if !domain_ok {
                                details.push(format!("{} (IP {}) works, domain fails -> SNI block", domain, ip));
                            }
                            ip_ok += 1;
                        }
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                            if !domain_ok {
                                details.push(format!("{} (IP {}) works, domain fails -> SNI block", domain, ip));
                            }
                            ip_ok += 1;
                        }
                        Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                            details.push(format!("{} (IP {}): RST", domain, ip));
                            ip_fail += 1;
                        }
                        Err(_) => {
                            ip_ok += 1;
                        }
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                    details.push(format!("{} (IP {}): RST on connect", domain, ip));
                    ip_fail += 1;
                }
                Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                    details.push(format!("{} (IP {}): timeout", domain, ip));
                    ip_fail += 1;
                }
                Err(_) => {}
            }
        }
    }

    if try_tcp_connect_domain(CLEAN_DOMAIN, 443).is_err() {
        return CheckResult::skip("Internet connectivity issue");
    }

    if domain_fail > 0 && ip_ok > ip_fail {
        CheckResult::fail(format!(
            "SNI blocking detected (domains fail but IPs work): {}",
            details.join("; ")
        ))
    } else if domain_fail == 0 {
        CheckResult::pass("No SNI blocking detected")
    } else {
        CheckResult::skip(format!("Inconclusive: {}", details.join("; ")))
    }
}

pub fn check_siberian_block() -> CheckResult {
    const MAX_CONCURRENT: usize = 15;
    const EXTRA_CONNECTIONS: usize = 10;

    let test_ips: Vec<&str> = KNOWN_IPS[0].1.to_vec();

    let clean_ok = try_tcp_connect_domain(CLEAN_DOMAIN, 443).is_ok();

    if !clean_ok {
        return CheckResult::skip("Internet connectivity issue");
    }

    let mut handles: Vec<std::thread::JoinHandle<Result<TcpStream, io::Error>>> = Vec::new();

    for _ in 0..MAX_CONCURRENT {
        for &ip in &test_ips {
            let handle = std::thread::spawn(move || try_tcp_connect(ip, 443));
            handles.push(handle);
        }
    }

    let mut alive = 0;
    let mut failed = 0;

    for handle in handles {
        match handle.join() {
            Ok(Ok(_)) => alive += 1,
            Ok(Err(_)) => failed += 1,
            Err(_) => failed += 1,
        }
    }

    let mut extra_failed = 0;

    for _ in 0..EXTRA_CONNECTIONS {
        let ok = test_ips.iter().any(|&ip| try_tcp_connect(ip, 443).is_ok());
        if ok {
            alive += 1;
        } else {
            extra_failed += 1;
            failed += 1;
        }
    }

    let total_attempted = alive + failed;
    let pass_ratio = if total_attempted > 0 {
        alive as f64 / total_attempted as f64
    } else {
        1.0
    };

    if extra_failed == 0 && pass_ratio > 0.95 {
        CheckResult::pass("No Siberian block detected (100% success after 15+ concurrent)")
    } else if extra_failed > 0 {
        CheckResult::fail(format!(
            "Possible Siberian block: {} of {} extra connections failed",
            extra_failed, EXTRA_CONNECTIONS
        ))
    } else if pass_ratio < 0.8 {
        CheckResult::fail(format!(
            "High failure rate: {}/{} connections failed",
            failed, total_attempted
        ))
    } else {
        CheckResult::skip(format!(
            "Mixed results: {}/{} alive, {}/{} extra failed",
            alive, total_attempted, extra_failed, EXTRA_CONNECTIONS
        ))
    }
}

pub fn check_quic_block() -> CheckResult {
    // First verify general UDP connectivity so a broken link isn't reported
    // as a QUIC block.
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(sock) => {
            let clean_ip: IpAddr = "8.8.8.8".parse().unwrap();
            if sock.connect((clean_ip, 53)).is_err() {
                return CheckResult::skip("Internet connectivity issue (cannot reach 8.8.8.8:53 UDP)");
            }
        }
        Err(_) => {
            return CheckResult::skip("Cannot create UDP socket");
        }
    }

    let mut details: Vec<String> = Vec::new();
    let mut quic_ok = 0;

    for &(domain, ips) in KNOWN_IPS {
        if domain == CLEAN_DOMAIN {
            continue;
        }
        for &ip_str in ips {
            let ip: IpAddr = match ip_str.parse() {
                Ok(ip) => ip,
                Err(_) => continue,
            };
            let addr = SocketAddr::new(ip, 443);
            match UdpSocket::bind("0.0.0.0:0") {
                Ok(sock) => {
                    if sock.connect(addr).is_err() {
                        details.push(format!("{}: UDP connect failed", ip_str));
                        continue;
                    }
                    if sock.set_read_timeout(Some(Duration::from_secs(2))).is_err() {
                        continue;
                    }
                    match quic::send_probe(&sock, domain) {
                        quic::ProbeOutcome::Reply => {
                            details.push(format!("{}: QUIC response", ip_str));
                            quic_ok += 1;
                        }
                        quic::ProbeOutcome::NoReply => {
                            details.push(format!("{}: QUIC sent, no response (possible QUIC block)", ip_str));
                        }
                        quic::ProbeOutcome::Error => {
                            details.push(format!("{}: QUIC probe error", ip_str));
                        }
                    }
                }
                Err(e) => {
                    details.push(format!("{}: socket bind error: {}", ip_str, e));
                }
            }
        }
    }

    if quic_ok > 0 {
        CheckResult::pass("QUIC/UDP traffic appears unblocked")
    } else {
        let fail_details: Vec<&str> = details
            .iter()
            .filter(|d| d.contains("no response") || d.contains("error"))
            .map(|s| s.as_str())
            .collect();
        CheckResult::fail(format!("QUIC/UDP likely blocked: {}", fail_details.join("; ")))
    }
}

pub fn check_cidr_whitelist() -> CheckResult {
    let test_ips = [
        ("1.1.1.1", "Cloudflare DNS"),
        ("8.8.8.8", "Google DNS"),
        ("77.88.8.8", "Yandex DNS"),
        ("185.178.208.97", "discord CDN (MCF)"),
        ("104.16.0.0", "Cloudflare edge"),
    ];

    let mut reachable = 0;
    let mut blocked = 0;
    let mut details: Vec<String> = Vec::new();

    for &(ip, label) in &test_ips {
        match try_tcp_connect(ip, 443) {
            Ok(_) => {
                reachable += 1;
                details.push(format!("{} ({}) reachable", ip, label));
            }
            Err(ref e) if e.kind() == ErrorKind::ConnectionReset => {
                blocked += 1;
                details.push(format!("{} ({}) RST", ip, label));
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                blocked += 1;
                details.push(format!("{} ({}) timeout", ip, label));
            }
            Err(e) => {
                details.push(format!("{} ({}): {}", ip, label, e));
            }
        }
    }

    if try_tcp_connect_domain(CLEAN_DOMAIN, 443).is_err() {
        return CheckResult::skip("Internet connectivity issue");
    }

    if blocked == 0 {
        CheckResult::pass("No CIDR-based blocking detected across tested subnets")
    } else if reachable > 0 && blocked > 0 {
        let fail_parts: Vec<&str> = details
            .iter()
            .filter(|d| d.contains("RST") || d.contains("timeout"))
            .map(|s| s.as_str())
            .collect();
        CheckResult::fail(format!(
            "Possible selective CIDR blocking ({}/{} blocked): {}",
            blocked,
            test_ips.len(),
            fail_parts.join("; ")
        ))
    } else {
        CheckResult::fail("All tested IPs blocked: possible whitelist-only policy".to_string())
    }
}

fn check_domain_alive(domain: &str) -> CheckStatus {
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

fn check_domain_http(domain: &str, num_req: usize) -> (CheckStatus, usize) {
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

fn check_domain_tls(domain: &str, num_req: usize) -> CheckStatus {
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

fn check_domain_quic(domain: &str, num_req: usize) -> (CheckStatus, usize) {
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

fn http_ok(code: &str) -> bool {
    !code.is_empty() && code != "000"
}

fn null_device() -> &'static str {
    if cfg!(target_os = "windows") {
        "NUL"
    } else {
        "/dev/null"
    }
}

fn curl_test(url: &str, extra_args: &[&str], num_requests: usize, ok: impl Fn(&str) -> bool) -> bool {
    if num_requests == 0 {
        return true;
    }
    for _ in 0..num_requests {
        let out = std::process::Command::new("curl")
            .arg("-s")
            .arg("-k")
            .args(extra_args)
            .args(["--connect-timeout", "4", "--max-time", "4", "-o", null_device(), "-w"])
            .arg("%{http_code}")
            .arg(url)
            .output();
        let code = out
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        if !ok(&code) {
            return false;
        }
    }
    true
}

fn test_tls(domain: &str, tls_flag: &str, num_requests: usize) -> bool {
    curl_test(&format!("https://{}", domain), &[tls_flag], num_requests, http_ok)
}

fn test_quic(domain: &str, num_requests: usize) -> bool {
    let ips = resolve_domain(domain);
    if ips.is_empty() {
        return false;
    }
    ips.iter().take(2).any(|&ip| {
        let addr = SocketAddr::new(ip, 443);
        quic::probe_quic(addr, domain, num_requests.max(1), Duration::from_secs(2))
    })
}

fn test_http(domain: &str, num_requests: usize) -> bool {
    curl_test(&format!("http://{}", domain), &[], num_requests, http_ok)
}

pub fn check_domain(config: &AutotuneConfig, domain: &str) -> DomainCheckResult {
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

fn get_strategy_name(name: &str) -> String {
    name.trim_end_matches(".bat").to_string()
}

fn save_ipset() -> Option<String> {
    let path = crate::ipset::get_ipset_all_path();
    std::fs::read_to_string(&path).ok()
}

fn restore_ipset(content: &str) {
    let _ = std::fs::write(crate::ipset::get_ipset_all_path(), content);
}

fn set_ipset_any() {
    let _ = std::fs::write(crate::ipset::get_ipset_all_path(), "");
    println!("  {}", rust_i18n::t!("autotune_ipset_any"));
}

fn strategy_dir() -> String {
    std::env::var("REPO_DIR").unwrap_or_else(|_| {
        crate::config::get_cache_dir()
            .join("zapret-discord-youtube-linux")
            .to_string_lossy()
            .into_owned()
    })
}

fn load_strategy_files(indices: &[usize], all_strategies: &[String]) -> Vec<(String, String)> {
    if indices.is_empty() {
        return Vec::new();
    }
    let repo = strategy_dir();
    let mut result = Vec::new();
    for &idx in indices {
        if idx >= all_strategies.len() {
            continue;
        }
        let name = &all_strategies[idx];
        let path = Path::new(&repo).join("custom-strategies").join(name);
        let path = if path.exists() {
            path
        } else {
            Path::new(&repo).join(name)
        };
        if !path.exists() {
            continue;
        }
        result.push((get_strategy_name(name), path.to_string_lossy().to_string()));
    }
    result
}

fn status_char(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pass => "OK",
        CheckStatus::Fail => "BLOCKED",
        CheckStatus::Skip => "-",
        CheckStatus::Error => "ERR",
    }
}

fn count_protocol_steps(config: &AutotuneConfig) -> usize {
    config.check_http as usize + config.check_tls12 as usize + config.check_tls13 as usize + config.check_quic as usize
}

fn run_network_checks(block_checks: &BlockChecks) -> Vec<CheckResult> {
    let checks: [fn() -> CheckResult; 6] = [
        check_dns_spoof,
        check_tcp_rst,
        check_sni_block,
        check_siberian_block,
        check_quic_block,
        check_cidr_whitelist,
    ];
    let mut handles: Vec<(usize, std::thread::JoinHandle<CheckResult>)> = Vec::new();
    for (i, &check) in checks.iter().enumerate() {
        if block_checks.get(i) {
            handles.push((i, std::thread::spawn(check)));
        }
    }
    let mut results: Vec<Option<CheckResult>> = vec![None; checks.len()];
    for (i, handle) in handles {
        results[i] = Some(handle.join().unwrap_or_else(|_| CheckResult::skip("Thread panic")));
    }
    results
        .into_iter()
        .map(|r| r.unwrap_or_else(|| CheckResult::skip("Not selected")))
        .collect()
}

fn wait_for_nfqws(timeout: Duration) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    let mut running = false;
    while std::time::Instant::now() < deadline {
        if crate::runner::nfqws_process_running() || crate::platform::is_nfqws_running() {
            running = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    if running {
        // Let nfqws bind its nfqueue before probing.
        std::thread::sleep(Duration::from_millis(500));
    }
    running
}

fn domain_check_error() -> DomainCheckResult {
    DomainCheckResult {
        domain: String::new(),
        alive: CheckStatus::Error,
        http: CheckStatus::Error,
        tls12: CheckStatus::Error,
        tls13: CheckStatus::Error,
        quic: CheckStatus::Error,
        baseline_pass: false,
        detail: "check thread panicked".to_string(),
        http_count: 0,
        quic_count: 0,
    }
}

pub fn run_all(
    config: &AutotuneConfig,
    progress: &dyn Fn(usize, usize),
    backend: &dyn FirewallBackend,
    interface: &str,
) -> AutotuneResults {
    // Run network checks once (shared across all presets)
    let block_results = run_network_checks(&config.block_checks);
    let net_check_count = config.block_checks.count_enabled();

    let all_strategies = crate::strategy::get_strategies();
    let loaded = load_strategy_files(&config.strategy_indices, &all_strategies);
    let strat_count = loaded.len();

    let proto_steps = count_protocol_steps(config);

    // Calculate total steps: network checks + per-preset domain checks + estimated strategy tests
    let mut total = net_check_count;
    for &preset_idx in config.preset_indices.iter() {
        let domain_count = get_domains_for_preset(preset_idx).len();
        total += domain_count * (1 + proto_steps);
    }
    total += config.preset_indices.len() * strat_count * 100; // rough estimate for strategy tests

    let mut done = 0;

    // === Network checks ===
    for _result in block_results.iter() {
        done += 1;
        progress(done, total);
    }

    let mut preset_results: Vec<PresetResult> = Vec::new();
    let mut all_working_strategy_names: Vec<std::collections::HashSet<String>> = Vec::new();

    // Save ipset once for all presets
    let saved_ipset = save_ipset();
    set_ipset_any();

    for &preset_idx in config.preset_indices.iter() {
        let domains = get_domains_for_preset(preset_idx);
        let preset_name = if preset_idx < PRESETS.len() {
            PRESETS[preset_idx].name.to_string()
        } else {
            "Custom".to_string()
        };

        println!(
            "\n--- {} [{}] ---",
            rust_i18n::t!("autotune_domain_results"),
            preset_name
        );

        // === Per-domain protocol checks (without any strategy) ===
        let mut domain_checks = Vec::with_capacity(domains.len());
        let mut handles: Vec<std::thread::JoinHandle<DomainCheckResult>> = Vec::new();
        for d in &domains {
            let cfg = config.clone();
            let d = d.clone();
            handles.push(std::thread::spawn(move || check_domain(&cfg, &d)));
        }
        for handle in handles {
            domain_checks.push(handle.join().unwrap_or_else(|_| domain_check_error()));
            done += 1 + proto_steps;
            progress(done, total);
        }

        // Determine which domains are blocked (baseline TLS 1.3 failed)
        let blocked_domains: Vec<String> = domain_checks
            .iter()
            .filter(|dc| !dc.baseline_pass)
            .map(|dc| dc.domain.clone())
            .collect();

        // === Strategy testing with real nfqws ===
        let mut strategy_results: Vec<StrategyCheckResult> = Vec::new();
        let mut working_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        if !loaded.is_empty() && !blocked_domains.is_empty() {
            for (strat_name, strat_path) in &loaded {
                println!("  {} {}", rust_i18n::t!("autotune_testing"), strat_name);

                let started = crate::runner::run_zapret_silent(strat_path, interface, false, false, backend);
                done += 1;
                progress(done, total);

                if let Err(e) = started {
                    println!("    {} {}: {}", rust_i18n::t!("status_failed"), strat_name, e);
                    strategy_results.push(StrategyCheckResult::failed(strat_name, &blocked_domains));
                    for _ in &blocked_domains {
                        done += 1;
                        progress(done, total);
                    }
                    continue;
                }

                let nfqws_alive = wait_for_nfqws(Duration::from_secs(3));

                if !nfqws_alive {
                    println!(
                        "    {} {} (nfqws exited early)",
                        rust_i18n::t!("status_failed"),
                        strat_name
                    );
                    strategy_results.push(StrategyCheckResult::failed(strat_name, &blocked_domains));
                    crate::runner::stop_zapret(backend);
                    for _ in &blocked_domains {
                        done += 1;
                        progress(done, total);
                    }
                    continue;
                }

                const PROTOCOLS: usize = 4; // http, tls12, tls13, quic
                let mut handles: Vec<(usize, usize, std::thread::JoinHandle<bool>)> = Vec::new();
                for (di, domain) in blocked_domains.iter().enumerate() {
                    for proto in 0..PROTOCOLS {
                        let d = domain.clone();
                        let n = config.num_requests;
                        handles.push((
                            di,
                            proto,
                            std::thread::spawn(move || match proto {
                                0 => test_http(&d, n),
                                1 => test_tls(&d, "--tlsv1.2", n),
                                2 => test_tls(&d, "--tlsv1.3", n),
                                _ => test_quic(&d, n),
                            }),
                        ));
                    }
                }

                let mut results = vec![false; blocked_domains.len() * PROTOCOLS];
                for (di, proto, handle) in handles {
                    results[di * PROTOCOLS + proto] = handle.join().unwrap_or(false);
                }

                let mut pass = Vec::new();
                let mut fail = Vec::new();
                let mut http_works = false;
                let mut tls12_works = false;
                let mut tls13_works = false;
                let mut quic_works = false;
                let mut dc_results = Vec::with_capacity(blocked_domains.len());
                for (di, domain) in blocked_domains.iter().enumerate() {
                    let http_ok = results[di * PROTOCOLS];
                    let tls12_ok = results[di * PROTOCOLS + 1];
                    let tls13_ok = results[di * PROTOCOLS + 2];
                    let quic_ok = results[di * PROTOCOLS + 3];
                    if http_ok {
                        http_works = true;
                    }
                    if tls12_ok {
                        tls12_works = true;
                    }
                    if tls13_ok {
                        tls13_works = true;
                    }
                    if quic_ok {
                        quic_works = true;
                    }

                    // Browsers use HTTPS; plain HTTP (port 80) is not enough.
                    let ok = tls12_ok || tls13_ok;
                    if ok {
                        pass.push(domain.clone());
                    } else {
                        fail.push(domain.clone());
                    }
                    dc_results.push(DomainProtocolCheck {
                        domain: domain.clone(),
                        http: http_ok,
                        tls12: tls12_ok,
                        tls13: tls13_ok,
                        quic: quic_ok,
                    });
                    done += 1;
                    progress(done, total);
                }

                let mut protocols_working = Vec::new();
                if http_works {
                    protocols_working.push("HTTP".to_string());
                }
                if tls12_works {
                    protocols_working.push("TLS12".to_string());
                }
                if tls13_works {
                    protocols_working.push("TLS13".to_string());
                }
                if quic_works {
                    protocols_working.push("QUIC".to_string());
                }

                crate::runner::stop_zapret(backend);

                let works = pass.len() >= blocked_domains.len() / 2;
                if works {
                    working_names.insert(strat_name.clone());
                }
                strategy_results.push(StrategyCheckResult {
                    strategy_name: strat_name.clone(),
                    domains_pass: pass,
                    domains_fail: fail,
                    works,
                    protocols_working,
                    domain_checks: dc_results,
                });
            }
        } else if !loaded.is_empty() && blocked_domains.is_empty() {
            for (strat_name, _) in &loaded {
                done += 1;
                progress(done, total);
                for _ in &domains {
                    done += 1;
                    progress(done, total);
                }
                working_names.insert(strat_name.clone());
                strategy_results.push(StrategyCheckResult {
                    strategy_name: strat_name.clone(),
                    domains_pass: domains.clone(),
                    domains_fail: Vec::new(),
                    works: true,
                    protocols_working: vec![
                        "HTTP".to_string(),
                        "TLS12".to_string(),
                        "TLS13".to_string(),
                        "QUIC".to_string(),
                    ],
                    domain_checks: domains
                        .iter()
                        .map(|d| DomainProtocolCheck {
                            domain: d.clone(),
                            http: true,
                            tls12: true,
                            tls13: true,
                            quic: true,
                        })
                        .collect(),
                });
            }
        }

        preset_results.push(PresetResult {
            preset_name,
            domain_checks,
            strategy_results,
        });
        all_working_strategy_names.push(working_names);
    }

    // Find common strategies (work across ALL presets)
    let common_strategies = if config.preset_indices.len() > 1 && !all_working_strategy_names.is_empty() {
        let mut common: std::collections::HashSet<String> = all_working_strategy_names[0].clone();
        for wm in &all_working_strategy_names[1..] {
            common.retain(|name| wm.contains(name));
        }
        let mut v: Vec<String> = common.into_iter().collect();
        v.sort();
        v
    } else {
        // Single preset: all working strategies are "common"
        preset_results
            .first()
            .map(|pr| {
                let mut names: Vec<String> = pr
                    .strategy_results
                    .iter()
                    .filter(|s| s.works)
                    .map(|s| s.strategy_name.clone())
                    .collect();
                names.sort();
                names
            })
            .unwrap_or_default()
    };

    // Restore original ipset
    if let Some(ref saved) = saved_ipset {
        restore_ipset(saved);
        println!("  {}", rust_i18n::t!("autotune_ipset_restored"));
    }

    let results = AutotuneResults {
        block_results,
        preset_results,
        common_strategies,
    };

    save_results_file(&results);
    results
}



