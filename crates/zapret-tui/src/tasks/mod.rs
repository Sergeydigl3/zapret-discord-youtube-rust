//! Long-running interactive jobs.
//!
//! Each of these takes the terminal over for a while, prints its own output and
//! gives the terminal back. The session loop only decides *when* one of them
//! runs, by checking the `should_*` flags on [`crate::state::AppState`].

pub mod autotune;
pub mod download;
pub mod edit;
pub mod ttl_run;
