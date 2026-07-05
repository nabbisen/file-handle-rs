# RFC 002 — Terminal availability probe

**Status.** Implemented in 0.4.0
**Tracks.** Best-effort terminal capability detection for callers that want to
shape UI or fallback behavior before calling `open_terminal`.
**Touches.** `src/file_handle.rs`, platform dispatch modules under
`src/file_handle/`, source-level tests under `src/file_handle/`, README/API
documentation.

## Summary

Add a feature-gated `FileHandle::terminal_availability()` method under the
existing `terminal` feature. The method returns an advisory availability enum
indicating whether `file-handle` can currently find enough evidence that a
terminal launch is worth trying for the current process environment.

This is a convenience API for callers, especially GUI applications that want to
disable or hide an "Open terminal here" action before the user clicks. It is not
a correctness boundary: callers must still handle `open_terminal` failure because
availability can change between probing and launching.

## Motivation

`file-handle` is usable from CLI apps, GUI apps, and headless environments.
However, `open_terminal` is specifically a desktop/OS-handler action. A terminal
emulator cannot be assumed in all environments.

Examples where terminal launch may be unavailable:

- headless Linux without a graphical session;
- CI, containers, servers, or SSH/service processes;
- minimal desktop installs without a supported terminal binary;
- Linux desktops where the user only has a terminal outside the current fixed
  candidate list;
- sandboxed applications where host process launching is restricted.

RFC 001 made unavailable handlers explainable via `NoHandlerAvailable`. That is
the correct runtime behavior. This RFC adds a lightweight pre-check for callers
that can use it to improve ergonomics.

## Goals

1. Add `FileHandle::terminal_availability() -> Availability` behind
   `feature = "terminal"`.
2. Keep the method best-effort and explicitly racy.
3. Reuse the same terminal candidate logic as `open_terminal` where practical.
4. Avoid adding dependencies or a broad probing framework.
5. Keep `open_terminal` as the authoritative operation.

## Non-goals

This RFC does not add:

- `open_terminal_all`;
- preferred terminal override;
- environment-variable terminal configuration;
- Linux `show` fallback behavior;
- probing APIs for `open`, `show`, or `trash`;
- async probing;
- guarantees that a later `open_terminal` call will succeed.

Preferred terminal override is related but separate. If pursued, it should be a
future RFC because it changes candidate selection policy rather than simply
exposing current availability.

## Public API

Add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Availability {
    Available,
    Unavailable,
    Unknown,
}

#[cfg(feature = "terminal")]
impl FileHandle {
    pub fn terminal_availability() -> Availability;
}
```

`Availability` is always exported once this RFC lands. The
`terminal_availability` method remains gated by `feature = "terminal"`.
`Availability` is intentionally exhaustive with the three variants above.
Adding another variant later is a breaking API change unless a future RFC
replaces this enum with a richer type.

The method returns:

- `Availability::Available` if `file-handle` can currently identify a plausible
  terminal launcher and the environment is not clearly unsuitable;
- `Availability::Unavailable` if no supported launcher can be identified, or if
  the environment is clearly unsuitable for launching a terminal;
- `Availability::Unknown` if the platform probe cannot make a useful advisory
  decision without doing work that is out of scope.

The method must not return `Result`. Detailed runtime failure remains the
responsibility of `open_terminal`.

## Behavior

### General Contract

`terminal_availability()` is best-effort. `Availability::Available` means
"trying `open_terminal` is reasonable." It does not guarantee that a terminal
will actually open.

`Availability::Unavailable` means "the current environment does not appear to
have a usable terminal launcher according to `file-handle`'s current probing
rules." A caller may hide a button, disable a menu item, print a fallback
instruction, or still try `open_terminal`.

`Availability::Unknown` means the probe cannot make a reliable advisory
decision. Callers should normally keep the action available but still handle
`open_terminal` errors.

Current platform policy:

- on supported platform backends with shallow evidence available, return
  `Available` or `Unavailable`;
- return `Unknown` when the backend cannot make a meaningful decision without
  side effects, unsupported target behavior, unavailable inspection APIs, or
  intentionally skipped environment checks;
- do not use `Unknown` merely because a later launch might race or fail, because
  that caveat applies to every availability value.

`terminal_availability()` is global to the current process environment, not
path-specific. It does not validate a target path, parent directory,
permissions, sandbox access, or working-directory usability.

`Availability::Unavailable` may be a false negative. It means only that
`file-handle` did not find a usable terminal according to its current
conservative probing rules. Applications should not treat it as a permanent
policy decision or a security decision.

Callers must continue to handle:

```rust
FileHandleError::NoHandlerAvailable { .. }
```

from `open_terminal`.

### Linux / Unix

The probe checks the same candidate terminal commands currently used by
`open_terminal`:

```text
xdg-terminal-exec
gnome-terminal
konsole
xterm
```

Return `Available` if any candidate can be found on `PATH` and the environment
is not clearly headless. Return `Unavailable` otherwise.

The probe may return `Unavailable` when the environment clearly lacks a
graphical session, such as both `DISPLAY` and `WAYLAND_DISPLAY` being absent.
This check is best-effort and should not grow into desktop-session detection
machinery.

The headless check is intentionally narrow. This RFC does not inspect D-Bus,
portals, process ancestry, SSH variables, compositor-specific state, or other
desktop/session internals.

SSH X11 forwarding, Wayland-only sessions, sandboxed desktops, wrappers, portals,
and nonstandard terminal emulators can produce false positives or false
negatives. Callers may still choose to show a manual retry action.

`Available` means a candidate was found according to normal process-environment
lookup rules. It does not mean the candidate is trusted for privileged,
elevated, setuid, service, or sandbox-broker execution contexts.

The probe must not spawn a terminal merely to test availability. It should check
candidate executability/discoverability only.

### macOS

Return `Available` if the system `open` launcher is discoverable. The probe
should prefer the known system launcher path, normally `/usr/bin/open`, over an
arbitrary PATH-shadowed `open` when practical. Return `Unavailable` if the
system launcher is not discoverable.

Generic `PATH` lookup for `open` is not required and should not shadow the
system launcher by default. A PATH-shadowed `open` is not required for
`Available`.

`open_terminal` uses:

```text
open -a Terminal
```

The probe does not need to verify that Terminal.app can fully launch. A later
`open_terminal` call remains authoritative.

### Windows

Return `Available` when the Windows terminal launcher used by `open_terminal`
can be resolved by the same backend logic. The probe should prefer a trusted
`cmd.exe` resolution strategy, such as a valid `%ComSpec%` or
`%SystemRoot%\System32\cmd.exe`, rather than arbitrary shell command
construction.

Trusted resolution order:

1. prefer a valid absolute `%ComSpec%` if it points to a file;
2. otherwise use `%SystemRoot%\System32\cmd.exe` if `%SystemRoot%` is valid;
3. do not resolve `cmd.exe` through arbitrary `PATH` unless a future RFC defines
   the security model;
4. return `Unavailable` when a trusted candidate cannot be resolved, unless the
   backend cannot inspect the environment at all, in which case return
   `Unknown`.

The probe does not need to verify shell policy, desktop session state, or whether
process launching is restricted. A later `open_terminal` call remains
authoritative.

The whole-library design identifies `cmd /C start` as design debt. This RFC must
not bless it as the durable Windows terminal-launch contract.

## Implementation Notes

The implementation should use small platform-local helpers. Do not add a new
dependency for `which`-style lookup; implement the needed path search with the
standard library.

Path lookup helper contract:

- accept an explicit search path parameter for deterministic tests;
- do not mutate process-global environment variables in tests;
- ignore empty, invalid, and non-directory path entries safely;
- return unavailable for directories and non-executable files;
- on Unix, require executable permission bits where practical;
- on Windows, handle `.exe` / `PATHEXT` intentionally, or avoid generic PATH
  probing for `cmd.exe` by using the trusted resolution strategy above;
- never spawn candidates during probing.

Suggested internal shape:

```rust
#[cfg(feature = "terminal")]
pub fn terminal_availability() -> Availability {
    Self::dispatch_terminal_availability()
}
```

Each platform module implements:

```rust
#[cfg(feature = "terminal")]
fn dispatch_terminal_availability() -> Availability;
```

Linux should share the terminal candidate list with `dispatch_terminal` to avoid
drift. A small constant or helper is preferred over duplicating the list.

## Testing

Deterministic tests should stay under `src/file_handle/tests.rs`.

Minimum coverage:

- A path-search helper reports available for an executable file in a temporary
  directory added to a synthetic search path.
- The same helper reports unavailable for a missing command.
- A directory named like a command is not treated as available.
- A non-executable file is not treated as available on Unix.
- Candidate-list order is deterministic.
- Linux candidate-list probing can be tested through a `pub(crate)` helper that
  accepts candidates and a synthetic search path.
- Tests use synthetic search path inputs and do not rely on or mutate the
  process `PATH`.
- `Availability` is exported in all feature combinations once this RFC lands.
- `terminal_availability()` is compiled under `feature = "terminal"`.
- Default and non-terminal feature builds do not expose
  `terminal_availability()`.

Do not write tests that require a real GUI terminal emulator to be installed.
Those would be environment-dependent and duplicate the role of `open_terminal`
integration tests.

## Documentation

README/API docs should state:

- `Availability` is an always-available public vocabulary type once this RFC
  lands.
- `terminal_availability()` is available only with the `terminal` feature.
- It is best-effort, process-environment scoped, and may race with later use.
- `Unknown` means the probe could not make a useful advisory decision.
- `Unavailable` may be a false negative.
- It is mainly useful for GUI pre-click affordances and graceful CLI/headless
  fallback decisions.
- Callers must still handle `open_terminal` errors.

Recommended caller shape:

```rust
match FileHandle::terminal_availability() {
    Availability::Available | Availability::Unknown => {
        // Keep the action visible, then handle open_terminal errors.
    }
    Availability::Unavailable => {
        // Disable or hide the action, or offer a manual fallback.
    }
}
```

## Compatibility

This RFC is additive and can land in a minor release after `0.3.0`.

It does not change existing `open_terminal` behavior or the candidate list by
itself.

## Acceptance Criteria

This RFC is implemented when:

1. `FileHandle::terminal_availability() -> Availability` exists behind
   `feature = "terminal"`.
2. `Availability` is exported in all feature combinations and derives `Debug`,
   `Clone`, `Copy`, `PartialEq`, `Eq`, and `Hash`.
3. Linux probing reuses the same terminal candidates as `open_terminal`.
4. The probe does not spawn a terminal.
5. Default and non-terminal feature builds remain clean.
6. Deterministic tests cover the path-search/probe helpers without requiring a
   real terminal emulator.
7. Documentation clearly states the best-effort/racy nature of the API and the
   meanings of `Available`, `Unavailable`, and `Unknown`.
8. Compile checks prove the intended public surface for default, terminal-only,
   and all-features builds.
