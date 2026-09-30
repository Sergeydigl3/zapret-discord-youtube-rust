use std::collections::HashMap;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::Mutex;

pub const TEST_DOMAINS: &[&str] = &["discord.com", "youtube.com", "cdn.discordapp.com"];

pub const KNOWN_IPS: &[(&str, &[&str])] = &[
    (
        "discord.com",
        &["162.159.128.233", "162.159.135.232", "162.159.136.232"],
    ),
    ("youtube.com", &["142.250.150.46", "216.58.209.46", "142.250.185.78"]),
    ("google.com", &["142.250.185.78", "216.58.215.14"]),
];

static DNS_CACHE: Mutex<Option<HashMap<String, Vec<IpAddr>>>> = Mutex::new(None);

pub fn resolve_domain(domain: &str) -> Vec<IpAddr> {
    if let Ok(mut guard) = DNS_CACHE.lock() {
        let cache = guard.get_or_insert_with(HashMap::new);
        if let Some(ips) = cache.get(domain) {
            return ips.clone();
        }
        let addrs: Vec<IpAddr> = (domain, 0)
            .to_socket_addrs()
            .map(|addrs| addrs.map(|a| a.ip()).collect())
            .unwrap_or_default();
        if !addrs.is_empty() {
            cache.insert(domain.to_string(), addrs.clone());
        }
        addrs
    } else {
        (domain, 0)
            .to_socket_addrs()
            .map(|addrs| addrs.map(|a| a.ip()).collect())
            .unwrap_or_default()
    }
}

#[allow(dead_code)]
pub fn clear_dns_cache() {
    if let Ok(mut guard) = DNS_CACHE.lock() {
        *guard = None;
    }
}
