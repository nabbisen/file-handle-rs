# Feature Flags

`file-handle` uses an empty default feature set.

| Feature | APIs |
| --- | --- |
| `open` | `FileHandle::open_with_default`, `FileHandle::open_all` |
| `show` | `FileHandle::show`, `FileHandle::show_all` |
| `terminal` | `FileHandle::open_terminal` |
| `trash` | `FileHandle::trash`, `FileHandle::trash_all` |
| `all` | Enables all operation features |

`open`, `show`, and `trash` batch helpers return `BatchOutcome`, preserving
per-path successes and failures instead of stopping at the first error.
