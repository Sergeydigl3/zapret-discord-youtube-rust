#[cfg(not(target_os = "windows"))]
pub mod iptables;
#[cfg(not(target_os = "windows"))]
pub mod nftables;

#[cfg(target_os = "windows")]
pub mod windivert;

pub trait FirewallBackend {
    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str) -> Result<(), String>;
    fn clear(&self) -> Result<(), String>;
}
