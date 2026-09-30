//! Localization for `zapret-core`.
//!
//! The translation tables themselves are declared in `lib.rs`: the `i18n!` macro
//! generates a `crate::_rust_i18n_t` that `t!` reaches through the crate root,
//! so it has to run at the crate root and not inside a submodule.

/// Pick the active locale from the process environment.
///
/// Shared by every crate so all of them agree on one language.
pub fn detect_locale() -> &'static str {
    let loc = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    if loc.to_lowercase().starts_with("ru") {
        "ru"
    } else {
        "en"
    }
}

/// Activate the detected locale for this crate.
pub fn init() {
    rust_i18n::set_locale(detect_locale());
}
