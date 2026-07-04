# Development

Run formatting after implementation work:

```sh
cargo fmt
```

Run the default and feature-gated test sets:

```sh
cargo test
cargo test --all-features
cargo test --features open
cargo test --features show
cargo test --features terminal
cargo test --features trash
```

Deterministic library tests live under `src/` beside the implementation. Tests
that launch real OS handlers stay under `tests/` and must tolerate headless CI
environments.
