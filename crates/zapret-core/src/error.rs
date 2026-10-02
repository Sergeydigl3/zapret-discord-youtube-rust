//! Result type shared by the whole core: failures are reported as
//! human-readable strings, stated here once so the contract is in one place.

pub type ZResult<T> = Result<T, String>;
