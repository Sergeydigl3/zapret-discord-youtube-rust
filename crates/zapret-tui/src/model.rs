//! The TUI session: the application, the message pump, and the frame loop.
//!
//! The loop has two modes. Normally it draws the focused screen and dispatches
//! what it says. While a sweep is in flight it draws the sweep's own screen
//! instead and reads only the cancel key — but it still repaints on the same
//! 50 ms tick, which is the point: the bar, the spinner and the elapsed clock
//! keep moving whether or not the sweep has anything new to say.
//!
//! While a job owns the screen the listener's ports are **locked**. The reader
//! thread keeps filling the channel and nothing else drains it, so the cancel
//! key reaches the loop that is looking for it and cannot leak into the menu
//! underneath. That is also why the channel is drained once the job is done.

use std::io;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Frame;
use ratatui::Terminal;
use tuirealm::application::PollStrategy;
use tuirealm::event::NoUserEvent;
use tuirealm::listener::EventListenerCfg;

use crate::channel_port::ChannelPort;
use crate::draw;
use crate::event::{drain_events, EventReader, SharedRx};
use crate::jobs::{start_autotune, start_ttl, JobOutcome};
use crate::menu::{HELP, WARN};
use crate::msg::{Msg, Pending};
use crate::screens::{self, Id, ScreenDef};
use crate::state::screens::ActiveScreen;
use crate::state::AppState;
use crate::tasks;

/// How long one frame waits for a key before repainting anyway.
const TICK: Duration = Duration::from_millis(50);

/// How often the listener asks the port whether there is anything to read. The
/// reader thread already sleeps in 50 ms slices, so polling faster would only
/// spin a core.
const PORT_INTERVAL: Duration = Duration::from_millis(10);

/// How many events one tick will hand over before giving up on the rest.
const MAX_POLL: usize = 8;

/// How long the listener's worker is given to leave a pass it is already in.
/// `lock_ports` only sets a flag, so this is the grace that stops one more
/// event being taken out from under whoever is reading directly.
const PORT_GRACE: Duration = Duration::from_millis(50);

/// The application, the state, and what the session still owes the process.
pub struct Model {
    pub app: screens::App,
    pub state: AppState,
    rx: SharedRx,
    /// The screen list, looked up by id whenever focus moves.
    defs: Vec<ScreenDef>,
    pub pending: Option<Pending>,
}

impl Model {
    pub fn new(state: AppState, rx: SharedRx) -> Self {
        // One source of input, and only one: the reader thread. The listener
        // polls it rather than the console, so no second thread ever waits on
        // the console input handle.
        let listener = EventListenerCfg::<NoUserEvent>::default()
            .add_port(Box::new(ChannelPort::new(rx.clone())), PORT_INTERVAL, MAX_POLL);
        let mut app = screens::App::init(listener);

        let defs = screens::all();
        for def in &defs {
            (def.mount)(&mut app, &state);
        }
        let main = Id::Main;
        let _ = app.active(&main);

        Self {
            app,
            state,
            rx,
            defs,
            pending: None,
        }
    }

    /// The component that currently has focus.
    fn focused(&self) -> Option<Id> {
        self.app.focus().copied()
    }

    fn def(&self, id: Id) -> Option<&ScreenDef> {
        self.defs.iter().find(|d| d.id == id)
    }

    /// Re-read the focused screen and paint it, plus the chrome around it.
    fn view(&mut self, f: &mut Frame) {
        let Some(id) = self.focused() else { return };
        // The rows come from the state, and the state can change under a screen
        // that is not looking at it, so they are re-read before every frame
        // rather than after every key.
        if let Some(def) = self.def(id) {
            (def.sync)(&mut self.app, &self.state);
        }
        let warning = self.warning();
        let chrome = draw::chrome(f, &self.state, warning.as_deref());
        let help = self.help_text();
        self.app.view(&id, f, chrome.body);
        draw::help(f, chrome.help, help, &self.state);
        if let Some(status) = chrome.status {
            draw::status(f, status, &self.state);
        }
    }

    /// What the focused screen wants said above it, if anything.
    fn warning(&self) -> Option<String> {
        let id = self.focused()?;
        self.app
            .query(&id, WARN)
            .ok()
            .flatten()
            .and_then(|q| q.into_attr().as_string().cloned())
    }

    /// The sentence about the row under the cursor, or the last thing the
    /// session had to say — which replaces it, and comes back on the next move.
    fn help_text(&mut self) -> String {
        if let Some(msg) = &self.state.status_message {
            return msg.clone();
        }
        let Some(id) = self.focused() else {
            return String::new();
        };
        self.app
            .query(&id, HELP)
            .ok()
            .flatten()
            .and_then(|q| q.into_attr().as_string().cloned())
            .unwrap_or_default()
    }

    /// Apply one message, and anything it answers with, until nothing is left.
    pub fn update(&mut self, msg: Msg) -> Option<Msg> {
        match msg {
            Msg::Moved => {
                // A message from the last action replaces the row's own
                // explanation, so a movement is what brings the explanation back.
                self.state.status_message = None;
                None
            }
            Msg::Redraw => None,
            Msg::Back => self.back(None),
            Msg::BackWith(text) => self.back(Some(text)),
            Msg::Quit => {
                self.pending = Some(Pending::Quit);
                None
            }
            Msg::Open(screen) => {
                self.state.open(screen);
                self.enter(screen);
                None
            }
            Msg::Main(m) => screens::main::update(self, m),
            #[cfg(target_os = "windows")]
            Msg::Defender(m) => screens::defender::update(self, m),
            Msg::Strategy(m) => screens::strategy::update(self, m),
            Msg::DownloadDeps(m) => screens::download::update_deps(self, m),
            Msg::Download(m) => screens::download::update_sub(self, m),
            Msg::Tag(m) => screens::tag::update(self, m),
            Msg::Gamefilter(m) => screens::gamefilter::update(self, m),
            Msg::Service(m) => screens::service::update(self, m),
            Msg::Lists(m) => screens::lists::update(self, m),
            Msg::Extended(m) => screens::extended::update(self, m),
            Msg::Ttl(m) => screens::ttl::update(self, m),
            Msg::Fakes(m) => screens::fakes::update(self, m),
            Msg::Autotune(m) => screens::autotune::update(self, m),
            Msg::Domains(m) => screens::autotune::update_domains(self, m),
            Msg::Protocols(m) => screens::autotune::update_protocols(self, m),
            Msg::BlockChecks(m) => screens::autotune::update_block_checks(self, m),
            Msg::Presets(m) => screens::autotune::update_presets(self, m),
            Msg::AutotuneStrategies(m) => screens::autotune::update_strategies(self, m),
            Msg::NumRequests(m) => screens::autotune::update_requests(self, m),
            Msg::Report(m) => screens::report_screen::update(self, m),
        }
    }

    /// One step up the back stack, or leave from the main menu.
    ///
    /// Going back is not the same as opening: a screen the user left keeps the
    /// row they left it on, so focus moves without the screen's own `enter`
    /// being told to start over.
    fn back(&mut self, message: Option<String>) -> Option<Msg> {
        if let Some(previous) = self.state.back() {
            self.focus(previous);
        } else {
            self.pending = Some(Pending::Quit);
        }
        self.state.status_message = message;
        None
    }

    /// Hand focus to a screen the state has just moved to.
    fn focus(&mut self, screen: ActiveScreen) {
        let id = screens::id_of(screen);
        let _ = self.app.active(&id);
    }

    /// Focus a screen that has just been opened: prepare it, then focus it.
    fn enter(&mut self, screen: ActiveScreen) {
        let id = screens::id_of(screen);
        if let Some(def) = self.def(id) {
            (def.enter)(&mut self.app, &mut self.state);
        }
        let _ = self.app.active(&id);
    }
}

/// Run the TUI until the user runs something or leaves, and give the state back
/// so the caller can read what was chosen.
pub fn run_tui(state: AppState, reader: &EventReader) -> Result<AppState, io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Drop events queued while the app was outside the TUI (e.g. keys pressed
    // during a foreground zapret run) so a fresh session starts clean.
    drain_events(&reader.rx());

    let result = pump(state, reader, &mut terminal);

    // The mouse has to be handed back explicitly: without this the shell that
    // starts next inherits a console that reports every mouse move.
    let _ = execute!(terminal.backend_mut(), DisableMouseCapture);
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();

    result
}

fn pump(
    state: AppState,
    reader: &EventReader,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<AppState, io::Error> {
    let mut model = Model::new(state, reader.shared());
    let mut job_owns_ports = false;

    let result = loop {
        if model.state.job.is_some() {
            if let Err(e) = tick_job(&mut model, reader, terminal) {
                break Err(e);
            }
            if model.state.job.is_some() {
                if !job_owns_ports {
                    job_owns_ports = lock_ports(&mut model);
                }
            } else if job_owns_ports {
                let _ = model.app.unlock_ports();
                job_owns_ports = false;
                // Keys pressed while the sweep was working must not open a menu
                // the user never aimed at.
                drain_events(&reader.rx());
            }
        } else {
            if let Err(e) = terminal.draw(|f| model.view(f)) {
                break Err(e);
            }
            let messages = model.app.tick(PollStrategy::UpTo(MAX_POLL, TICK)).unwrap_or_default();
            for msg in messages {
                let mut next = Some(msg);
                while let Some(m) = next {
                    next = model.update(m);
                }
            }
            if leaves(&model.pending) {
                break Ok(());
            }
            if let Err(e) = dispatch(&mut model, reader, terminal) {
                break Err(e);
            }
        }

        if leaves(&model.pending) {
            break Ok(());
        }
    };

    // Quitting is an outcome of the session, not of a screen: whichever row or
    // which key asked for it — the main menu's "Quit", `Msg::Quit`, or Esc with
    // an empty back stack — the caller has to hear about it, and it has to hear
    // about it from here rather than from whichever screen happened to set it.
    // Running is the other kind of outcome, and the caller reads the chosen
    // settings and carries on, so it leaves this flag alone.
    if matches!(model.pending, Some(Pending::Quit)) {
        model.state.should_quit = true;
    }
    result.map(|()| model.state)
}

/// Whether the session is done: the user asked to run something, or to leave.
fn leaves(pending: &Option<Pending>) -> bool {
    matches!(pending, Some(Pending::Run) | Some(Pending::Quit))
}

/// Give the cancel key to the loop that is looking for it.
///
/// The ports are locked first: the listener's worker is a second reader of the
/// same channel, and while the job owns the screen only this loop may take from
/// it.
fn lock_ports(model: &mut Model) -> bool {
    if model.app.lock_ports().is_err() {
        return false;
    }
    std::thread::sleep(PORT_GRACE);
    true
}

fn tick_job(
    model: &mut Model,
    reader: &EventReader,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), io::Error> {
    if let Some(job) = model.state.job.as_ref() {
        terminal.draw(|f| job.render(f))?;
    }
    if let Ok(Event::Key(key)) = reader.rx().recv_timeout(TICK) {
        if key.kind == KeyEventKind::Press && is_cancel(key.code) {
            if let Some(job) = model.state.job.as_ref() {
                job.request_cancel();
            }
        }
    }
    if model.state.job.as_ref().is_some_and(|j| j.is_finished()) {
        finish_job(model);
    }
    Ok(())
}

/// The keys that mean "stop", on a sweep screen. `q` and Esc, like everywhere
/// else; nothing else is read while a job owns the screen.
fn is_cancel(code: KeyCode) -> bool {
    matches!(code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
}

/// Apply what a finished sweep left behind, and get out of its way.
fn finish_job(model: &mut Model) {
    let Some(outcome) = model.state.job.as_mut().and_then(|j| j.take_outcome()) else {
        return;
    };
    model.state.job = None;
    model.state.autotune_running = false;

    match outcome {
        JobOutcome::Autotune(results, cancelled) => {
            let state = &mut model.state;
            state.autotune_results = Some(*results);
            state.has_autotune_results_file = true;
            state.dpi_desync_ttl = zapret_wrapper::config::load_ttl();
            // The report is one step further in from where the sweep was
            // started, so Esc from it lands back on the autotune menu.
            state.open(ActiveScreen::AutotuneResultsSubmenu);
            state.status_message = Some(if cancelled {
                rust_i18n::t!("autotune_cancelled").into_owned()
            } else {
                rust_i18n::t!("autotune_done").into_owned()
            });
            model.enter(ActiveScreen::AutotuneResultsSubmenu);
        }
        JobOutcome::Ttl(Ok(ttl)) => {
            let previous = {
                let state = &mut model.state;
                let _ = zapret_wrapper::config::save_ttl(Some(ttl));
                state.dpi_desync_ttl = Some(ttl);
                // Back to the row the sweep was started from, not to the top.
                state.back()
            };
            if let Some(previous) = previous {
                model.focus(previous);
            }
            model.state.status_message = Some(rust_i18n::t!("ttl_found").replace("{}", &ttl.to_string()));
        }
        JobOutcome::Ttl(Err(e)) => {
            model.state.show_error(e);
        }
    }
}

/// Start whatever the last message asked for.
fn dispatch(
    model: &mut Model,
    reader: &EventReader,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), io::Error> {
    let Some(pending) = model.pending.take() else {
        return Ok(());
    };
    match pending {
        Pending::DownloadZapret => to_child(model, terminal, |state, t, rx| {
            tasks::download::download_zapret(state, t, rx)
        }),
        Pending::DownloadStrategies => to_child(model, terminal, |state, t, rx| {
            tasks::download::download_strategies(state, t, rx)
        }),
        Pending::DownloadDefaults => to_child(model, terminal, |state, t, rx| {
            tasks::download::download_defaults(state, t, rx)
        }),
        Pending::OpenEditor(path) => to_child(model, terminal, |state, t, rx| {
            // The editor also needs the reader thread itself off the console, or
            // it competes with this crate for every keystroke.
            reader.pause();
            let out = tasks::edit::open_in_editor(state, &path, t, rx, reader);
            reader.resume();
            out
        }),
        Pending::StartAutotune => {
            model.state.job = Some(start_autotune(&model.state));
            Ok(())
        }
        Pending::StartTtl => {
            model.state.job = Some(start_ttl(&model.state));
            Ok(())
        }
        // Run and Quit are decided by the loop above, which has to leave the
        // terminal in an orderly state first.
        Pending::Run | Pending::Quit => Ok(()),
    }
}

/// Hand the terminal to a child program, then take it back.
///
/// The listener's port has to be locked for the duration: it is a second reader
/// of the same channel, and while a child owns the terminal this crate must not
/// be reading stdin at all.
fn to_child(
    model: &mut Model,
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    work: impl FnOnce(
        &mut AppState,
        &mut Terminal<CrosstermBackend<io::Stdout>>,
        &Receiver<Event>,
    ) -> Result<(), io::Error>,
) -> Result<(), io::Error> {
    let _ = model.app.lock_ports();
    std::thread::sleep(PORT_GRACE);
    let out = {
        let rx = model.rx.lock().unwrap_or_else(|e| e.into_inner());
        work(&mut model.state, terminal, &rx)
    };
    let _ = model.app.unlock_ports();
    out
}
