# Service management

`zapret-core::service` installs, starts, stops and removes zapret-rust as a
system service. One `ServiceManager` implementation per init system, one
factory to pick the right one.

## The trait

```rust
pub trait ServiceManager: Send + Sync {
    fn is_installed(&self) -> bool;
    fn is_active(&self) -> bool;
    fn install(&self, exe_path: &Path, config_path: &Path, cache_dir: &Path) -> Result<(), String>;
    fn uninstall(&self) -> Result<(), String>;
    fn start(&self) -> Result<(), String>;
    fn stop(&self) -> Result<(), String>;
    fn restart(&self) -> Result<(), String>;
}
```

`is_installed` and `is_active` return `bool` because the UI polls them after
every key press and has nothing useful to do with an error string. The other
five report why they failed.

## Layout

```
mod.rs        the trait, get_detected_manager(), get_manager()
detect.rs     InitType and detect_init_system()
script.rs     the <exe> --config <cfg> --cache-dir <dir> command line, built once
sysv.rs   systemd.rs   openrc.rs   runit.rs   s6.rs   dinit.rs
windows.rs    the Windows SCM client
```

`script.rs` exists because every init system has to launch the same binary with
the same three paths but writes them in its own syntax (`ExecStart=`, `exec`,
`command_args=`). The path conversion and the argument tail are identical
everywhere, so they are built once and the managers supply only the syntax
around them.

A new init system is: a struct, the six trait methods, and a `cfg`-gated arm in
`get_manager`. It does not need to touch `detect.rs` unless detection has to
learn a new way to spot it.

## Windows: two halves

The Windows service is split, because the two halves are different kinds of
code:

- `service::windows::WindowsServiceManager` — the SCM client. Installs, starts,
  stops and removes the service.
- `src/daemon.rs` in the binary — the runtime. What the SCM actually starts:
  it registers the control handler, reads `--config` and `--cache-dir`, and
  boots the application.

The runtime is a process entry point, not a service manager, so it does not
belong in the service layer of the domain crate. Before this split a single file
held both, which made the service layer a second way into the whole application.

## The installed service

The generated unit always launches the same command:

```
<exe> --config <cache>/conf.env --cache-dir <cache>
```

`--cache-dir` is always passed explicitly so the service does not depend on the
environment it was started from.
