use crate::firewalls::FirewallBackend;
use nftables::helper::{apply_ruleset, get_current_ruleset};
use nftables::schema::Nftables;
use serde_json::{json, Value};
use std::process::{Command, Stdio};

const NFT_TABLE: &str = "zapret";
const NFT_TABLE_NAT: &str = "zapret_nat";
const NFT_CHAIN_POST: &str = "zapret_post";
const NFT_CHAIN_PRE: &str = "zapret_pre";
const NFT_CHAIN_FWD: &str = "zapret_fwd";

pub struct NftablesBackend;

pub fn is_available() -> bool {
    Command::new("nft")
        .arg("--version")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn parse_ports(ports: &str) -> Vec<Value> {
    ports.split(',')
        .map(|p| {
            let p = p.trim();
            if let Some((lo, hi)) = p.split_once('-') {
                let lo: u32 = lo.trim().parse().unwrap_or(0);
                let hi: u32 = hi.trim().parse().unwrap_or(0);
                json!({ "range": [lo, hi] })
            } else {
                let port: u32 = p.parse().unwrap_or(0);
                json!(port)
            }
        })
        .collect()
}

fn has_table(current_ruleset: &Nftables, name: &str) -> bool {
    for obj in current_ruleset.objects.iter() {
        let s = serde_json::to_string(obj).unwrap_or_default();
        if s.contains(name) {
            return true;
        }
    }
    false
}

impl FirewallBackend for NftablesBackend {
    fn clear(&self) -> Result<(), String> {
        println!("{}", rust_i18n::t!("msg_clear_nftables"));

        let current_ruleset = get_current_ruleset().map_err(|e| format!("Failed to get current ruleset: {:?}", e))?;

        if has_table(&current_ruleset, NFT_TABLE) {
            // Flush all known chains (fwd chain may or may not exist — errors are ignored)
            let _ = {
                let p = json!({
                    "nftables": [
                        { "flush": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_FWD } } },
                        { "delete": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_FWD } } }
                    ]
                });
                serde_json::from_value::<Nftables>(p).ok().map(|n| apply_ruleset(&n))
            };

            let clear_payload = json!({
                "nftables": [
                    { "flush": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_POST } } },
                    { "flush": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_PRE } } },
                    { "delete": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_POST } } },
                    { "delete": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_PRE } } },
                    { "delete": { "table": { "family": "inet", "name": NFT_TABLE } } }
                ]
            });

            let n = serde_json::from_value::<Nftables>(clear_payload).map_err(|e| e.to_string())?;
            apply_ruleset(&n).map_err(|e| format!("Failed to apply ruleset during clear: {:?}", e))?;
        }

        if has_table(&current_ruleset, NFT_TABLE_NAT) {
            let nat_payload = json!({
                "nftables": [
                    { "delete": { "table": { "family": "ip", "name": NFT_TABLE_NAT } } }
                ]
            });
            if let Ok(n) = serde_json::from_value::<Nftables>(nat_payload) {
                let _ = apply_ruleset(&n);
            }
        }

        Ok(())
    }

    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str, router_mode: bool) -> Result<(), String> {
        let _ = self.clear();

        println!("{}", rust_i18n::t!("msg_setup_nftables"));

        let mut rules = vec![
            json!({ "add": { "table": { "family": "inet", "name": NFT_TABLE } } }),
            json!({ "add": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_POST, "type": "filter", "hook": "postrouting", "prio": -150 } } }),
            json!({ "add": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_PRE, "type": "filter", "hook": "prerouting", "prio": 0 } } }),
        ];

        if !tcp_ports.is_empty() {
            let mut exprs = vec![
                json!({ "match": { "op": "!=", "left": { "meta": { "key": "mark" } }, "right": "0x40000000" } }),
                json!({ "match": { "op": "==", "left": { "payload": { "protocol": "tcp", "field": "dport" } }, "right": { "set": parse_ports(tcp_ports) } } }),
                json!({ "match": { "op": "==", "left": { "ct": { "key": "packets", "dir": "original" } }, "right": { "range": [1, 6] } } }),
                json!({ "counter": null }),
                json!({ "queue": { "num": 200, "bypass": true } })
            ];

            if !interface.is_empty() && interface != "any" {
                exprs.insert(0, json!({ "match": { "op": "==", "left": { "meta": { "key": "oifname" } }, "right": interface } }));
            }

            rules.push(json!({
                "add": {
                    "rule": {
                        "family": "inet",
                        "table": NFT_TABLE,
                        "chain": NFT_CHAIN_POST,
                        "expr": exprs,
                        "comment": "zapret-rust-rule-tcp"
                    }
                }
            }));

            let pre_exprs = vec![
                json!({ "match": { "op": "==", "left": { "payload": { "protocol": "tcp", "field": "sport" } }, "right": { "set": parse_ports(tcp_ports) } } }),
                json!({ "match": { "op": "==", "left": { "ct": { "key": "packets", "dir": "reply" } }, "right": { "range": [1, 3] } } }),
                json!({ "counter": null }),
                json!({ "queue": { "num": 200, "bypass": true } })
            ];

            rules.push(json!({
                "add": {
                    "rule": {
                        "family": "inet",
                        "table": NFT_TABLE,
                        "chain": NFT_CHAIN_PRE,
                        "expr": pre_exprs,
                        "comment": "zapret-rust-rule-tcp-reply"
                    }
                }
            }));
        }

        if !udp_ports.is_empty() {
            let mut exprs = vec![
                json!({ "match": { "op": "!=", "left": { "meta": { "key": "mark" } }, "right": "0x40000000" } }),
                json!({ "match": { "op": "==", "left": { "payload": { "protocol": "udp", "field": "dport" } }, "right": { "set": parse_ports(udp_ports) } } }),
                json!({ "match": { "op": "==", "left": { "ct": { "key": "packets", "dir": "original" } }, "right": { "range": [1, 6] } } }),
                json!({ "counter": null }),
                json!({ "queue": { "num": 200, "bypass": true } })
            ];

            if !interface.is_empty() && interface != "any" {
                exprs.insert(0, json!({ "match": { "op": "==", "left": { "meta": { "key": "oifname" } }, "right": interface } }));
            }

            rules.push(json!({
                "add": {
                    "rule": {
                        "family": "inet",
                        "table": NFT_TABLE,
                        "chain": NFT_CHAIN_POST,
                        "expr": exprs,
                        "comment": "zapret-rust-rule-udp"
                    }
                }
            }));
        }

        if router_mode {
            // In router mode, nfqws must intercept forwarded packets at the FORWARD hook
            // (not postrouting). At postrouting, the input interface is gone and nfqws
            // cannot correctly handle packet reinjection for forwarded traffic.
            // This mirrors the reference zapretunix layout: forward hook for routed traffic.
            rules.push(json!({
                "add": {
                    "chain": {
                        "family": "inet",
                        "table": NFT_TABLE,
                        "name": NFT_CHAIN_FWD,
                        "type": "filter",
                        "hook": "forward",
                        "prio": -150
                    }
                }
            }));

            if !tcp_ports.is_empty() {
                let mut fwd_exprs = vec![
                    json!({ "match": { "op": "!=", "left": { "meta": { "key": "mark" } }, "right": "0x40000000" } }),
                    json!({ "match": { "op": "==", "left": { "payload": { "protocol": "tcp", "field": "dport" } }, "right": { "set": parse_ports(tcp_ports) } } }),
                    json!({ "match": { "op": "==", "left": { "ct": { "key": "packets", "dir": "original" } }, "right": { "range": [1, 6] } } }),
                    json!({ "counter": null }),
                    json!({ "queue": { "num": 200, "bypass": true } })
                ];
                if !interface.is_empty() && interface != "any" {
                    fwd_exprs.insert(0, json!({ "match": { "op": "==", "left": { "meta": { "key": "oifname" } }, "right": interface } }));
                }
                rules.push(json!({
                    "add": {
                        "rule": {
                            "family": "inet",
                            "table": NFT_TABLE,
                            "chain": NFT_CHAIN_FWD,
                            "expr": fwd_exprs,
                            "comment": "zapret-rust-fwd-tcp"
                        }
                    }
                }));
            }

            if !udp_ports.is_empty() {
                let mut fwd_exprs = vec![
                    json!({ "match": { "op": "!=", "left": { "meta": { "key": "mark" } }, "right": "0x40000000" } }),
                    json!({ "match": { "op": "==", "left": { "payload": { "protocol": "udp", "field": "dport" } }, "right": { "set": parse_ports(udp_ports) } } }),
                    json!({ "match": { "op": "==", "left": { "ct": { "key": "packets", "dir": "original" } }, "right": { "range": [1, 6] } } }),
                    json!({ "counter": null }),
                    json!({ "queue": { "num": 200, "bypass": true } })
                ];
                if !interface.is_empty() && interface != "any" {
                    fwd_exprs.insert(0, json!({ "match": { "op": "==", "left": { "meta": { "key": "oifname" } }, "right": interface } }));
                }
                rules.push(json!({
                    "add": {
                        "rule": {
                            "family": "inet",
                            "table": NFT_TABLE,
                            "chain": NFT_CHAIN_FWD,
                            "expr": fwd_exprs,
                            "comment": "zapret-rust-fwd-udp"
                        }
                    }
                }));
            }

            // NAT table: MASQUERADE outgoing traffic
            rules.push(json!({ "add": { "table": { "family": "ip", "name": NFT_TABLE_NAT } } }));
            rules.push(json!({
                "add": {
                    "chain": {
                        "family": "ip",
                        "table": NFT_TABLE_NAT,
                        "name": "postrouting",
                        "type": "nat",
                        "hook": "postrouting",
                        "prio": 100
                    }
                }
            }));

            let mut nat_exprs = Vec::new();
            if !interface.is_empty() && interface != "any" {
                nat_exprs.push(json!({ "match": { "op": "==", "left": { "meta": { "key": "oifname" } }, "right": interface } }));
            }
            nat_exprs.push(json!({ "masquerade": null }));

            rules.push(json!({
                "add": {
                    "rule": {
                        "family": "ip",
                        "table": NFT_TABLE_NAT,
                        "chain": "postrouting",
                        "expr": nat_exprs,
                        "comment": "zapret-rust-nat"
                    }
                }
            }));
        }

        let payload = json!({ "nftables": rules });

        let n = serde_json::from_value::<Nftables>(payload).map_err(|e| format!("JSON Schema error: {}", e))?;
        apply_ruleset(&n).map_err(|e| format!("Failed to apply ruleset: {:?}", e))?;

        Ok(())
    }
}
