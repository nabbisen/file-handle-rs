# Quick Start

Add `file-handle` with the features you need:

```toml
[dependencies]
file-handle = { version = "0.3", features = ["open", "show", "trash"] }
```

Use the single-path helpers for direct actions:

```rust
use file_handle::FileHandle;

FileHandle::open_with_default("report.pdf")?;
FileHandle::show("report.pdf")?;
FileHandle::trash("old-report.pdf")?;
# Ok::<(), file_handle::FileHandleError>(())
```

Use batch helpers when acting on a selection:

```rust
use file_handle::FileHandle;

let outcome = FileHandle::trash_all(["old-a.txt", "old-b.txt"]);

if outcome.any_failed() {
    for (path, error) in outcome.failed {
        eprintln!("{}: {error}", path.display());
    }
}
```
