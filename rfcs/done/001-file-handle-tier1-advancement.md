# RFC 001 — File-handle Tier 1 advancement

**Status.** Implemented (0.3.0)
**Tracks.** Public API ergonomics, batch operation reporting, operation failure classification, and focused test coverage for `file-handle`.
**Touches.** `src/error.rs`, `src/file_handle.rs`, platform dispatch modules under `src/file_handle/`, source-level tests under `src/file_handle/`, existing integration tests under `tests/`, README/API documentation.

## Summary

This RFC advances `file-handle` as a selection-aware file handling crate for the
`0.3.0` release theme. It adds batch helpers for multi-path UI actions, gives
callers a structured way to distinguish "no OS handler is available" from other
operation failures, makes `trash` path validation consistent and explicit, checks
native launcher exit status, and brings source comments into the project's
English-only comment rule.

The theme is intentionally narrow. It improves the current single-path API for
desktop/file-manager-style callers without expanding the crate into clipboard
handling, file mutation, async runtime management, file watching, trash
management, or application selection.

## Motivation

The current public surface is single-target:

```rust
FileHandle::open_with_default(path)
FileHandle::show(path)
FileHandle::open_terminal(path)
FileHandle::trash(path)
```

That is enough for simple callers, but applications with tree or file-manager UI
usually operate on a selection of N paths. Today every caller must write its own
loop, decide how to preserve partial failures, and string-match `OpFailed`
messages to explain environment problems to the user.

This RFC moves that common execution/reporting glue into `file-handle`, where
the OS-specific handling already lives.

## Goals

1. Add per-path batch outcomes for `open`, `show`, and `trash`.
2. Preserve partial success and per-target failure attribution.
3. Add a structured error for genuine "no OS handler is available" cases.
4. Check native process exit status instead of treating every spawned command as
   success.
5. Add an up-front `trash` existence check that accepts dangling symlinks as
   trashable filesystem entries.
6. Translate existing non-English source/test comments touched by this theme.
7. Add deterministic tests for the new behavior.

## Non-goals

This RFC does not add:

- `open_terminal_all`; opening multiple terminals for a multi-selection is
  surprising.
- Capability probes such as `terminal_available()`.
- Linux `show` fallback to `xdg-open`.
- Preferred terminal override or environment-based terminal selection.
- Clipboard, rename, create, copy, move, trash restore/list/undo, file watching,
  async APIs, or bundled runtimes.
- A structured command-failure error variant. Non-zero process exits remain
  `OpFailed(String)` in this RFC.

Each deferred item requires a separate future RFC if pursued.

## Public API

### Operation

Add a small operation enum used by structured handler-availability errors.

```rust
#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Operation {
    Open,
    Show,
    Terminal,
}
```

Implement `std::fmt::Display` manually for `Operation`; it renders `open`,
`show`, and `terminal`. Do not add a string-conversion dependency for this.

### FileHandleError

Mark `FileHandleError` non-exhaustive and add `NoHandlerAvailable`. The existing
`thiserror` derive, `#[error(...)]` messages, and `#[from]` conversions are
retained.

```rust
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FileHandleError {
    #[error("Path not found: {0}")]
    NotFound(PathBuf),

    #[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
    #[error("no OS handler available for {operation} (tried: {tried:?})")]
    NoHandlerAvailable {
        operation: Operation,
        tried: Vec<String>,
    },

    #[error("Operation failed: {0}")]
    OpFailed(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(all(feature = "show", not(target_os = "macos"), not(target_os = "windows")))]
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),

    #[cfg(feature = "trash")]
    #[error("Trash error: {0}")]
    Trash(String),
}
```

`tried` is `Vec<String>` so future caller-configured candidates can be reported
without changing the field type.

`trash` never returns `NoHandlerAvailable`; it has no spawned handler or
candidate list. It returns `NotFound` for absent entries and `Trash(String)` for
errors returned by `trash_lib`.

### BatchOutcome

Add an owned per-path report:

```rust
#[derive(Debug, Default)]
pub struct BatchOutcome {
    pub succeeded: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, FileHandleError)>,
}

impl BatchOutcome {
    pub fn all_ok(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn any_failed(&self) -> bool {
        !self.failed.is_empty()
    }
}
```

Every input path must appear exactly once, either in `succeeded` or `failed`.
Empty input returns an empty `BatchOutcome`; `all_ok()` is true for empty input.

### Batch methods

Add feature-gated batch helpers:

```rust
#[cfg(feature = "open")]
pub fn open_all(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> BatchOutcome;

#[cfg(feature = "show")]
pub fn show_all(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> BatchOutcome;

#[cfg(feature = "trash")]
pub fn trash_all(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> BatchOutcome;
```

The methods return `BatchOutcome`, not `Result<BatchOutcome, _>`, because there
is no whole-batch preflight failure. Each path is handled independently.

The `impl IntoIterator<Item = impl AsRef<Path>>` shape is accepted as the public
signature for Rust 1.90+ and matches the crate's existing `AsRef<Path>` style
while avoiding call-site allocation for common UI selection shapes.

## Behavior

### Batch execution

`open_all`, `show_all`, and `trash_all` loop over inputs and delegate each path
to the corresponding single-path method. If the single-path method returns
`Ok(())`, the owned path is pushed to `succeeded`. If it returns `Err(error)`,
the owned path and error are pushed to `failed`.

`show_all` deliberately uses per-path execution in Tier 1, even on Linux where
D-Bus can show multiple items in one call. macOS and Windows reveal APIs are
single-item-oriented, and uniform per-path behavior gives consistent attribution
and simpler tests. OS-level grouped reveal is reserved as a future optimization
only if per-path attribution is preserved.

### NoHandlerAvailable mapping

`NoHandlerAvailable` is used only for genuine environment availability failures:

| Operation | Maps to `NoHandlerAvailable` | Does not map |
| --- | --- | --- |
| Linux `open` | Every launcher candidate (`xdg-open`, `gio`, `mimeopen`) is missing. | A launcher spawns but exits non-zero or otherwise fails. |
| Linux `terminal` | Every terminal candidate is missing. | A terminal spawns but exits non-zero or otherwise fails. |
| Linux `show` | Session bus is unreachable, or FileManager1 service is unknown / has no owner. | Other D-Bus reply or operation errors. |
| macOS / Windows `open`, `show`, `terminal` | The system launcher binary is missing on spawn. | Ordinary process failures or non-zero exit status. |
| `trash` | Never. | Missing path is `NotFound`; `trash_lib` failures are `Trash(String)`. |

Do not classify every `zbus::Error` as `NoHandlerAvailable`. Only known service
or name availability failures should be mapped that way.

### Process exit status

All native commands touched by this RFC must check the returned process status.
If the command exits non-zero, return `FileHandleError::OpFailed` with operation,
command, and status context.

A zero exit status is treated as success, but only as best-effort evidence. Many
desktop launchers hand off to another application and return immediately.

### Trash validation

`FileHandle::trash` validates the path before calling `trash_lib::delete`.

Use `std::fs::symlink_metadata()` rather than `Path::exists()` or
`Path::try_exists()`. A dangling symlink is still a filesystem entry that a file
manager should allow the user to trash, even though its target is missing.

Validation semantics are operation-specific:

- `open` acts on the target and may continue to follow symlinks.
- `trash` acts on the directory entry and therefore uses symlink-aware
  validation.
- `show` also has an argument for entry-oriented behavior, but changing its
  existing validation is deferred to a future alignment RFC.

This creates an explicit Tier 1 asymmetry: `trash` and `trash_all` accept a
dangling symlink as a trashable entry, while `show` and `show_all` continue to
follow the link through `metadata()` and report a dangling symlink as
`NotFound`. This is a conscious deferred boundary, not an accidental contract.

## Implementation Plan

1. Add `Operation`, make `FileHandleError` non-exhaustive, and add
   `NoHandlerAvailable`.
2. Add command-status handling helpers for native dispatch code.
3. Update Linux, macOS, and Windows dispatchers to classify missing handlers and
   non-zero exits according to this RFC.
4. Add `BatchOutcome` and the three batch methods.
5. Add `trash` up-front validation using `symlink_metadata()`.
6. Translate touched non-English comments to English.
7. Add deterministic source-level tests and keep integration tests tolerant of
   headless environments.
8. Run `cargo fmt` once after implementation, then run checks/tests.

## Testing

Deterministic library behavior should be tested under `src/file_handle/tests.rs`,
with further splitting under `src/file_handle/tests/` only if the file grows too
large. Environment-dependent tests that launch real desktop handlers stay under
`tests/`.

Minimum deterministic coverage:

- `BatchOutcome::all_ok`, `BatchOutcome::any_failed`, and empty input behavior.
- Batch methods report `NotFound` per missing path.
- `trash` returns `NotFound` before delegating to `trash_lib`.
- `trash` validation accepts dangling symlinks where the platform supports
  symlinks.
- `NoHandlerAvailable` display and value matching using `Operation`, without
  string matching on the operation.
- Deterministic all-candidates-missing mapping through a small `pub(crate)`
  helper that accepts candidate lists.

The candidate-list helper is allowed only as a small factoring seam, not as a
general dependency-injection framework.

`NoHandlerAvailable` is only partially deterministically testable. The
candidate-list rows for Linux `open` and `terminal` should be covered by the
helper above. Linux `show` D-Bus availability classification and macOS/Windows
system-launcher-missing cases may be covered by direct classifier tests where
practical, opportunistic integration behavior, and code inspection; the RFC does
not require tests that fake a full D-Bus session or remove system binaries.

## Compatibility

This RFC is appropriate for a `0.3.0` minor release.

The new batch APIs are additive. `#[non_exhaustive]` and the new
`NoHandlerAvailable` variant are minor-release-appropriate changes in the
current `0.x` series and prepare the error type for future growth.

Existing callers that exhaustively match `FileHandleError` will need to add a
wildcard arm after `#[non_exhaustive]`.

## Documentation

README/API docs should mention:

- Features remain opt-in with `default = []`.
- Batch helpers are enabled by the same per-operation feature flags.
- Batch helpers preserve partial success and do not stop at the first error.
- `NoHandlerAvailable` is intended for user-facing "no suitable OS handler"
  messaging.
- Native launcher success is best effort.
- `trash_all` accepts dangling symlinks as trashable entries, while `show_all`
  does not until a future validation-alignment RFC changes `show`.

## Acceptance Criteria

This RFC is implemented when:

1. The public API described above is available behind the existing feature flags.
2. `show_all` reports per-path outcomes and does not use grouped reveal in Tier 1.
3. `NoHandlerAvailable` is generated only by the mapping rules in this RFC.
4. Native process exit status is checked and non-zero exits become `OpFailed`.
5. `trash` validates with `symlink_metadata()` and accepts dangling symlinks as
   entries where supported.
6. Deterministic source-level tests cover batch outcomes, missing paths,
   `NoHandlerAvailable`, and `trash` validation.
7. Touched source/test comments are in English.
8. Tier 2 items remain deferred.
