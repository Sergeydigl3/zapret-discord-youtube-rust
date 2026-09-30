//! Localization for `zapret-tui`.
//!
//! Every crate declares its own translation table over the same flat
//! `locales/` directory, and the binary picks one language for the whole
//! process. This is the TUI's half of that: it accepts the decision rather than
//! making one.

/// Activate the locale chosen by `src/locale.rs` in the binary.
///
/// The UI must not re-detect on its own: the binary picks one language for the
/// whole process and every crate has to agree with it.
pub fn init(locale: &str) {
    rust_i18n::set_locale(locale);
}
