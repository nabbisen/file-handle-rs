# file-handle

[![crates.io](https://img.shields.io/crates/v/file-handle?label=rust)](https://crates.io/crates/file-handle)
[![License](https://img.shields.io/github/license/nabbisen/file-handle-rs)](https://github.com/nabbisen/file-handle-rs/blob/main/LICENSE)
[![Documentation](https://docs.rs/file-handle/badge.svg?version=latest)](https://docs.rs/file-handle)
[![Dependency Status](https://deps.rs/crate/file-handle/latest/status.svg)](https://deps.rs/crate/file-handle)
[![CI tests](https://github.com/nabbisen/file-handle-rs/actions/workflows/test.yaml/badge.svg)](https://github.com/nabbisen/file-handle-rs/actions/workflows/test.yaml)

A humble Rust library to show files in the system manager, open terminals, or move items to the trash.

## Design goals

`file-handle` was created to provide a minimal and dependency-conscious way to handle common desktop interactions. This project focuses on:

- **Minimalism**: Small dependency footprint using native commands where possible.
- **Modernity**: Clean, type-safe API designed for the latest Rust editions.
- **Flexibility**: Feature flags to ensure you only compile what you actually use.

## Usage

### Important note 🗒️ for first-time users

If you simply add `file-handle` to your dependencies **without specifying any features**, you will not see any methods available under `FileHandle::`.

To keep compile time fast and the dependency tree as small as possible, `file-handle` is entirely opt-in. You must explicitly enable the features you want to use, or enable the all feature to get everything.

### Feature flags

| Flag | Method Enabled | Description |
| --- | --- | --- |
| `show` | `FileHandle::show(path)` | Opens the system's file manager. If the path is a file, it opens the parent folder and selects the file. If it's a directory, it opens that directory. (Uses zbus on Linux) |
| `terminal` | `FileHandle::open_terminal(path)` | Opens the system's default terminal emulator at the specified path. |
| `trash` | `FileHandle::trash(path)` | Moves the specified file or directory to the system trash/recycle bin. (Uses the trash crate) |
| `all` | All methods | Enables show, terminal, and trash all at once. |

---

## Open-source, with care

This project is lovingly built and maintained by volunteers.  
We hope it helps streamline your API development.  
Please understand that the project has its own direction — while we welcome feedback, it might not fit every edge case 🌱

## Acknowledgements

Depends on [thiserror](https://crates.io/crates/thiserror), [url](https://crates.io/crates/url), [zbus](https://crates.io/crates/zbus), [trash](https://crates.io/crates/trash), In addition, [tempfile](https://crates.io/crates/tempfile) for test automation.
