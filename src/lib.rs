mod error;
mod file_handle;

pub use error::Operation;
pub use {
    error::FileHandleError,
    file_handle::{Availability, BatchOutcome, FileHandle},
};
