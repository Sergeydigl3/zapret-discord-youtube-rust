use super::FirewallBackend;
use std::process::Command;

pub struct IptablesBackend;

const CHAIN_NAME: &str = "zapret_chain";

impl FirewallBackend for IptablesBackend {
    fn clear(&self) -> Result<(), String> {
        println!("Очистка правил iptables...");

        // Remove from OUTPUT chain
        let _ = Command::new("iptables")
            .args(["-t", "filter", "-D", "OUTPUT", "-j", CHAIN_NAME])
            .status();

        // Flush and delete chain
        let _ = Command::new("iptables")
            .args(["-t", "filter", "-F", CHAIN_NAME])
            .status();

        let _ = Command::new("iptables")
            .args(["-t", "filter", "-X", CHAIN_NAME])
            .status();

        Ok(())
    }

    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str) -> Result<(), String> {
        let _ = self.clear();

        println!("Настройка iptables...");

        // Create chain
        let status = Command::new("iptables")
            .args(["-t", "filter", "-N", CHAIN_NAME])
            .status()
            .map_err(|e| format!("Failed to create iptables chain: {}", e))?;

        if !status.success() {
            // Ignore if chain already exists, but continue
        }

        // Add to OUTPUT
        Command::new("iptables")
            .args(["-t", "filter", "-I", "OUTPUT", "-j", CHAIN_NAME])
            .status()
            .map_err(|e| format!("Failed to link iptables chain: {}", e))?;

        if !tcp_ports.is_empty() {
            let ports = tcp_ports.replace(" ", "");
            let mut args = vec!["-t", "filter", "-A", CHAIN_NAME];
            if !interface.is_empty() && interface != "any" {
                args.extend(vec!["-o", interface]);
            }
            args.extend(vec![
                "-p", "tcp",
                "-m", "multiport", "--dports", &ports,
                "-m", "mark", "!", "--mark", "0x40000000/0x40000000",
                "-j", "NFQUEUE", "--queue-num", "200", "--queue-bypass",
            ]);
            Command::new("iptables").args(&args).status().ok();
        }

        if !udp_ports.is_empty() {
            let ports = udp_ports.replace(" ", "");
            let mut args = vec!["-t", "filter", "-A", CHAIN_NAME];
            if !interface.is_empty() && interface != "any" {
                args.extend(vec!["-o", interface]);
            }
            args.extend(vec![
                "-p", "udp",
                "-m", "multiport", "--dports", &ports,
                "-m", "mark", "!", "--mark", "0x40000000/0x40000000",
                "-j", "NFQUEUE", "--queue-num", "200", "--queue-bypass",
            ]);
            Command::new("iptables").args(&args).status().ok();
        }

        Ok(())
    }
}
