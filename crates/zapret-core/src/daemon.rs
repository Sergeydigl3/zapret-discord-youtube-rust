//! Process lifecycle for the zapret daemon.
//!
//! This is the launch kernel: given a fully resolved [`LaunchPlan`] it puts the
//! firewall in place, grants the capability the queue needs, spawns
//! `nfqws` / `winws`, keeps track of the child and tears everything down again.
//!
//! It deliberately knows nothing about strategies. It does not parse a `.bat`,
//! does not know what an alias or a list is, does not read the configuration
//! file, does not resolve a single path on its own and does not print or log a
//! single line. Everything it needs arrives in the plan, everything it observes
//! leaves as an outcome.

use crate::firewall::FirewallBackend;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

static NFQWS_PROCESSES: Mutex<Vec<Child>> = Mutex::new(Vec::new());

/// Everything the runner needs to start the daemon.
///
/// The caller resolves the binary, the working directory and the whole command
/// line, and reads the port ranges out of whatever it used to build them. That
/// is the seam: the kernel never has to learn what a strategy is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaunchPlan {
    /// `nfqws` or `winws.exe`.
    pub binary: PathBuf,
    /// Directory the daemon runs in. Relative `bin/` and `lists/` paths baked
    /// into a command line resolve against it.
    pub work_dir: PathBuf,
    /// The complete argument vector, platform head already applied.
    pub args: Vec<String>,
    /// Port range the firewall backend has to divert.
    pub tcp_ports: String,
    /// Port range the firewall backend has to divert.
    pub udp_ports: String,
}

/// Why a launch did not produce a running daemon.
#[derive(Debug)]
pub enum LaunchError {
    /// The runtime binary is not on disk where the plan said it would be.
    BinaryMissing(PathBuf),
    /// The scratch file the startup output is captured into could not be made.
    CaptureFile(PathBuf, String),
    /// `Command::spawn` refused.
    Spawn(String),
    /// The firewall backend refused the rules. Only [`LaunchPlan::launch_quiet`]
    /// reports this; [`LaunchPlan::launch`] treats it as a warning.
    Firewall(String),
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LaunchError::BinaryMissing(p) => write!(f, "binary not found: {:?}", p),
            LaunchError::CaptureFile(p, e) => write!(f, "failed to create capture file {:?}: {}", p, e),
            LaunchError::Spawn(e) => write!(f, "failed to start nfqws: {}", e),
            LaunchError::Firewall(e) => write!(f, "firewall setup error: {}", e),
        }
    }
}

/// What a launch did, as far as the caller is concerned.
#[derive(Debug, Default)]
pub struct LaunchOutcome {
    /// `"<binary> <args…>"`, ready for a console line or a log section.
    pub command: String,
    /// What the daemon printed while starting up. Empty in [`LaunchPlan::launch_quiet`].
    pub output: String,
    /// The firewall could not be set up, and the daemon was started anyway.
    pub firewall_error: Option<String>,
    /// `CAP_NET_ADMIN` could not be granted, and the daemon was started anyway.
    pub setcap_failed: bool,
}

impl LaunchPlan {
    /// The command line as a single string, for display and for the log.
    pub fn command(&self) -> String {
        format!("{:?} {:?}", self.binary, self.args)
    }

    /// Set the firewall up, spawn the daemon and report what it printed.
    ///
    /// The daemon's startup output is captured through `capture`, a scratch file
    /// the caller picks — it lives in the application cache rather than next to
    /// the daemon so that Windows Defender does not quarantine it. The file is
    /// removed before returning.
    ///
    /// A firewall that refuses the rules is reported through
    /// [`LaunchOutcome::firewall_error`] instead of aborting: the rules may
    /// already be in place from an earlier run, and the daemon is worth starting
    /// either way.
    pub fn launch(
        &self,
        interface: &str,
        backend: &dyn FirewallBackend,
        capture: &Path,
    ) -> Result<LaunchOutcome, LaunchError> {
        let mut outcome = LaunchOutcome {
            command: self.command(),
            ..Default::default()
        };

        if let Err(e) = backend.setup(&self.tcp_ports, &self.udp_ports, interface) {
            outcome.firewall_error = Some(e);
        }

        crate::process::kill_stale_zapret();

        if !self.binary.exists() {
            return Err(LaunchError::BinaryMissing(self.binary.clone()));
        }

        // Set CAP_NET_ADMIN on the binary so it can use nfqueue without root.
        outcome.setcap_failed = !crate::process::set_cap(&self.binary);

        // Capture the daemon output to a temp file so it can be shown once the
        // process has had a moment to say something.
        if let Some(parent) = capture.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let output_file =
            fs::File::create(capture).map_err(|e| LaunchError::CaptureFile(capture.to_path_buf(), e.to_string()))?;
        let err_dup = output_file
            .try_clone()
            .map_err(|e| LaunchError::CaptureFile(capture.to_path_buf(), e.to_string()))?;

        let child = Command::new(&self.binary)
            .args(&self.args)
            .current_dir(&self.work_dir)
            .stdin(Stdio::null())
            .stdout(output_file)
            .stderr(err_dup)
            .spawn()
            .map_err(|e| LaunchError::Spawn(e.to_string()))?;

        if let Ok(mut procs) = NFQWS_PROCESSES.lock() {
            procs.push(child);
        }

        // Wait briefly for the startup output before reading it back.
        thread::sleep(Duration::from_millis(300));
        outcome.output = fs::read_to_string(capture).unwrap_or_default();
        let _ = fs::remove_file(capture);

        Ok(outcome)
    }

    /// Start the daemon without capturing or printing anything.
    ///
    /// Used by the sweeps, which run dozens of launches in a row and report
    /// their own progress. A firewall that refuses the rules is fatal here,
    /// because a strategy test measured against no rules proves nothing.
    pub fn launch_quiet(&self, interface: &str, backend: &dyn FirewallBackend) -> Result<LaunchOutcome, LaunchError> {
        backend
            .setup(&self.tcp_ports, &self.udp_ports, interface)
            .map_err(LaunchError::Firewall)?;

        crate::process::kill_stale_zapret();

        if !self.binary.exists() {
            return Err(LaunchError::BinaryMissing(self.binary.clone()));
        }

        let _ = crate::process::set_cap(&self.binary);

        let child = Command::new(&self.binary)
            .args(&self.args)
            .current_dir(&self.work_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| LaunchError::Spawn(e.to_string()))?;

        if let Ok(mut procs) = NFQWS_PROCESSES.lock() {
            procs.push(child);
        }

        Ok(LaunchOutcome {
            command: self.command(),
            ..Default::default()
        })
    }
}

/// Returns true if any spawned zapret process is still running.
pub fn is_running() -> bool {
    let Ok(mut procs) = NFQWS_PROCESSES.lock() else {
        return false;
    };
    procs
        .iter_mut()
        .any(|p| p.try_wait().map(|s| s.is_none()).unwrap_or(false))
}

/// Kill the daemons this process started and drop the firewall rules.
pub fn stop(backend: &dyn FirewallBackend) {
    if let Ok(mut procs) = NFQWS_PROCESSES.lock() {
        for child in procs.iter_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
        procs.clear();
    }

    let _ = backend.clear();
}
