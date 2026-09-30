//! Delegation to the shared capability helper in `crate::process`.

use std::path::Path;

pub(crate) fn set_cap(bin_path: &Path) -> bool {
    crate::process::set_cap(bin_path)
}
