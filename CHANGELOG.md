# Changelog

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
