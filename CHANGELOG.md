# Changelog

## 0.4.0

- Added `Availability` and `FileHandle::terminal_availability()` for
  best-effort terminal launch probing.
- Added trusted Windows `cmd.exe` resolution for terminal availability probing;
  Windows terminal launch still uses `cmd /C start` and remains design debt.
- Made `Operation` and `FileHandleError::NoHandlerAvailable` available without
  operation features.
- Added deterministic source-level tests for terminal availability probing.
- Updated user and maintainer docs for terminal availability and the clippy
  release-candidate gate.
- Lowered the crate `rust-version` from `1.90` to `1.87`.

## 0.3.0

- Added `BatchOutcome` and batch helpers for `open`, `show`, and `trash`.
- Added `Operation` and `FileHandleError::NoHandlerAvailable` for handler
  availability failures.
- Added symlink-aware `trash` validation so dangling symlinks are accepted as
  trashable entries.
- Added process exit status checks for native launcher commands.
- Added deterministic source-level tests for RFC 001 behavior.
- Updated user docs for batch outcomes, handler availability errors, and the
  current `trash_all` / `show_all` dangling-symlink asymmetry.
