//! What the sweep reports while it runs.
//!
//! The sweep used to print to stdout, which forced every caller to take the
//! terminal over and left the user with a wall of text it could not scroll. It
//! now reports through this event stream instead: the sweep stays a pure
//! function of the network, and the UI decides whether that becomes a bar, a
//! log or a line of plain text.

/// How a log line should read, so a UI can colour it without parsing the text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Good,
    Bad,
}

/// One thing that happened during the sweep.
#[derive(Debug, Clone, PartialEq)]
pub enum SweepEvent {
    /// The sweep moved on to a named step. The text is already localized.
    Phase(String),
    /// The step counter moved. `total` is 0 until the sweep has sized itself
    /// up, and `done` may pass it when the sweep is cut short.
    Step { done: usize, total: usize },
    /// A line for the log pane. The text is already localized.
    Log { level: LogLevel, text: String },
}

/// The sweep's side of the stream: it emits events and asks once per step
/// whether to keep going.
pub(crate) struct Reporter<'a> {
    sink: &'a mut dyn FnMut(SweepEvent) -> bool,
    total: usize,
}

impl<'a> Reporter<'a> {
    pub fn new(sink: &'a mut dyn FnMut(SweepEvent) -> bool, total: usize) -> Self {
        Self { sink, total }
    }

    /// Move the counter on. `false` means the sweep should stop now.
    pub fn step(&mut self, done: usize) -> bool {
        (self.sink)(SweepEvent::Step {
            done,
            total: self.total,
        })
    }

    pub fn phase(&mut self, name: impl Into<String>) {
        (self.sink)(SweepEvent::Phase(name.into()));
    }

    /// A line for the log. Emitting one never stops the sweep.
    pub fn log(&mut self, level: LogLevel, text: impl Into<String>) {
        (self.sink)(SweepEvent::Log {
            level,
            text: text.into(),
        });
    }
}
