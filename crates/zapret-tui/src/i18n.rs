//! Localization for `zapret-tui`.
//!
//! See `zapret_core::i18n`: the tables are per-crate, the locale files are shared.

/// Activate the locale chosen by `zapret_core::i18n::detect_locale`.
///
/// The UI must not re-detect on its own: the binary picks one language for the
/// whole process and every crate has to agree with it.
pub fn init(locale: &str) {
    rust_i18n::set_locale(locale);
}
