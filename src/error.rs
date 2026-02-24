use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileHandleError {
    #[error("Path not found: {0}")]
    NotFound(PathBuf),
    #[error("Operation failed: {0}")]
    OpFailed(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[cfg(all(feature = "show", not(target_os = "macos"), not(target_os = "windows")))]
    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),

    #[cfg(feature = "trash")]
    #[error("Trash error: {0}")]
    Trash(String),
}
