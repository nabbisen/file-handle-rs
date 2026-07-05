# file-handle

[![crates.io](https://img.shields.io/crates/v/file-handle?label=rust)](https://crates.io/crates/file-handle)
[![License](https://img.shields.io/github/license/nabbisen/file-handle-rs)](https://github.com/nabbisen/file-handle-rs/blob/main/LICENSE)
[![Documentation](https://docs.rs/file-handle/badge.svg?version=latest)](https://docs.rs/file-handle)
[![Dependency Status](https://deps.rs/crate/file-handle/latest/status.svg)](https://deps.rs/crate/file-handle)
[![CI tests](https://github.com/nabbisen/file-handle-rs/actions/workflows/test.yaml/badge.svg)](https://github.com/nabbisen/file-handle-rs/actions/workflows/test.yaml)

A lean lib to manage files: open with default apps, reveal in managers or terminals, or trash files.

## Overview

`file-handle` provides small, feature-gated Rust APIs for common desktop file
actions. It delegates to native OS handlers where possible and keeps optional
dependencies behind operation-specific feature flags.

## Why / When

Use this crate when an application needs to:

- open files or directories with the OS default app;
- reveal paths in the system file manager;
- open a terminal at a path;
- move files or directories to the system trash;
- report partial failures for multi-selection file actions.

## Quick Start

```toml
[dependencies]
file-handle = { version = "0.4", features = ["open", "show", "trash"] }
```

```rust
use file_handle::FileHandle;

let outcome = FileHandle::trash_all(["old-a.txt", "old-b.txt"]);

if outcome.any_failed() {
    for (path, error) in outcome.failed {
        eprintln!("{}: {error}", path.display());
    }
}
```

If you add `file-handle` without features, no operation methods are enabled.
Enable the features you need, or use `all`.

## Features / Design Notes

| Flag | Method Enabled | Description |
| --- | --- | --- |
| `open` | `FileHandle::open_with_default(path)` | Opens the file using the system's default application associated with its file extension. |
| `show` | `FileHandle::show(path)` | Opens the system's file manager. If the path is a file, it opens the parent folder and selects the file. If it's a directory, it opens that directory. (Uses zbus on Linux) |
| `terminal` | `FileHandle::open_terminal(path)`, `FileHandle::terminal_availability()` | Opens the system's default terminal emulator at the specified path and provides a best-effort availability probe. |
| `trash` | `FileHandle::trash(path)` | Moves the specified file or directory to the system trash/recycle bin. (Uses the trash crate) |
| `all` | All methods | Enables open, show, terminal, and trash all at once. |

The `open`, `show`, and `trash` features also provide batch helpers:
`FileHandle::open_all(paths)`, `FileHandle::show_all(paths)`, and
`FileHandle::trash_all(paths)`. These return a `BatchOutcome` with per-path
successes and failures, so callers can report partial failure without stopping
at the first error.

When no suitable OS handler is available for `open`, `show`, or `terminal`,
the error is reported as `FileHandleError::NoHandlerAvailable`. Native launcher
success is best-effort: many desktop launchers hand off to another application
and return immediately.

`Availability` is always exported. With the `terminal` feature enabled,
`FileHandle::terminal_availability()` reports whether opening a terminal appears
worth trying in the current process environment. The result is advisory:
`Available` can still fail later, `Unavailable` can be a false negative, and
`Unknown` means callers should usually keep the action visible while still
handling `open_terminal` errors.

`trash` and `trash_all` treat dangling symlinks as trashable filesystem entries.
`show` and `show_all` currently follow symlink targets and may report a dangling
symlink as `NotFound`.

## More Detail

Full documentation lives under [`docs/src`](./docs/src/SUMMARY.md).

## Open-source, with care

This project is lovingly built and maintained by volunteers.  
We hope it helps streamline your API development.  
Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on [thiserror](https://crates.io/crates/thiserror), [url](https://crates.io/crates/url), [zbus](https://crates.io/crates/zbus), and [trash](https://crates.io/crates/trash). Uses [tempfile](https://crates.io/crates/tempfile) for test automation.
