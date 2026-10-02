//! Terminal UI of zapret-rust.
//!
//! The crate owns every screen, menu and long-running interactive task, and it
//! may only use `zapret-core` through its public API. It exports nothing back.
//! A key becomes a state change through `session::handle_key`, which dispatches
//! into `state::actions`; a screen that starts a long job only raises a
//! `should_*` flag on [`AppState`], and the session then starts the job.

rust_i18n::i18n!("../../locales", fallback = "en");

pub mod i18n;

pub mod event;
pub mod jobs;
pub mod screen;
pub mod session;
pub mod tasks;

pub mod draw;
pub mod editor;
pub mod menus;
pub mod mouse;
pub mod state;
pub mod theme;
pub mod views;

pub use event::{spawn_event_reader, EventReader};
pub use screen::setup_console;
pub use session::run_tui;
pub use state::AppState;
