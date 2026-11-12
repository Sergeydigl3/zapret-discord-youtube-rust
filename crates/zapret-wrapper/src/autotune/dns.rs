use std::collections::HashMap;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::Mutex;

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
