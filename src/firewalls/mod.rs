#[cfg(target_os = "linux")]
pub mod backends;

#[cfg(target_os = "windows")]
pub mod windivert;

pub trait FirewallBackend {
    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str) -> Result<(), String>;
    fn clear(&self) -> Result<(), String>;
}

#[cfg(target_os = "linux")]
pub use backends::LinuxBackend;
