use super::FirewallBackend;

pub struct WinDivertBackend;

impl FirewallBackend for WinDivertBackend {
    fn clear(&self) -> Result<(), String> {
        println!("{}", rust_i18n::t!("msg_clear_windivert"));
        Ok(())
    }

    fn setup(&self, _tcp_ports: &str, _udp_ports: &str, _interface: &str) -> Result<(), String> {
        println!("{}", rust_i18n::t!("msg_setup_windivert"));
        Ok(())
    }
}
