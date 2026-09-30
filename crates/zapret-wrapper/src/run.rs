//! Running zapret with a console and a log file attached.
//!
//! `zapret_core::daemon` knows how to start the process; this module is the
//! framing around it: it turns a [`RunRequest`] into a plan, runs it, and
//! reports what happened in the user's language and in
//! `<cache>/logs/zapret.log`.
//!
//! Everything here is presentation. Nothing here belongs in the kernel, and the
//! kernel knows nothing about it.

use crate::plan::RunRequest;
use zapret_core::daemon::{self, LaunchError, LaunchOutcome};
use zapret_core::error::ZResult;
use zapret_core::firewall::FirewallBackend;

/// Run zapret in the foreground, printing progress and writing the launch log.
///
/// A failed launch is reported on the console and in the log rather than
/// returned: the interactive caller has nothing to do with an error but print
/// it, and it must keep waiting for the user to stop the daemon either way.
pub fn run_foreground(req: &RunRequest, backend: &dyn FirewallBackend) {
    let plan = match crate::plan::plan(req) {
        Ok(p) => p,
        Err(e) => {
            println!("{}{}", rust_i18n::t!("err_parse_strat"), e);
            return;
        }
    };

    // Ensure user list files exist (original scripts create empty ones)
    crate::lists::ensure_user_lists();

    let mut term: Vec<String> = Vec::new();

    let start_msg = rust_i18n::t!("msg_start_nfqws").to_string();
    term.push(start_msg.clone());
    println!("{}", start_msg);

    let cmd_msg = format!("{}{}", rust_i18n::t!("msg_cmd"), plan.command());
    term.push(cmd_msg.clone());
    println!("{}", cmd_msg);

    let outcome = match launch(req, &plan, backend, &crate::paths::nfqws_output_log()) {
        Ok(o) => o,
        Err(e) => {
            let msg = describe_launch_error(&e);
            term.push(msg.clone());
            println!("{}", msg);
            // The firewall was already set up before the failure; leaving the
            // rules behind would divert traffic until the next clear.
            let _ = backend.clear();
            // A refused spawn still reached the log before this was split in
            // two; a missing binary or an unwritable scratch file did not.
            if matches!(e, LaunchError::Spawn(_)) {
                crate::diagnose::log_launch(&plan, &term);
            }
            return;
        }
    };

    if let Some(e) = &outcome.firewall_error {
        let msg = format!("{}{}", rust_i18n::t!("msg_err_firewall"), e);
        term.push(msg.clone());
        println!("{}", msg);
    }

    if outcome.setcap_failed {
        let msg = rust_i18n::t!("err_setcap").to_string();
        term.push(msg.clone());
        println!("{}", msg);
    }

    let run_msg = rust_i18n::t!("msg_nfqws_run").to_string();
    term.push(run_msg.clone());
    println!("{}", run_msg);

    if !outcome.output.is_empty() {
        term.push(outcome.output.clone());
        print!("{}", outcome.output);
    }

    crate::diagnose::log_launch(&plan, &term);
}

/// Run zapret without any output of its own, for the sweeps.
///
/// The caller reports progress; this only reports failure through `Err`.
///
/// The daemon's output is left in `capture` rather than thrown away. A sweep
/// that finds nothing has to be able to say *why* — and when winws rejects an
/// argument it prints one line and exits, which is the whole explanation.
pub fn run_quiet(req: &RunRequest, backend: &dyn FirewallBackend, capture: &std::path::Path) -> ZResult<LaunchOutcome> {
    let plan = crate::plan::plan(req).map_err(|e| format!("parse error: {}", e))?;
    crate::lists::ensure_user_lists();
    launch_quiet(req, &plan, backend, capture).map_err(|e| e.to_string())
}

/// The one place the launch signature's platform difference is spelled out.
///
/// `LaunchPlan::launch` takes the interface on Linux and does without it
/// everywhere else, so both call sites above would otherwise repeat a `cfg`.
fn launch(
    req: &RunRequest,
    plan: &daemon::LaunchPlan,
    backend: &dyn FirewallBackend,
    capture: &std::path::Path,
) -> Result<LaunchOutcome, LaunchError> {
    #[cfg(target_os = "linux")]
    {
        let router = crate::router::enabled();
        if router {
            report_router(crate::router::enable());
        }
        plan.launch(&req.interface, router, backend, capture)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = req;
        plan.launch(backend, capture)
    }
}

fn launch_quiet(
    req: &RunRequest,
    plan: &daemon::LaunchPlan,
    backend: &dyn FirewallBackend,
    capture: &std::path::Path,
) -> Result<LaunchOutcome, LaunchError> {
    #[cfg(target_os = "linux")]
    {
        let router = crate::router::enabled();
        if router {
            let _ = crate::router::enable();
        }
        plan.launch_quiet(&req.interface, router, backend, capture)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = req;
        plan.launch_quiet(backend, capture)
    }
}

#[cfg(target_os = "linux")]
fn report_router(result: Result<(), String>) {
    if let Err(e) = result {
        println!("{}{}", rust_i18n::t!("msg_err_router"), e);
    }
}

/// Whatever the daemon wrote while starting up, for a sweep to show or log.
pub fn read_launch_output(capture: &std::path::Path) -> String {
    std::fs::read_to_string(capture).unwrap_or_default()
}

/// True when something already holds the queue, so a launch would fight it.
///
/// Two questions, one answer, and the only place the second one is asked: the
/// daemon this program started is a handle, while a daemon somebody else started
/// — a managed service, a binary started by hand, a leftover of an earlier run
/// — is only knowable from the operating system.
pub fn queue_in_use() -> bool {
    daemon::is_running() || crate::platform::is_nfqws_running()
}

/// The two lines a stop produces, in the order they are produced.
fn stop_terms() -> Vec<String> {
    vec![
        rust_i18n::t!("msg_zapret_stop").to_string(),
        rust_i18n::t!("msg_zapret_clear").to_string(),
    ]
}

/// Stop the daemon, reporting it on the console and in the log.
pub fn stop(backend: &dyn FirewallBackend) {
    let terms = stop_terms();
    for msg in &terms {
        println!("{}", msg);
    }
    daemon::stop(backend);
    stop_router();
    crate::diagnose::log_stop(&terms);
}

/// Stop the daemon without saying so, for the sweeps.
///
/// The counterpart of [`run_quiet`]: a sweep paints its own progress and the
/// console is not where it reports, so a stop in the middle of one has to stay
/// quiet or it lands on top of the bar.
pub fn stop_quiet(backend: &dyn FirewallBackend) {
    let terms = stop_terms();
    daemon::stop(backend);
    stop_router();
    crate::diagnose::log_stop(&terms);
}

/// Hand the network back the way it was found. No-op without router mode.
#[cfg(target_os = "linux")]
fn stop_router() {
    if crate::router::enabled() {
        crate::router::disable();
    }
}

#[cfg(not(target_os = "linux"))]
fn stop_router() {}

fn describe_launch_error(e: &LaunchError) -> String {
    match e {
        LaunchError::BinaryMissing(p) => rust_i18n::t!("err_bin_miss").replace("{:?}", &format!("{:?}", p)),
        LaunchError::CaptureFile(_, _) => "failed to create temp log file".to_string(),
        LaunchError::Spawn(e) => format!("{}{}", rust_i18n::t!("err_start_nfqws"), e),
        // Unreachable through run_foreground, which uses launch(): a refused
        // firewall is a warning there. Handled so the match stays exhaustive.
        LaunchError::Firewall(e) => format!("{}{}", rust_i18n::t!("msg_err_firewall"), e),
    }
}
