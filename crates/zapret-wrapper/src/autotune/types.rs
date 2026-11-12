#[derive(Debug, Clone, PartialEq)]
pub enum CheckStatus {
    Pass,
    Fail,
    Skip,
    Error,
}

/// Protocol switches accepted by `--autotune-protocols`, in help order.
pub const PROTOCOL_NAMES: &[&str] = &["http", "tls12", "tls13", "quic"];

#[derive(Debug, Clone)]
pub struct AutotuneConfig {
    pub preset_indices: Vec<usize>,
    pub num_requests: usize,
    pub check_http: bool,
    pub check_tls12: bool,
    pub check_tls13: bool,
    pub check_quic: bool,
    pub strategy_indices: Vec<usize>,
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
    pub preset_results: Vec<PresetResult>,
    pub common_strategies: Vec<String>, // strategies that work across ALL selected presets
    pub elapsed_secs: u64,
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
