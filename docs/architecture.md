# Architecture

`zapret-rust` is a single binary split into five packages. The split exists so
that the compiler, not a code review, keeps the boundaries intact.

```
zapret-rust (bin)   src/        bootstrap only: locale, --service branch,
                                  console, CLI parsing, then app::run
zapret-core (lib)   crates/zapret-core/    the launch kernel
zapret-wrapper(lib) crates/zapret-wrapper/ everything zapret-shaped
zapret-fetch (lib)  crates/zapret-fetch/   downloading dependencies
zapret-tui  (lib)   crates/zapret-tui/     the interface
```

Dependencies point in one direction only:

```
zapret-tui ──▶ zapret-wrapper ──▶ zapret-core
     │                  │
     └──────────────────┴──────▶ zapret-fetch
```

`zapret-fetch` depends on neither `zapret-core` nor `zapret-wrapper`: it is handed
an `InstallTargets` and puts files on disk. Nothing depends on the binary.

## The seam: `LaunchPlan`

The whole split turns on one data type. `zapret-core` does not know what a
strategy is; `zapret-wrapper` does not know how a process is spawned. The line
between them is `zapret_core::daemon::LaunchPlan`:

```rust
pub struct LaunchPlan {
    pub binary: PathBuf,   // nfqws or winws.exe
    pub work_dir: PathBuf, // the daemon's cwd: bin/ and lists/ resolve against it
    pub args: Vec<String>, // the complete argv, platform head already applied
    pub tcp_ports: String, // the range the firewall has to divert
    pub udp_ports: String,
}
```

Everything below the line takes a plan and returns a `LaunchOutcome` (or a
`LaunchError`). Everything above the line builds the plan. Consequences worth
knowing:

- `zapret-core` has no `paths` module. It resolves no file and reads no
  configuration. Its only path is the scratch file the caller passes to `launch`
  to capture the daemon's startup output.
- `zapret-core` has no i18n. It prints nothing. Every console line and the whole
  `zapret.log` come from `zapret_wrapper::run` and `zapret_wrapper::diagnose`.
- A firewall that refuses the rules is a *warning* in `LaunchPlan::launch` and
  *fatal* in `LaunchPlan::launch_quiet`. That asymmetry is deliberate and old: an
  interactive run may find the rules already in place from an earlier run, while
  a strategy test measured against no rules would prove nothing.

## Rules

1. **Dependencies point down only.** A module may use the layers beneath it and
   nothing above. There are no cycles anywhere, and a layer table is part of each
   crate's `lib.rs` so the rule is visible from the entry point. Across crates
   the arrow above is the same rule.
2. **One owner per concern.** A decision lives in exactly one module. Every
   other module asks that one; it is not re-implemented elsewhere.
3. **The lowest layer knows nothing about the domain.** `zapret_core::process`
   and `zapret_core::daemon` know about a process and a queue. They do not know
   what a strategy, a list or a config file is. `zapret_wrapper::paths` knows
   about the filesystem and not what the files mean.
4. **The interface layer does not do I/O of its own** beyond the terminal. It
   calls the domain and renders the result.
5. **A platform difference is a function, not a call site.** Whatever the OS
   changes, the caller asks for the operation — "kill this image", "which
   firewall backend", "what time is it" — and one definition of that function
   knows about `pkill` versus `taskkill`. `#[cfg]` inside a function body
   (`cfg!`, one tail expression) is how that is written; two `#[cfg]` blocks
   around the same call in a caller is the duplication this replaces. Type-level
   differences stay where they are: a field or an enum variant that only exists
   on one OS is data, not a decision.
6. **A process this program started is a handle, not a name.** `daemon` keeps
   the `Child` it spawned, and `daemon::is_running` / `daemon::stop` answer
   through it. Nothing scans the machine to find out whether our own zapret runs.
   The scan (`platform::is_nfqws_running`, reached only through
   `run::queue_in_use`) exists for the one question a handle cannot answer: is
   somebody *else* — a managed service, a binary started by hand, a leftover of
   an earlier run — holding the queue.

## Layers of `zapret-core`

```
L2  daemon            start, track, stop
L1  firewall, process the network description and the two launch prerequisites
L0  error             the error contract in one place
```

| Module | Owns |
|---|---|
| `error` | The `ZResult` alias |
| `firewall` | The `FirewallBackend` trait, the nftables/iptables backends and the WinDivert stub |
| `process` | A process *by name*, for the ones this program does not own: the daemon image (`winws.exe` / `nfqws`), killing leftovers, and asking whether somebody else's daemon runs. The one it started itself is a child handle in `daemon` |
| `daemon` | `LaunchPlan`, `LaunchError`, `LaunchOutcome`, and the child handle of the daemon it started — the single source of truth for "is zapret running" and the only way it is stopped |

`firewall` carries a `build.rs` that generates `firewall/backends/_backends.rs`
from the files in `firewall/backends/`, keeping `nftables` first so it stays the
default. The generated file is `include!`d from the same directory, so the
generator and the backends must not be separated.

## Layers of `zapret-wrapper`

```
L4  autotune            the auto-tuning sweep, probes included
L3  run, plan, service, diagnose   actions: running, installing a service, logging
L2  lists, strategy     how zapret describes the network on disk
L1  config, defender, domains, fakes, platform   configuration and OS access
L0  paths               every path in the program
```

| Module | Owns |
|---|---|
| `paths` | **Every** path: cache, repository, both `bin` directories, config, logs, and the install-state predicates |
| `config` | `RunConfig` and nothing else — reading, writing and schema migration of the config file |
| `platform` | Talking to the OS: privileges, running-process detection, interface enumeration, the null device. Detection is a name; the `pgrep` / `tasklist` spelling is `zapret_core::process` |
| `domains` | The domain list files. Shared by autotune and the TTL sweep, which is what keeps those two from forming a cycle |
| `fakes` | The `.bin` payloads a strategy references, and which one is active |
| `defender` | The Windows Defender exclusion (Windows only) |
| `lists` | The list files (`ipset-all.txt` and friends): the directory, the available modes, applying one, and the user lists a run expects to exist |
| `strategy` | `.bat` parsing, the bundled strategies, discovery of what is on disk, name resolution, and the `.bat` to argv conversion in `strategy::args` |
| `plan` | `RunRequest` and its resolution into a `LaunchPlan` |
| `run` | Running a request: console output and the launch log |
| `service` | One `ServiceManager` per init system. The managers are dumb: they write a unit and shell out |
| `diagnose` | The launch log: system facts, timestamp, and writing the file |
| `autotune` | The whole sweep: the probes it measures with, the checks, the orchestrator and the results file |

### A zone needs a second consumer

Splitting a large file into focused files *inside* a zone is cheap and always
worthwhile. Promoting a group of files to a zone of their own is a stronger
claim, and it needs a second consumer — otherwise it is a layer that only one
caller stands on, and the next person will wire around it.

`autotune` is the worked example. Its probes (`dns`, `quic`, `probe`,
`checks_network`, `checks_domain`, `cancel`) used to sit in a top-level `net`
module presented as shared reachability infrastructure. Nothing outside the sweep
used them, and they reported in the sweep's own result types, so `net` was a
cycle wearing a layer's clothes. They now live in `autotune/`, where the single
consumer is honest.

The same test is why `domains` *is* a zone: the autotune sweep and the TTL sweep
both need the domain list files, and hoisting them is what stopped those two from
depending on each other.

### Strategy names are resolved in one place

`strategy::resolve` is the only function that turns a strategy *name* into a
file. It checks `<cache>/custom-strategies` before `<repo>/custom-strategies`
before `<repo>`, so a user strategy survives a repository re-download. Discovery
lists the names, the TTL sweep lists the name, autotune lists the name — and all
of them end up at the same resolver. Do not add a second copy; one existed once
and autotune ended up skipping user overrides the ordinary run honoured.

### Two directories called `bin`

Both exist on disk and both are reachable through `paths`:

- `paths::bin_runtime_dir()` — `<cache>/bin`, where `nfqws` / `winws.exe` is installed.
- `paths::bin_assets_dir()` — `<repo>/bin`, the `.bin` payload files a strategy refers to.

### `--cache-dir` and the list files

`paths::repo_dir()` and `paths::exe_relative_lists_dir()` are deliberately
different. The first honours `ZAPRET_CACHE_DIR`; the second resolves relative to
the executable only, which is what the list editor and the ipset modes use. So
`--cache-dir` does not reach the lists. That is the current behaviour, kept as
is; the two names exist so the difference is visible instead of accidental.
Making `exe_relative_lists_dir()` delegate to `repo_dir()` is a one-line change
and belongs in its own `fix(paths)` commit.

## The TUI

```
key ──▶ session::handle_key ──▶ state::actions::{on_activate, on_cycle, on_back}
                                   │
                                   └──▶ actions::<screen>   one file per screen

a screen that starts a long job only raises a should_* flag on AppState;
session then runs tasks::<job>.
```

| Module | Owns |
|---|---|
| `event` | The one console reader thread for the process, plus draining it and waiting for a key |
| `screen` | Raw mode, the alternate screen, and handing the terminal to a child program |
| `draw` | One frame. Pure rendering |
| `views` | The screens that own their whole area: a sweep's progress bar, and the report's short and long forms |
| `session` | The frame loop and the key dispatch. Two modes: menus, or a running job |
| `jobs` | A sweep on a worker thread, and the cancel flag the UI sets on it |
| `state` | `AppState` — data, the refreshes, and the screens in `state::screens` |
| `state::actions` | One handler per screen. `actions/mod.rs` only routes |
| `menus` | The rows each screen draws. No logic |
| `tasks` | The jobs that hand the terminal to a child program: downloads and the editor |
| `editor` | Choosing an editor from `$EDITOR` and falling back |

The TUI depends on all three libraries: `zapret-core` for the firewall backend
it lets the user pick, `zapret-wrapper` for everything else, `zapret-fetch` for
the downloads. The only thing it takes from `zapret-core` is `firewall`; if a
second `zapret-core` import appears, the code probably belongs in the wrapper.

Adding a screen means four edits: a variant in `state::screens`, a handler in
`state::actions`, a render branch in `draw`, and a menu module. A screen that is
a table or a bar rather than a list of rows skips the last two and gets a
`views` module instead.

### Three kinds of job

`tasks` and `jobs` split on where the work runs, and the split is about the
terminal:

- A job that runs a **child program** — a download, `$EDITOR` — hands the
  terminal over (`screen::begin_external_output`), lets the child and its own
  `println!` own the console, and takes it back afterwards. These live in
  `tasks`.
- A job that runs **in-process work** — the autotune sweep, the TTL sweep —
  keeps the terminal and paints frames, and it runs on **its own thread**
  (`jobs::Job`). The frame loop keeps repainting on its 50 ms tick and reads
  only the cancel key, so the bar, the spinner and the elapsed clock keep
  moving through a step that takes thirty seconds. A sweep that blocked the
  loop's thread would freeze exactly when it has the most to say.
- Everything else is a menu.

A sweep on a worker thread cannot print, which is the other half of why it does
not: it reports through an event callback (`autotune::SweepEvent`,
`domains::TtlEvent`), and the wrapper helpers it calls have quiet modes —
`run_quiet` / `stop_quiet`, and `firewall::QuietGuard` for the backends whose
`setup` / `clear` notices would otherwise land between two frames.

The rules that fall out:

- **Inside the TUI, nothing prints to stdout.**
- `FirewallBackend: Send + Sync`, because a backend reference crosses into the
  worker thread. Every backend is a unit struct or a fieldless enum, so this
  costs nothing.
- The daemon and the firewall are process-wide singletons. Only one sweep may
  run at a time, which the single `AppState::job` slot already guarantees.


## Localization

One flat `locales/` directory, four `rust_i18n::i18n!` invocations — one per
crate, all at the crate root, because `t!` reaches the generated table through
`crate::_rust_i18n_t` and that only exists at the root. `rust_i18n::set_locale`
is global, so the binary sets it once in `src/main.rs` before anything runs and
every crate's table follows.

The `../..` in the macro path is depth-coupled to `crates/<name>/`: a crate moved
deeper, or out of `crates/`, will not find the locale files.

## Adding a module

Pick the layer first. If a new module needs something from a layer above it, the
boundary is wrong: move the shared part down instead. If two modules need the
same thing, it belongs to the layer below both — or, if the two modules are on
opposite sides of the `LaunchPlan` seam, in one of the two crates that meet
there.

Before adding to the process or path world, check `zapret_wrapper::paths` and
`zapret_core::process` — most of what looks new is a variant of something already
there.
