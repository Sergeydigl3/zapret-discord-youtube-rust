//! Network probes: DNS, TCP, TLS and QUIC reachability.
//!
//! These are pure measurements with no knowledge of autotune. Autotune is the
//! feature built on top of them, so anything that answers "can I reach X"
//! belongs here and not in `crate::autotune`.

pub mod cancel;
pub mod checks_domain;
pub mod checks_network;
pub mod dns;
pub mod probe;
pub mod quic;
