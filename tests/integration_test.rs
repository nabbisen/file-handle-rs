// ```sh
// # test all:
// cargo test -F all
//
// # mock run:
// cargo run --example smoke_test --features show
// ```

#[cfg(feature = "show")]
#[path = "integration_test/show.rs"]
mod show;
#[cfg(feature = "terminal")]
#[path = "integration_test/terminal.rs"]
mod terminal;
#[cfg(feature = "trash")]
#[path = "integration_test/trash.rs"]
mod trash;
