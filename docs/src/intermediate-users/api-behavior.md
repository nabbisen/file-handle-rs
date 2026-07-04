# API Behavior

## Handler Availability

When no suitable OS handler is available for `open`, `show`, or `terminal`,
the operation returns `FileHandleError::NoHandlerAvailable`.

This error is intended for user-facing messages such as "no terminal emulator is
available" without string-matching generic operation failures.

## Launcher Status

Native launcher commands are checked for non-zero exit status. A non-zero status
returns `FileHandleError::OpFailed`.

A zero status is best-effort evidence only. Many desktop launchers hand work off
to another application and return before the application finishes opening.

## Symlinks

`trash` and `trash_all` act on filesystem entries and validate with
`symlink_metadata()`. A dangling symlink is therefore accepted as a trashable
entry.

`show` and `show_all` currently follow symlink targets through `metadata()`.
They may report a dangling symlink as `NotFound`. This is a documented deferred
boundary for a future validation-alignment RFC.
