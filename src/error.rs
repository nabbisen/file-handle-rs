use std::fmt;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Operation {
    Open,
    Show,
    Terminal,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open => f.write_str("open"),
            Self::Show => f.write_str("show"),
            Self::Terminal => f.write_str("terminal"),
        }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum FileHandleError {
    #[error("Path not found: {0}")]
    NotFound(PathBuf),
    #[error("no OS handler available for {operation} (tried: {tried:?})")]
    NoHandlerAvailable {
        operation: Operation,
        tried: Vec<String>,
    },
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
