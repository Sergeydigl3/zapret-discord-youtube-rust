use crate::firewall::{notice, FirewallBackend};
use nftables::helper::{apply_ruleset, get_current_ruleset};
use nftables::schema::Nftables;
use serde_json::{json, Value};

use super::tool_available;

const NFT_TABLE: &str = "zapret";
const NFT_CHAIN_POST: &str = "zapret_post";
const NFT_CHAIN_PRE: &str = "zapret_pre";
const NFT_TABLE_NAT: &str = "zapret_nat";
const NFT_CHAIN_PNAT: &str = "zapret_pnat";

pub struct NftablesBackend;

pub fn is_available() -> bool {
    tool_available("nft")
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

/// Whether the ruleset holds anything under this name. A batch naming an
/// object that is not there is refused as a whole, so teardown asks first.
fn has_object(current_ruleset: &Nftables, name: &str) -> bool {
    current_ruleset
        .objects
        .iter()
        .any(|obj| serde_json::to_string(obj).unwrap_or_default().contains(name))
}

/// The NAT objects that hide a routed device's address from the internet.
/// A separate table because `masquerade` is only valid in an `ip` NAT chain.
fn router_rules(interface: &str) -> Vec<Value> {
    let mut exprs = Vec::new();
    if !interface.is_empty() && interface != "any" {
        exprs.push(json!({ "match": { "op": "==", "left": { "meta": { "key": "oifname" } }, "right": interface } }));
    }
    exprs.push(json!({ "masquerade": null }));

    vec![
        json!({ "add": { "table": { "family": "ip", "name": NFT_TABLE_NAT } } }),
        json!({ "add": { "chain": { "family": "ip", "table": NFT_TABLE_NAT, "name": NFT_CHAIN_PNAT, "type": "nat", "hook": "postrouting", "prio": 100 } } }),
        json!({ "add": { "rule": { "family": "ip", "table": NFT_TABLE_NAT, "chain": NFT_CHAIN_PNAT, "expr": exprs, "comment": "zapret-rust-rule-masquerade" } } }),
    ]
}

impl FirewallBackend for NftablesBackend {
    fn clear(&self) -> Result<(), String> {
        notice(&rust_i18n::t!("msg_clear_nftables"));

        let current_ruleset = get_current_ruleset().map_err(|e| format!("Failed to get current ruleset: {:?}", e))?;

        if has_object(&current_ruleset, NFT_TABLE) {
            let mut cmds = vec![
                json!({ "flush": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_POST } } }),
                json!({ "flush": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_PRE } } }),
                json!({ "delete": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_POST } } }),
                json!({ "delete": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_PRE } } }),
                json!({ "delete": { "table": { "family": "inet", "name": NFT_TABLE } } }),
            ];

            if has_object(&current_ruleset, NFT_TABLE_NAT) {
                cmds.push(json!({ "delete": { "table": { "family": "ip", "name": NFT_TABLE_NAT } } }));
            }

            let clear_payload = json!({ "nftables": cmds });

            let n = serde_json::from_value::<Nftables>(clear_payload).map_err(|e| e.to_string())?;
            apply_ruleset(&n).map_err(|e| format!("Failed to apply ruleset during clear: {:?}", e))?;
        }

        Ok(())
    }

    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str, router: bool) -> Result<(), String> {
        let _ = self.clear();

        notice(&rust_i18n::t!("msg_setup_nftables"));

        let mut rules = vec![
            json!({ "add": { "table": { "family": "inet", "name": NFT_TABLE } } }),
            json!({ "add": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_POST, "type": "filter", "hook": "postrouting", "prio": -150 } } }),
            json!({ "add": { "chain": { "family": "inet", "table": NFT_TABLE, "name": NFT_CHAIN_PRE, "type": "filter", "hook": "prerouting", "prio": 0 } } }),
        ];

        if router {
            rules.extend(router_rules(interface));
        }

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

        let payload = json!({ "nftables": rules });

        let n = serde_json::from_value::<Nftables>(payload).map_err(|e| format!("JSON Schema error: {}", e))?;
        apply_ruleset(&n).map_err(|e| format!("Failed to apply ruleset: {:?}", e))?;

        Ok(())
    }
}
