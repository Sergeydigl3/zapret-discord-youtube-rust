use crate::firewalls::FirewallBackend;
use std::process::{Command, Stdio};

pub struct IptablesBackend;

pub fn is_available() -> bool {
    Command::new("iptables")
        .arg("--version")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

const CHAIN_NAME: &str = "zapret_chain";

fn normalize_ports(ports: &str) -> String {
    ports.split(',')
        .map(|p| {
            let p = p.trim();
            if let Some((lo, hi)) = p.split_once('-') {
                format!("{}:{}", lo.trim(), hi.trim())
            } else {
                p.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

impl FirewallBackend for IptablesBackend {
    fn clear(&self) -> Result<(), String> {
        println!("{}", rust_i18n::t!("msg_clear_iptables"));

        // Remove from OUTPUT chain
        let _ = Command::new("iptables")
            .args(["-t", "filter", "-D", "OUTPUT", "-j", CHAIN_NAME])
            .stderr(Stdio::null())
            .status();

        // Flush and delete chain
        let _ = Command::new("iptables")
            .args(["-t", "filter", "-F", CHAIN_NAME])
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("iptables")
            .args(["-t", "filter", "-X", CHAIN_NAME])
            .stderr(Stdio::null())
            .status();

        Ok(())
    }

    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str) -> Result<(), String> {
        let _ = self.clear();

        println!("{}", rust_i18n::t!("msg_setup_iptables"));

        // Create chain
        let _ = Command::new("iptables")
            .args(["-t", "filter", "-N", CHAIN_NAME])
            .stderr(Stdio::null())
            .status();

        // Add to OUTPUT
        Command::new("iptables")
            .args(["-t", "filter", "-I", "OUTPUT", "-j", CHAIN_NAME])
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_iptables_link"), e))?;

        if !tcp_ports.is_empty() {
            let ports = normalize_ports(&tcp_ports.replace(" ", ""));
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
            Command::new("iptables").args(&args).stderr(Stdio::null()).status().ok();
        }

        if !udp_ports.is_empty() {
            let ports = normalize_ports(&udp_ports.replace(" ", ""));
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
            Command::new("iptables").args(&args).stderr(Stdio::null()).status().ok();
        }

        Ok(())
    }
}
