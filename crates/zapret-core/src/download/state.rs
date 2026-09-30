//! Install-state predicates, delegated to the single path owner in `crate::paths`.
pub fn check_nfqws_installed() -> bool {
    crate::paths::nfqws_installed()
}

pub fn check_strategies_installed() -> bool {
    crate::paths::strategies_installed()
}
