use crate::firewall::{notice, FirewallBackend};
use std::process::{Command, Stdio};

use super::tool_available;

const CHAIN_POST: &str = "zapret_post";
const CHAIN_PRE: &str = "zapret_pre";
/// The mark zapret puts on packets it has already handled.
const HANDLED_MARK: &str = "0x40000000/0x40000000";
const QUEUE: [&str; 5] = ["-j", "NFQUEUE", "--queue-num", "200", "--queue-bypass"];

/// Per-chain parts of a divert rule: device flag, port flag, connbytes
/// direction, connbytes packet range. The pre chain takes replies.
const POST_SHAPE: [&str; 4] = ["-o", "--dports", "original", "1:6"];
const PRE_SHAPE: [&str; 4] = ["-i", "--sports", "reply", "1:3"];

/// Unhook both chains, then flush and delete them. A step on a chain that is not
/// there fails, and the failure is ignored: there was nothing to clean up.
const CLEAR_STEPS: &[(&str, &str, &str)] = &[
    ("-D", "POSTROUTING", CHAIN_POST),
    ("-D", "PREROUTING", CHAIN_PRE),
    ("-F", "", CHAIN_POST),
    ("-F", "", CHAIN_PRE),
    ("-X", "", CHAIN_POST),
    ("-X", "", CHAIN_PRE),
];

pub struct IptablesBackend;

pub fn is_available() -> bool {
    tool_available("iptables")
}

fn run<I: IntoIterator<Item = S>, S: AsRef<std::ffi::OsStr>>(args: I) {
    let _ = Command::new("iptables").args(args).stderr(Stdio::null()).status();
}

fn normalize_ports(ports: &str) -> String {
    ports.split(',')
        .map(|p| {
            let p = p.trim();
            match p.split_once('-') {
                Some((lo, hi)) => format!("{}:{}", lo.trim(), hi.trim()),
                None => p.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// The mangle rule that pushes matching traffic into nfqueue. `-m mark ! --mark`
/// is what keeps out what zapret has already handled.
fn divert(chain: &str, shape: &[&str; 4], proto: &str, ports: &str, interface: &str) -> Vec<String> {
    let [iface_flag, port_flag, ct_dir, ct_range] = *shape;
    // connbytes takes its direction as one `key=value` argument.
    let dir = format!("--connbytes-dir={ct_dir}");
    let mut args = vec!["-t", "mangle", "-A", chain];
    if !interface.is_empty() && interface != "any" {
        args.extend([iface_flag, interface]);
    }
    args.extend(["-p", proto, "-m", "multiport", port_flag, ports, "-m", "connbytes", &dir]);
    args.extend(["--connbytes-mode", "packets", "--connbytes", ct_range, "-m", "mark", "!", "--mark", HANDLED_MARK]);
    args.extend(QUEUE);
    args.into_iter().map(str::to_string).collect()
}

impl FirewallBackend for IptablesBackend {
    fn clear(&self) -> Result<(), String> {
        notice(&rust_i18n::t!("msg_clear_iptables"));

        for &(flag, hook, chain) in CLEAR_STEPS {
            let mut args = vec!["-t", "mangle", flag];
            if !hook.is_empty() {
                args.extend([hook, "-j"]);
            }
            args.push(chain);
            run(&args);
        }

        Ok(())
    }

    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str, _router: bool) -> Result<(), String> {
        let _ = self.clear();

        notice(&rust_i18n::t!("msg_setup_iptables"));

        run(["-t", "mangle", "-N", CHAIN_POST]);
        run(["-t", "mangle", "-N", CHAIN_PRE]);

        // The one step whose failure is reported: without this hook the rules
        // below would be installed but never reached.
        Command::new("iptables")
            .args(["-t", "mangle", "-I", "POSTROUTING", "-j", CHAIN_POST])
            .stderr(Stdio::null())
            .status()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_iptables_link"), e))?;

        run(["-t", "mangle", "-I", "PREROUTING", "-j", CHAIN_PRE]);

        for (chain, shape, proto, ports) in [
            (CHAIN_POST, &POST_SHAPE, "tcp", tcp_ports),
            (CHAIN_PRE, &PRE_SHAPE, "tcp", tcp_ports),
            (CHAIN_POST, &POST_SHAPE, "udp", udp_ports),
        ] {
            if !ports.is_empty() {
                let ports = normalize_ports(&ports.replace(' ', ""));
                run(divert(chain, shape, proto, &ports, interface));
            }
        }

        Ok(())
    }
}
