// ```sh
// # test all:
// cargo test --all-features
//
// # mock run:
// cargo run --example smoke_test --features show
//
// Real desktop-handler tests are ignored by default because they can open file
// managers, browsers, or terminal applications on developer machines.
// Run them explicitly with:
// cargo test --all-features -- --ignored
// ```

#[cfg(feature = "open")]
#[path = "integration_test/open.rs"]
mod open;
#[cfg(feature = "show")]
#[path = "integration_test/show.rs"]
mod show;
#[cfg(feature = "terminal")]
#[path = "integration_test/terminal.rs"]
mod terminal;
#[cfg(feature = "trash")]
#[path = "integration_test/trash.rs"]
mod trash;
