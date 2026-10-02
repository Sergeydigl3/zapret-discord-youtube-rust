//! Terminal UI of zapret-rust.
//!
//! The crate owns every screen, menu and long-running interactive task, and it
//! may only use `zapret-core` through its public API. It exports nothing back.
//!
//! The UI is a tui-realm `Application`: every screen is a component mounted once
//! at start-up, focus moves between them, and a component's only output is a
//! [`Msg`]. [`model::Model::update`] applies that message — in the same file as
//! the component that sent it, which is the whole reason a screen is one file.
//! What is outside the app is [`state::AppState`]: the data the screens show,
//! and the handful of operations that read the outside world.

rust_i18n::i18n!("../../locales", fallback = "en");

pub mod i18n;

pub mod channel_port;
pub mod event;
pub mod jobs;
pub mod screen;
pub mod tasks;

pub mod draw;
pub mod editor;
pub mod menu;
pub mod menus;
pub mod model;
pub mod msg;
pub mod screens;
pub mod state;
pub mod theme;
pub mod views;

pub use event::{spawn_event_reader, EventReader};
pub use model::run_tui;
pub use screen::setup_console;
pub use state::AppState;
