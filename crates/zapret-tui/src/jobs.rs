//! A long sweep on a worker thread.
//!
//! The frame loop must not stop while a sweep works, or the spinner stops, the
//! elapsed clock freezes and a program that is working hard looks hung. So a job
//! runs on its own thread and reports into a shared view; the loop keeps painting
//! on its own timer and only the cancel key is read. The cost is that the job no
//! longer owns the terminal, which is why every sweep takes an event callback.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use ratatui::Frame;
use zapret_core::firewall::QuietGuard;
use zapret_wrapper::autotune::{self, AutotuneResults, SweepEvent};
use zapret_wrapper::domains::{ttl, TtlEvent};

use crate::state::AppState;
use crate::views::progress::{BarFormat, ProgressView};

/// What a finished job leaves behind.
#[derive(Debug)]
pub enum JobOutcome {
    Autotune(Box<AutotuneResults>, bool),
    Ttl(Result<u8, String>),
}

/// What a worker gets to talk to the UI with.
///
/// Everything it is given is `Send`, so the closure it runs in can be sent to a
/// thread; the only thing it must not do is touch the terminal.
pub struct Reporter {
    view: Arc<Mutex<ProgressView>>,
    cancel: Arc<AtomicBool>,
}

impl Reporter {
    /// Fold something into the picture the frame loop is painting.
    pub fn update(&self, f: impl FnOnce(&mut ProgressView)) {
        if let Ok(mut view) = self.view.lock() {
            f(&mut view);
        }
    }

    /// True once the user has asked the job to stop.
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

/// A sweep in flight, or the memory of one.
pub struct Job {
    view: Arc<Mutex<ProgressView>>,
    cancel: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    outcome: Arc<Mutex<Option<JobOutcome>>>,
}

impl Job {
    /// Start `work` on its own thread. The view is handed to the UI, not to the
    /// worker: the worker gets a [`Reporter`] and never sees the terminal.
    pub fn spawn(view: ProgressView, work: impl FnOnce(Reporter) -> JobOutcome + Send + 'static) -> Self {
        let view = Arc::new(Mutex::new(view));
        let cancel = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let outcome = Arc::new(Mutex::new(None));

        let worker_view = Arc::clone(&view);
        let worker_cancel = Arc::clone(&cancel);
        let worker_finished = Arc::clone(&finished);
        let worker_outcome = Arc::clone(&outcome);
        std::thread::spawn(move || {
            let result = work(Reporter {
                view: worker_view,
                cancel: worker_cancel,
            });
            if let Ok(mut slot) = worker_outcome.lock() {
                *slot = Some(result);
            }
            // Set last: the loop must never read a finished job whose outcome is
            // not in place yet.
            worker_finished.store(true, Ordering::SeqCst);
        });

        Self {
            view,
            cancel,
            finished,
            outcome,
        }
    }

    pub fn render(&self, f: &mut Frame) {
        if let Ok(view) = self.view.lock() {
            view.render(f, f.area());
        }
    }

    /// Ask the job to stop. The sweep notices at its next event, which is at
    /// least as often as its slowest single probe.
    pub fn request_cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }

    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    /// The result, once. Returns `None` if asked before the job finished.
    pub fn take_outcome(&mut self) -> Option<JobOutcome> {
        if !self.is_finished() {
            return None;
        }
        self.outcome.lock().ok().and_then(|mut slot| slot.take())
    }
}

/// Start the autotune sweep. Everything the sweep needs is copied out of the app
/// first: the worker thread cannot borrow it, and the menu underneath keeps being
/// drawn from the same state the next time the loop comes round.
pub fn start_autotune(app: &AppState) -> Job {
    let config = app.autotune_config.clone();
    #[cfg(target_os = "linux")]
    let interface = app.interface().to_string();
    let backend = app.owned_backend();

    Job::spawn(ProgressView::new(), move |reporter| {
        let _quiet = QuietGuard::new();
        let mut sink = |event: SweepEvent| -> bool {
            reporter.update(|view| view.apply(event));
            !reporter.cancelled()
        };
        #[cfg(target_os = "linux")]
        let results = autotune::run_all(&config, &mut sink, &*backend, &interface);
        #[cfg(not(target_os = "linux"))]
        let results = autotune::run_all(&config, &mut sink, &*backend);
        JobOutcome::Autotune(Box::new(results), reporter.cancelled())
    })
}

/// Start the fixed-TTL sweep.
pub fn start_ttl(app: &AppState) -> Job {
    let strategy = app.strategies.get(app.selected_strategy).cloned().unwrap_or_default();
    #[cfg(target_os = "linux")]
    let interface = app.interface().to_string();
    let backend = app.owned_backend();

    let view = ProgressView::titled(rust_i18n::t!("ttl_screen_title").to_string(), BarFormat::Count);

    Job::spawn(view, move |reporter| {
        let _quiet = QuietGuard::new();
        if strategy.is_empty() {
            return JobOutcome::Ttl(Err(rust_i18n::t!("msg_no_strat").into_owned()));
        }
        let mut on_event = |event: TtlEvent| -> bool {
            reporter.update(|view| view.apply_ttl(event));
            !reporter.cancelled()
        };
        #[cfg(target_os = "linux")]
        let result = ttl::autopick_ttl(&strategy, &interface, &*backend, &mut on_event);
        #[cfg(not(target_os = "linux"))]
        let result = ttl::autopick_ttl(&strategy, &*backend, &mut on_event);
        JobOutcome::Ttl(result)
    })
}
