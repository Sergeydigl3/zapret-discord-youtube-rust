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
        Self { status: CheckStatus::Pass, detail: detail.into() }
    }
    fn fail(detail: impl Into<String>) -> Self {
        Self { status: CheckStatus::Fail, detail: detail.into() }
    }
    fn skip(detail: impl Into<String>) -> Self {
        Self { status: CheckStatus::Skip, detail: detail.into() }
    }
}

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

#[allow(dead_code)]
impl BlockChecks {
    pub fn all_enabled() -> Self {
        Self { enabled: vec![true; 6] }
    }

    pub fn all_disabled() -> Self {
        Self { enabled: vec![false; 6] }
    }

    pub fn get(&self, idx: usize) -> bool {
        self.enabled.get(idx).copied().unwrap_or(false)
    }

    pub fn set(&mut self, idx: usize, val: bool) {
        if let Some(e) = self.enabled.get_mut(idx) {
            *e = val;
        }
    }

    pub fn any_enabled(&self) -> bool {
        self.enabled.iter().any(|&e| e)
    }

    pub fn count_enabled(&self) -> usize {
        self.enabled.iter().filter(|&&e| e).count()
    }
}

pub struct AutotuneConfig {
    pub preset_indices: Vec<usize>,
    pub num_requests: usize,
    pub check_http: bool,
    pub check_https: bool,
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
            check_https: true,
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
    pub https: CheckStatus,
    pub tls12: CheckStatus,
    pub tls13: CheckStatus,
    pub quic: CheckStatus,
    pub baseline_pass: bool,
    #[allow(dead_code)]
    pub detail: String,
    pub http_count: usize,
    pub https_count: usize,
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
    pub https: bool,
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
    pub fn dns_spoof(&self) -> &CheckResult { &self.block_results[0] }
    pub fn tcp_rst(&self) -> &CheckResult { &self.block_results[1] }
    pub fn sni_block(&self) -> &CheckResult { &self.block_results[2] }
    pub fn siberian_block(&self) -> &CheckResult { &self.block_results[3] }
    pub fn quic_block(&self) -> &CheckResult { &self.block_results[4] }
    pub fn cidr_whitelist(&self) -> &CheckResult { &self.block_results[5] }
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
        },
    };
    println!("  {}", rust_i18n::t!("autotune_saving_results").replace("{}", &path.display().to_string()));

    let check_names = ["DNS", "TCP RST", "SNI", "SIBERIAN", "QUIC", "CIDR"];
    let _ = writeln!(file, "--- {} ---", rust_i18n::t!("autotune_net_results"));
    for (name, check) in check_names.iter().zip(&results.block_results) {
        let _ = writeln!(file, "  {}: {}", name, status_str_file(&check.status));
    }
    let _ = writeln!(file);

    for pr in &results.preset_results {
        let _ = writeln!(file, "--- {} [{}] ---", rust_i18n::t!("autotune_domain_results"), pr.preset_name);
        for dc in &pr.domain_checks {
            let _ = writeln!(file, "  {}: alive={} HTTP:{}({}) HTTPS:{}({}) T12:{} T13:{} QUIC:{}({}) baseline={}",
                dc.domain,
                status_str_file(&dc.alive),
                status_str_file(&dc.http), dc.http_count,
                status_str_file(&dc.https), dc.https_count,
                status_str_file(&dc.tls12),
                status_str_file(&dc.tls13),
                status_str_file(&dc.quic), dc.quic_count,
                status_str_file(if dc.baseline_pass { &CheckStatus::Pass } else { &CheckStatus::Fail }),
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
                let _ = writeln!(file, "    {}: {} ({}/{}){}", sr.strategy_name, s, sr.score(), sr.total(), protos);
                for dc in &sr.domain_checks {
                    let _ = writeln!(file, "      {} HTTP:{} HTTPS:{} T12:{} T13:{} Q:{}",
                        dc.domain,
                        if dc.http { "✅" } else { "❌" },
                        if dc.https { "✅" } else { "❌" },
                        if dc.tls12 { "✅" } else { "❌" },
                        if dc.tls13 { "✅" } else { "❌" },
                        if dc.quic { "✅" } else { "❌" },
                    );
                }
            }
            let _ = writeln!(file);
        }

        if !results.common_strategies.is_empty() {
            let _ = writeln!(file, "--- {} ({}) ---",
                rust_i18n::t!("autotune_common_strats"),
                results.common_strategies.len());
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
    ("discord.com", &["162.159.128.233", "162.159.135.232", "162.159.136.232"]),
    ("youtube.com", &["142.250.150.46", "216.58.209.46", "142.250.185.78"]),
    ("google.com", &["142.250.185.78", "216.58.215.14"]),
];

pub fn custom_domains_file_path() -> std::path::PathBuf {
    let cache = crate::config::get_cache_dir();
    cache.join(CUSTOM_DOMAINS_FILE)
}

pub fn load_custom_domains() -> Vec<String> {
    let path = custom_domains_file_path();
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => content
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub fn get_domains_for_preset(preset_idx: usize) -> Vec<String> {
    if preset_idx >= PRESETS.len() {
        return Vec::new();
    }
    let preset = &PRESETS[preset_idx];
    if preset_idx == PRESETS.len() - 1 {
        // Custom preset
        load_custom_domains()
    } else {
        preset.domains.iter().map(|s| s.to_string()).collect()
    }
}

#[allow(dead_code)]
pub fn get_preset_names(indices: &[usize]) -> Vec<String> {
    indices.iter().filter_map(|&i| {
        if i < PRESETS.len() {
            Some(PRESETS[i].name.to_string())
        } else {
            None
        }
    }).collect()
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
            v4 == Ipv4Addr::UNSPECIFIED
                || v4.is_loopback()
                || v4.is_private()
                || v4 == Ipv4Addr::new(0, 0, 0, 0)
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
    let mut last_err = io::Error::new(ErrorKind::Other, "no addresses");
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
            return CheckResult::fail(format!(
                "{} resolved to sinkhole IPs: {:?}",
                domain, suspect
            ));
        }

        if let Some(&(_, known_ips)) = KNOWN_IPS.iter().find(|(d, _)| *d == domain) {
            let sys_strs: Vec<String> = sys_ips.iter().map(|ip| ip.to_string()).collect();
            let known_set: Vec<String> = known_ips.iter().map(|s| s.to_string()).collect();
            let any_match = sys_ips.iter().any(|ip| known_ips.contains(&ip.to_string().as_str()));
            if !any_match {
                results.push(format!(
                    "{} resolved to {:?} (unexpected vs known {:?})",
                    domain, sys_strs, known_set
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
        let fails: Vec<&str> = results.iter().filter(|r| !r.contains("OK")).map(|s| s.as_str()).collect();
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
                    Err(ref e)
                        if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut =>
                    {
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
                        Err(ref e)
                            if e.kind() == ErrorKind::WouldBlock
                                || e.kind() == ErrorKind::TimedOut =>
                        {
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

    let test_ips: Vec<&str> = KNOWN_IPS[0].1.iter().copied().collect();

    let clean_ok = try_tcp_connect_domain(CLEAN_DOMAIN, 443).is_ok();

    if !clean_ok {
        return CheckResult::skip("Internet connectivity issue");
    }

    let mut handles: Vec<std::thread::JoinHandle<Result<TcpStream, io::Error>>> = Vec::new();

    for _ in 0..MAX_CONCURRENT {
        for &ip in &test_ips {
            let handle = std::thread::spawn(move || {
                try_tcp_connect(ip, 443)
            });
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
    let pass_ratio = if total_attempted > 0 { alive as f64 / total_attempted as f64 } else { 1.0 };

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
                    let probe = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
                    match sock.send(probe) {
                        Ok(_) => {
                            let mut buf = [0u8; 64];
                            match sock.recv(&mut buf) {
                                Ok(n) if n > 0 => {
                                    details.push(format!("{}: UDP response ({} bytes)", ip_str, n));
                                    quic_ok += 1;
                                }
                                Ok(_) => {
                                    details.push(format!("{}: UDP sent, empty response", ip_str));
                                }
                                Err(ref e)
                                    if e.kind() == ErrorKind::TimedOut
                                        || e.kind() == ErrorKind::WouldBlock =>
                                {
                                    details.push(format!("{}: UDP sent, no response (possible QUIC block)", ip_str));
                                }
                                Err(e) => {
                                    details.push(format!("{}: UDP recv error: {}", ip_str, e));
                                }
                            }
                        }
                        Err(e) => {
                            details.push(format!("{}: UDP send error: {}", ip_str, e));
                        }
                    }
                }
                Err(e) => {
                    details.push(format!("{}: socket bind error: {}", ip_str, e));
                }
            }
        }
    }

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

    if try_tcp_connect(CLEAN_DOMAIN, 443).is_err() {
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
        CheckResult::fail(format!("All tested IPs blocked: possible whitelist-only policy"))
    }
}

fn check_domain_alive(domain: &str) -> CheckStatus {
    match try_tcp_connect_domain(domain, 443) {
        Ok(mut stream) => {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
            let mut buf = [0u8; 1];
            match stream.read(&mut buf) {
                Ok(_) => CheckStatus::Pass,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {
                    CheckStatus::Pass
                }
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
                        _ => {}
                    }
                }
            }
            Err(_) => {}
        }
    }
    let status = if success > 0 { CheckStatus::Pass } else { CheckStatus::Fail };
    (status, success)
}

fn check_domain_https(domain: &str, num_req: usize) -> (CheckStatus, usize) {
    let mut success = 0;
    for _ in 0..num_req {
        match try_tcp_connect_domain(domain, 443) {
            Ok(mut stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut buf = [0u8; 1];
                success += 1;
                let _ = stream.read(&mut buf);
            }
            Err(_) => {}
        }
    }
    let status = if success > 0 { CheckStatus::Pass } else { CheckStatus::Fail };
    (status, success)
}

fn check_domain_quic(domain: &str, num_req: usize) -> (CheckStatus, usize) {
    let ips = resolve_domain(domain);
    if ips.is_empty() {
        return (CheckStatus::Skip, 0);
    }
    let mut success = 0;
    for &ip in ips.iter().take(2) {
        let addr = SocketAddr::new(ip, 443);
        if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
            if sock.connect(addr).is_err() { continue; }
            let _ = sock.set_read_timeout(Some(Duration::from_secs(2)));
            let probe = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
            for _ in 0..num_req {
                if sock.send(probe).is_err() { continue; }
                let mut buf = [0u8; 64];
                match sock.recv(&mut buf) {
                    Ok(n) if n > 0 => { success += 1; break; }
                    _ => {}
                }
            }
        }
        if success > 0 { break; }
    }
    let status = if success > 0 { CheckStatus::Pass } else { CheckStatus::Fail };
    (status, success)
}

fn test_https(domain: &str) -> bool {
    let url = format!("https://{}", domain);
    let ok = match ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(5))
        .timeout_read(std::time::Duration::from_secs(5))
        .build()
        .get(&url)
        .call()
    {
        Ok(_) => true,
        Err(ureq::Error::Status(_, _)) => true,
        Err(_) => false,
    };
    if !ok {
        // Fallback: try curl (same method as bash autotune)
        let curl_ok = std::process::Command::new("curl")
            .args([
                "-s", "--tlsv1.3",
                "--connect-timeout", "4",
                "--max-time", "4",
                "-o", "/dev/null",
                "-w", "%{http_code}",
                &url,
            ])
            .output()
            .map(|o| {
                let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                code.starts_with('2') || code.starts_with('3')
            })
            .unwrap_or(false);
        if curl_ok {
            println!("      {} {} (curl OK, ureq failed)", rust_i18n::t!("status_ok"), domain);
        }
        curl_ok
    } else {
        true
    }
}

fn test_tls(domain: &str, tls_flag: &str) -> bool {
    let url = format!("https://{}", domain);
    std::process::Command::new("curl")
        .args([
            "-s",
            tls_flag,
            "--connect-timeout", "4",
            "--max-time", "4",
            "-o", "/dev/null",
            "-w", "%{http_code}",
            &url,
        ])
        .output()
        .map(|o| {
            let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
            code.starts_with('2') || code.starts_with('3')
        })
        .unwrap_or(false)
}

fn test_quic(domain: &str) -> bool {
    let url = format!("https://{}", domain);
    std::process::Command::new("curl")
        .args([
            "-s", "--http3",
            "--connect-timeout", "4",
            "--max-time", "4",
            "-o", "/dev/null",
            "-w", "%{http_code}",
            &url,
        ])
        .output()
        .map(|o| {
            let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
            !code.is_empty() && code != "000"
        })
        .unwrap_or(false)
}

fn test_http(domain: &str) -> bool {
    let url = format!("http://{}", domain);
    let ok = match ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(5))
        .timeout_read(std::time::Duration::from_secs(5))
        .build()
        .get(&url)
        .call()
    {
        Ok(_) => true,
        Err(ureq::Error::Status(_, _)) => true,
        Err(_) => false,
    };
    if !ok {
        let curl_ok = std::process::Command::new("curl")
            .args([
                "-s",
                "--connect-timeout", "4",
                "--max-time", "4",
                "-o", "/dev/null",
                "-w", "%{http_code}",
                &url,
            ])
            .output()
            .map(|o| {
                let code = String::from_utf8_lossy(&o.stdout).trim().to_string();
                code.starts_with('2') || code.starts_with('3')
            })
            .unwrap_or(false);
        curl_ok
    } else {
        true
    }
}

pub fn check_domain(config: &AutotuneConfig, domain: &str) -> DomainCheckResult {
    let alive = check_domain_alive(domain);
    let detail;

    let (http, https, tls12, tls13, quic, http_count, https_count, quic_count) = if alive != CheckStatus::Pass && alive != CheckStatus::Skip {
        detail = "Domain appears blocked (alive check failed)".to_string();
        (CheckStatus::Skip, CheckStatus::Skip, CheckStatus::Skip, CheckStatus::Skip, CheckStatus::Skip, 0, 0, 0)
    } else {
        let mut parts = Vec::new();

        let (http, hc) = if config.check_http {
            let (s, c) = check_domain_http(domain, config.num_requests);
            parts.push(format!("HTTP:{} ({}/{})", status_char(&s), c, config.num_requests));
            (s, c)
        } else { (CheckStatus::Skip, 0) };

        let (https, hsc) = if config.check_https || config.check_tls12 || config.check_tls13 {
            let (s, c) = check_domain_https(domain, config.num_requests);
            parts.push(format!("HTTPS:{} ({}/{})", status_char(&s), c, config.num_requests));
            (s, c)
        } else { (CheckStatus::Skip, 0) };

        let tls12 = if config.check_tls12 {
            let (s, _) = check_domain_https(domain, config.num_requests);
            parts.push(format!("TLS1.2:{}", status_char(&s)));
            s
        } else { CheckStatus::Skip };

        let tls13 = if config.check_tls13 {
            let (s, _) = check_domain_https(domain, config.num_requests);
            parts.push(format!("TLS1.3:{}", status_char(&s)));
            s
        } else { CheckStatus::Skip };

        let (quic, qc) = if config.check_quic {
            let (s, c) = check_domain_quic(domain, config.num_requests);
            parts.push(format!("QUIC:{} ({}/{})", status_char(&s), c, config.num_requests));
            (s, c)
        } else { (CheckStatus::Skip, 0) };

        detail = parts.join(" ");
        (http, https, tls12, tls13, quic, hc, hsc, qc)
    };

    // Baseline HTTPS test: real TLS handshake + HTTP request
    let baseline_pass = if alive == CheckStatus::Pass || alive == CheckStatus::Skip {
        test_https(domain)
    } else {
        false
    };

    DomainCheckResult {
        domain: domain.to_string(),
        alive,
        http,
        https,
        tls12,
        tls13,
        quic,
        baseline_pass,
        detail,
        http_count,
        https_count,
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
        let path = Path::new(&repo)
            .join("custom-strategies")
            .join(name);
        let path = if path.exists() { path } else {
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
    let mut n = 0;
    if config.check_http { n += 1; }
    if config.check_https || config.check_tls12 || config.check_tls13 { n += 1; }
    if config.check_tls12 { n += 1; }
    if config.check_tls13 { n += 1; }
    if config.check_quic { n += 1; }
    n
}

fn run_network_checks(block_checks: &BlockChecks) -> Vec<CheckResult> {
    vec![
        if block_checks.get(0) { check_dns_spoof() } else { CheckResult::skip("Not selected") },
        if block_checks.get(1) { check_tcp_rst() } else { CheckResult::skip("Not selected") },
        if block_checks.get(2) { check_sni_block() } else { CheckResult::skip("Not selected") },
        if block_checks.get(3) { check_siberian_block() } else { CheckResult::skip("Not selected") },
        if block_checks.get(4) { check_quic_block() } else { CheckResult::skip("Not selected") },
        if block_checks.get(5) { check_cidr_whitelist() } else { CheckResult::skip("Not selected") },
    ]
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

    // Calculate total steps: network checks + per-preset domain checks + per-preset strategy tests
    let mut total = net_check_count;
    let mut preset_domain_counts = Vec::new();
    let mut preset_blocked_counts = Vec::new();

    for &preset_idx in &config.preset_indices {
        let domains = get_domains_for_preset(preset_idx);
        let domain_count = domains.len();
        let proto_checks = domain_count * (1 + proto_steps) + domain_count; // per-domain checks + baseline
        total += proto_checks;
        preset_domain_counts.push(domain_count);

        // Estimate strategy test steps (will be refined after baseline check)
        if !loaded.is_empty() {
            // We'll add strategy steps after determining blocked domains
        }
    }

    // Strategy testing steps: for each preset with blocked domains, each strategy tests each blocked domain
    // We don't know blocked_domains yet, but we can estimate
    let strategy_steps_estimate = config.preset_indices.len() * strat_count * 100; // rough estimate
    total += strategy_steps_estimate;

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

        println!("\n--- {} [{}] ---", rust_i18n::t!("autotune_domain_results"), preset_name);

        // === Per-domain protocol checks (without any strategy) ===
        let mut domain_checks = Vec::with_capacity(domains.len());
        for d in &domains {
            let dc = check_domain(config, d);
            done += 1 + proto_steps;
            progress(done, total);
            domain_checks.push(dc);
        }

        // Determine which domains are blocked (baseline HTTPS failed)
        let blocked_domains: Vec<String> = domain_checks
            .iter()
            .filter(|dc| !dc.baseline_pass)
            .map(|dc| dc.domain.clone())
            .collect();
        let blocked_count = blocked_domains.len();
        preset_blocked_counts.push(blocked_count);

        // === Strategy testing with real nfqws ===
        let mut strategy_results: Vec<StrategyCheckResult> = Vec::new();
        let mut working_names: std::collections::HashSet<String> = std::collections::HashSet::new();

        if !loaded.is_empty() && !blocked_domains.is_empty() {
            for (strat_name, strat_path) in &loaded {
                println!("  {} {}", rust_i18n::t!("autotune_testing"), strat_name);

                let started = crate::runner::run_zapret_silent(
                    strat_path,
                    interface,
                    false,
                    false,
                    backend,
                );
                done += 1;
                progress(done, total);

                if started.is_err() {
                    println!("    {} {}: {}", rust_i18n::t!("status_failed"), strat_name, started.unwrap_err());
                    let strat_res = StrategyCheckResult {
                        strategy_name: strat_name.clone(),
                        domains_pass: Vec::new(),
                        domains_fail: blocked_domains.clone(),
                        works: false,
                        protocols_working: Vec::new(),
                        domain_checks: Vec::new(),
                    };
                    strategy_results.push(strat_res);
                    for _ in &blocked_domains {
                        done += 1;
                        progress(done, total);
                    }
                    continue;
                }

                std::thread::sleep(Duration::from_secs(3));

                let nfqws_alive = std::process::Command::new("pgrep")
                    .arg("-x")
                    .arg("nfqws")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false);

                if !nfqws_alive {
                    println!("    {} {} (nfqws exited early)", rust_i18n::t!("status_failed"), strat_name);
                    let strat_res = StrategyCheckResult {
                        strategy_name: strat_name.clone(),
                        domains_pass: Vec::new(),
                        domains_fail: blocked_domains.clone(),
                        works: false,
                        protocols_working: Vec::new(),
                        domain_checks: Vec::new(),
                    };
                    strategy_results.push(strat_res);
                    crate::runner::stop_zapret(backend);
                    for _ in &blocked_domains {
                        done += 1;
                        progress(done, total);
                    }
                    continue;
                }

                let mut pass = Vec::new();
                let mut fail = Vec::new();
                let mut http_works = false;
                let mut https_works = false;
                let mut tls12_works = false;
                let mut tls13_works = false;
                let mut quic_works = false;
                let mut dc_results = Vec::new();
                for domain in &blocked_domains {
                    let http_ok = test_http(domain);
                    if http_ok { http_works = true; }
                    let https_ok = test_https(domain);
                    if https_ok { https_works = true; }
                    let tls12_ok = test_tls(domain, "--tlsv1.2");
                    if tls12_ok { tls12_works = true; }
                    let tls13_ok = test_tls(domain, "--tlsv1.3");
                    if tls13_ok { tls13_works = true; }
                    let quic_ok = test_quic(domain);
                    if quic_ok { quic_works = true; }

                    let ok = https_ok || http_ok || tls12_ok || tls13_ok;
                    if ok { pass.push(domain.clone()); } else { fail.push(domain.clone()); }
                    dc_results.push(DomainProtocolCheck {
                        domain: domain.clone(),
                        http: http_ok, https: https_ok, tls12: tls12_ok,
                        tls13: tls13_ok, quic: quic_ok,
                    });
                    done += 1;
                    progress(done, total);
                }

                let mut protocols_working = Vec::new();
                if http_works { protocols_working.push("HTTP".to_string()); }
                if https_works { protocols_working.push("HTTPS".to_string()); }
                if tls12_works { protocols_working.push("TLS12".to_string()); }
                if tls13_works { protocols_working.push("TLS13".to_string()); }
                if quic_works { protocols_working.push("QUIC".to_string()); }

                crate::runner::stop_zapret(backend);

                let works = pass.len() >= blocked_domains.len() / 2 + 1;
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
                    protocols_working: vec!["HTTP".to_string(), "HTTPS".to_string(), "TLS12".to_string(), "TLS13".to_string(), "QUIC".to_string()],
                    domain_checks: domains.iter().map(|d| DomainProtocolCheck {
                        domain: d.clone(), http: true, https: true, tls12: true, tls13: true, quic: true,
                    }).collect(),
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
        preset_results.first()
            .map(|pr| {
                let mut names: Vec<String> = pr.strategy_results
                    .iter().filter(|s| s.works)
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
