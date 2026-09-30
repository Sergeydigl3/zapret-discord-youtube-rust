//! Terminal UI of zapret-rust.
//!
//! The crate owns every screen, menu and long-running interactive task, and it
//! may only use `zapret-core` through its public API. It exports nothing back.
//!
//! How a key becomes a state change:
//!
//! ```text
//!   key ──▶ session::handle_key ──▶ state::actions::{on_activate,on_cycle,on_back}
//!                                        │
//!                                        └──▶ actions::<screen>  (one file per screen)
//!
//!   mouse ▶ draw records where the rows landed in AppState::hit
//!        ─▶ session::handle_mouse ──▶ actions::mouse (click = cursor + press)
//!
//!   a screen that starts a long job only sets a should_* flag on AppState;
//!   session then calls tasks::<job>.
//! ```
//!
//! A job that runs a child program hands the terminal over; a job that runs
//! in-process work — the autotune sweep — keeps it and paints frames, which is
//! why `views` exists alongside `draw`.

// The macro joins this path onto CARGO_MANIFEST_DIR, so `../..` reaches the
// workspace-level locale directory shared with `zapret-core` and the binary.
// It must run at the crate root: `t!` reaches the generated table through
// `crate::_rust_i18n_t`, which only exists at the root.
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
