//! Result type shared by the whole core.
//!
//! The codebase still reports failures as human-readable strings; this alias
//! exists so the error contract is stated in one place and can later be swapped
//! for a real error enum without touching every signature at once.

pub type ZResult<T> = Result<T, String>;
