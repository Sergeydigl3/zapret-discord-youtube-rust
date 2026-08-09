use std::sync::atomic::{AtomicBool, Ordering};

pub static CANCELLED: AtomicBool = AtomicBool::new(false);

pub fn reset_cancel() {
    CANCELLED.store(false, Ordering::Relaxed);
}

pub fn is_cancelled() -> bool {
    CANCELLED.load(Ordering::Relaxed)
}

pub fn trigger_cancel() {
    CANCELLED.store(true, Ordering::Relaxed);
    kill_active_curls();
}

pub fn kill_active_curls() {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "curl.exe", "/T"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("pkill")
            .args(["-9", "curl"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

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
    pub fn pass(detail: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Pass,
            detail: detail.into(),
        }
    }
    pub fn fail(detail: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Fail,
            detail: detail.into(),
        }
    }
    pub fn skip(detail: impl Into<String>) -> Self {
        Self {
            status: CheckStatus::Skip,
            detail: detail.into(),
        }
    }
}

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
    pub fn failed(name: &str, blocked_domains: &[String]) -> Self {
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
    pub elapsed_secs: u64,
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

pub fn status_str_file(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pass => "OK",
        CheckStatus::Fail => "BLOCKED",
        CheckStatus::Skip => "SKIP",
        CheckStatus::Error => "ERROR",
    }
}

pub fn status_char(s: &CheckStatus) -> &'static str {
    match s {
        CheckStatus::Pass => "OK",
        CheckStatus::Fail => "BLOCKED",
        CheckStatus::Skip => "-",
        CheckStatus::Error => "ERR",
    }
}
