//! Long-running jobs that hand the terminal to a child program.
//!
//! These are the ones a sweep is *not*: a download, the text editor, a `git`
//! invocation. They own the console for a while and give it back, which is the
//! opposite of what the sweeps do — see [`crate::jobs`].

pub mod download;
pub mod edit;
