# Architecture

`zapret-rust` is a single binary split into three packages. The split exists so
that the compiler, not a code review, keeps the boundaries intact.

```
zapret-rust (bin)   src/        bootstrap only: locale, --service branch,
                                  console, CLI parsing, then app::run
zapret-core (lib)   crates/zapret-core/    the domain
zapret-tui  (lib)   crates/zapret-tui/     the interface
```

`zapret-tui` depends on `zapret-core` and exports nothing back. The binary
depends on both. Neither library depends on the binary.

## Rules

1. **Dependencies point down only.** A module may use the layers beneath it and
   nothing above. There are no cycles anywhere in `zapret-core`, and a layer
   table is part of its `lib.rs` so the rule is visible from the entry point.
2. **One owner per concern.** A decision lives in exactly one module. Every
   other module asks that one; it is not re-implemented elsewhere.
3. **The lowest layer knows nothing about the domain.** `paths` and `process`
   know about the filesystem and about programs. They do not know what a
   strategy, a list or an init system is.
4. **The interface layer does not do I/O of its own** beyond the terminal. It
   calls the domain and renders the result.

## Layers of `zapret-core`

```
L4  autotune            the auto-tuning feature, composed from everything below
L3  net, run, service, diagnose      actions: probing, running, installing, logging
L2  download, firewall, lists, strategy   what gets installed and how the network is described
L1  config, defender, domains, fakes, platform   configuration and OS access
L0  error, i18n, paths, process      no dependencies of their own
```

| Module | Owns |
|---|---|
| `error` | The `ZResult` alias — the crate's error contract in one place |
| `i18n` | Locale detection, shared by all three packages so they agree on one language |
| `paths` | **Every** path in the program: cache, repository, both `bin` directories, config, logs, and the install-state predicates |
| `process` | Every external program: capabilities, stale-daemon cleanup, the null device |
| `config` | `RunConfig` and nothing else — reading, writing and schema migration of the config file |
| `platform` | Talking to the OS: privileges, running-process detection, interface enumeration |
| `domains` | The domain list files. Shared by autotune and the TTL sweep, which is what keeps those two from forming a cycle |
| `fakes` | The `.bin` payloads a strategy references, and which one is active |
| `defender` | The Windows Defender exclusion (Windows only) |
| `download` | Fetching zapret and the strategies: release lookup, unpacking, installing. Split into `platform_matrix`, `github`, `archive`, `state` |
| `firewall` | The `FirewallBackend` trait, the nftables/iptables backends and the WinDivert stub |
| `lists` | The list files (`ipset-all.txt` and friends): the directory, the available modes, and applying one |
| `strategy` | `.bat` parsing, the bundled strategies, and discovery of what is on disk |
| `net` | Reachability probes that know nothing about autotune: DNS, TCP, TLS, QUIC |
| `run` | The daemon lifecycle: build the arguments, spawn, track, stop |
| `service` | One `ServiceManager` per init system. The managers are dumb: they write a unit and shell out |
| `diagnose` | The launch log: system facts, timestamp, and writing the file |
| `autotune` | The feature itself: configuration, results, the orchestrator |

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
session then runs tasks::<job>, which takes the terminal over and gives it back.
```

| Module | Owns |
|---|---|
| `event` | The one console reader thread for the process, plus draining it and waiting for a key |
| `screen` | Raw mode, the alternate screen, and handing the terminal to a child program |
| `draw` | One frame. Pure rendering |
| `session` | The frame loop and the key dispatch |
| `state` | `AppState` — data, the refreshes, and the screens in `state::screens` |
| `state::actions` | One handler per screen. `actions/mod.rs` only routes |
| `menus` | The rows each screen draws. No logic |
| `tasks` | The jobs that take the terminal over: downloads, the editor, autotune, the TTL sweep |
| `editor` | Choosing an editor from `$EDITOR` and falling back |

Adding a screen means four edits: a variant in `state::screens`, a handler in
`state::actions`, a render branch in `draw`, and a menu module.

## Adding a module

Pick the layer first. If a new module needs something from a layer above it, the
boundary is wrong: move the shared part down instead. If two modules need the
same thing, it belongs to the layer below both.

Before adding to the process or path world, check `paths` and `process` — most
of what looks new is a variant of something already there.
