use super::FirewallBackend;
use nftables::helper::{apply_ruleset, get_current_ruleset};
use nftables::schema::Nftables;
use serde_json::json;

const NFT_TABLE: &str = "zapret";
const NFT_CHAIN: &str = "zapret_chain";

pub struct NftablesBackend;

impl FirewallBackend for NftablesBackend {
    fn clear(&self) -> Result<(), String> {
        println!("{}", rust_i18n::t!("msg_clear_nftables"));
        
        let current_ruleset = get_current_ruleset().map_err(|e| format!("Failed to get current ruleset: {:?}", e))?;
        let mut has_table = false;
        
        for obj in current_ruleset.objects.iter() {
            let s = serde_json::to_string(obj).unwrap_or_default();
            if s.contains(NFT_TABLE) {
                has_table = true;
                break;
            }
        }
        
        if has_table {
            let clear_payload = json!({
                "nftables": [
                    { "flush": { "chain": { "family": "ip", "table": NFT_TABLE, "name": NFT_CHAIN } } },
                    { "delete": { "chain": { "family": "ip", "table": NFT_TABLE, "name": NFT_CHAIN } } },
                    { "delete": { "table": { "family": "ip", "name": NFT_TABLE } } }
                ]
            });
            
            let n = serde_json::from_value::<Nftables>(clear_payload).map_err(|e| e.to_string())?;
            apply_ruleset(&n).map_err(|e| format!("Failed to apply ruleset during clear: {:?}", e))?;
        }
        
        Ok(())
    }

    fn setup(&self, tcp_ports: &str, udp_ports: &str, interface: &str) -> Result<(), String> {
        let _ = self.clear(); // Ignore errors during clear as it might not be fully configured

        println!("{}", rust_i18n::t!("msg_setup_nftables"));

        let mut rules = vec![
            json!({ "add": { "table": { "family": "ip", "name": NFT_TABLE } } }),
            json!({ "add": { "chain": { "family": "ip", "table": NFT_TABLE, "name": NFT_CHAIN, "type": "filter", "hook": "output", "prio": 0 } } })
        ];

        if !tcp_ports.is_empty() {
            let mut exprs = vec![
                json!({ "match": { "op": "!=", "left": { "meta": { "key": "mark" } }, "right": "0x40000000" } }),
                json!({ "match": { "op": "==", "left": { "payload": { "protocol": "tcp", "field": "dport" } }, "right": { "set": tcp_ports.split(',').map(|s| s.trim().parse::<u32>().unwrap_or(0)).collect::<Vec<u32>>() } } }),
                json!({ "counter": null }),
                json!({ "queue": { "num": 200, "bypass": true } })
            ];
            
            if !interface.is_empty() && interface != "any" {
                exprs.insert(0, json!({ "match": { "op": "==", "left": { "meta": { "key": "oifname" } }, "right": interface } }));
            }

            rules.push(json!({
                "add": {
                    "rule": {
                        "family": "ip",
                        "table": NFT_TABLE,
                        "chain": NFT_CHAIN,
                        "expr": exprs,
                        "comment": "zapret-rust-rule-tcp"
                    }
                }
            }));
        }

        if !udp_ports.is_empty() {
            let mut exprs = vec![
                json!({ "match": { "op": "!=", "left": { "meta": { "key": "mark" } }, "right": "0x40000000" } }),
                json!({ "match": { "op": "==", "left": { "payload": { "protocol": "udp", "field": "dport" } }, "right": { "set": udp_ports.split(',').map(|s| s.trim().parse::<u32>().unwrap_or(0)).collect::<Vec<u32>>() } } }),
                json!({ "counter": null }),
                json!({ "queue": { "num": 200, "bypass": true } })
            ];

            if !interface.is_empty() && interface != "any" {
                exprs.insert(0, json!({ "match": { "op": "==", "left": { "meta": { "key": "oifname" } }, "right": interface } }));
            }

            rules.push(json!({
                "add": {
                    "rule": {
                        "family": "ip",
                        "table": NFT_TABLE,
                        "chain": NFT_CHAIN,
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
