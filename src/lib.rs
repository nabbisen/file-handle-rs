mod error;
mod file_handle;

#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
pub use error::Operation;
pub use {
    error::FileHandleError,
    file_handle::{BatchOutcome, FileHandle},
};
