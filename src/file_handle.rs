#[cfg(feature = "show")]
use std::path::Path;

#[cfg(feature = "show")]
use crate::FileHandleError;

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
mod linux_unix;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// A humble helper for file-related UI operations.
pub struct FileHandle;

impl FileHandle {
    // --- feature: show ---
    /// Opens the file manager and selects the path (or opens the directory).
    #[cfg(feature = "show")]
    pub fn show<P: AsRef<Path>>(path: P) -> Result<(), FileHandleError> {
        let path = path.as_ref();

        let is_dir = path
            .metadata()
            .map_err(|_| FileHandleError::NotFound(path.to_owned()))?
            .is_dir();

        Self::dispatch_show(path, is_dir)
    }

    // --- feature: terminal ---
    /// Opens a terminal emulator at the given path.
    #[cfg(feature = "terminal")]
    pub fn open_terminal<P: AsRef<Path>>(path: P) -> Result<(), FileHandleError> {
        let path = path.as_ref();

        let dir = if path.is_dir() {
            path
        } else {
            path.parent()
                .ok_or_else(|| FileHandleError::OpFailed("No parent".into()))?
        };

        Self::dispatch_terminal(dir)
    }

    // --- feature: trash ---
    /// Moves the given path to the system trash.
    #[cfg(feature = "trash")]
    pub fn trash<P: AsRef<Path>>(path: P) -> Result<(), FileHandleError> {
        trash_lib::delete(path).map_err(|e| FileHandleError::Trash(e.to_string()))
    }
}
