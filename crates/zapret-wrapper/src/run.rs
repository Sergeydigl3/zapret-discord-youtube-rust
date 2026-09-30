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

    let outcome = match plan.launch(&req.interface, backend, &crate::paths::nfqws_output_log()) {
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
pub fn run_quiet(req: &RunRequest, backend: &dyn FirewallBackend) -> ZResult<LaunchOutcome> {
    let plan = crate::plan::plan(req).map_err(|e| format!("parse error: {}", e))?;
    crate::lists::ensure_user_lists();
    plan.launch_quiet(&req.interface, backend).map_err(|e| e.to_string())
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

/// Stop the daemon, reporting it on the console and in the log.
pub fn stop(backend: &dyn FirewallBackend) {
    let mut term: Vec<String> = Vec::new();

    let msg = rust_i18n::t!("msg_zapret_stop").to_string();
    term.push(msg.clone());
    println!("{}", msg);

    daemon::stop(backend);

    let msg = rust_i18n::t!("msg_zapret_clear").to_string();
    term.push(msg.clone());
    println!("{}", msg);

    crate::diagnose::log_stop(&term);
}

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
