//! Picking the language the whole process speaks.
//!
//! The translation tables are declared in each crate's `lib.rs` — the `i18n!`
//! macro generates a `crate::_rust_i18n_t` that `t!` reaches through the crate
//! root, so it has to run at the crate root and not inside a submodule. All four
//! crates share one flat locale directory, and this is the single place that
//! decides which of its files is used: `rust_i18n::set_locale` is global, so
//! setting it once here is what makes the tables agree.

/// Pick the active locale from the process environment.
pub fn detect() -> &'static str {
    let loc = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    if loc.to_lowercase().starts_with("ru") {
        "ru"
    } else {
        "en"
    }
}
