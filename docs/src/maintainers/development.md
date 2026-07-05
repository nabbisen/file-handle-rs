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

Release-candidate verification should include:

```sh
cargo fmt
cargo check --no-default-features
cargo test
cargo test --all-features
cargo test --features open
cargo test --features show
cargo test --features terminal
cargo test --features trash
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --all-features --no-deps
mdbook build docs
cargo package
cargo publish --dry-run
```

Use `cargo package --allow-dirty` only for local review packaging, not final
release approval.

Deterministic library tests live under `src/` beside the implementation. Tests
that launch real OS handlers stay under `tests/` and must tolerate headless CI
environments. Real desktop-handler integration tests are ignored by default so
routine `cargo test` runs do not open file managers, browsers, or terminals.
Run them explicitly with `cargo test --all-features -- --ignored` when manually
checking local OS integration.
