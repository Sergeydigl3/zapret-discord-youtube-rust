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

const CHAIN_POST: &str = "zapret_post";
const CHAIN_PRE: &str = "zapret_pre";

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

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-D", "POSTROUTING", "-j", CHAIN_POST])
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-D", "PREROUTING", "-j", CHAIN_PRE])
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-F", CHAIN_POST])
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-F", CHAIN_PRE])
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-X", CHAIN_POST])
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-X", CHAIN_PRE])
            .stderr(Stdio::null())
            .status();

        Ok(())
    }

    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str) -> Result<(), String> {
        let _ = self.clear();

        println!("{}", rust_i18n::t!("msg_setup_iptables"));

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-N", CHAIN_POST])
            .stderr(Stdio::null())
            .status();

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-N", CHAIN_PRE])
            .stderr(Stdio::null())
            .status();

        Command::new("iptables")
            .args(["-t", "mangle", "-I", "POSTROUTING", "-j", CHAIN_POST])
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_iptables_link"), e))?;

        let _ = Command::new("iptables")
            .args(["-t", "mangle", "-I", "PREROUTING", "-j", CHAIN_PRE])
            .stderr(Stdio::null())
            .status();

        if !tcp_ports.is_empty() {
            let ports = normalize_ports(&tcp_ports.replace(" ", ""));

            let mut args = vec!["-t", "mangle", "-A", CHAIN_POST];
            if !interface.is_empty() && interface != "any" {
                args.extend(["-o", interface]);
            }
            args.extend([
                "-p", "tcp",
                "-m", "multiport", "--dports", &ports,
                "-m", "connbytes", "--connbytes-dir=original", "--connbytes-mode=packets", "--connbytes", "1:6",
                "-m", "mark", "!", "--mark", "0x40000000/0x40000000",
                "-j", "NFQUEUE", "--queue-num", "200", "--queue-bypass",
            ]);
            Command::new("iptables").args(&args).stderr(Stdio::null()).status().ok();

            let mut pre_args = vec!["-t", "mangle", "-A", CHAIN_PRE];
            if !interface.is_empty() && interface != "any" {
                pre_args.extend(["-i", interface]);
            }
            pre_args.extend([
                "-p", "tcp",
                "-m", "multiport", "--sports", &ports,
                "-m", "connbytes", "--connbytes-dir=reply", "--connbytes-mode=packets", "--connbytes", "1:3",
                "-m", "mark", "!", "--mark", "0x40000000/0x40000000",
                "-j", "NFQUEUE", "--queue-num", "200", "--queue-bypass",
            ]);
            Command::new("iptables").args(&pre_args).stderr(Stdio::null()).status().ok();
        }

        if !udp_ports.is_empty() {
            let ports = normalize_ports(&udp_ports.replace(" ", ""));

            let mut args = vec!["-t", "mangle", "-A", CHAIN_POST];
            if !interface.is_empty() && interface != "any" {
                args.extend(["-o", interface]);
            }
            args.extend([
                "-p", "udp",
                "-m", "multiport", "--dports", &ports,
                "-m", "connbytes", "--connbytes-dir=original", "--connbytes-mode=packets", "--connbytes", "1:6",
                "-m", "mark", "!", "--mark", "0x40000000/0x40000000",
                "-j", "NFQUEUE", "--queue-num", "200", "--queue-bypass",
            ]);
            Command::new("iptables").args(&args).stderr(Stdio::null()).status().ok();
        }

        Ok(())
    }
}
