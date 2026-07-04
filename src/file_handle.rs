#[cfg(any(
    feature = "show",
    feature = "open",
    feature = "terminal",
    feature = "trash"
))]
use std::path::Path;
use std::path::PathBuf;

#[allow(unused_imports)]
use crate::FileHandleError;

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
mod linux_unix;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// A humble helper for file-related UI operations.
pub struct FileHandle;

/// Per-path result report for batch file operations.
#[derive(Debug, Default)]
pub struct BatchOutcome {
    pub succeeded: Vec<PathBuf>,
    pub failed: Vec<(PathBuf, FileHandleError)>,
}

impl BatchOutcome {
    pub fn all_ok(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn any_failed(&self) -> bool {
        !self.failed.is_empty()
    }
}

impl FileHandle {
    // --- feature: open ---
    /// Opens the file (or directory) with the OS default application
    /// associated with its type / extension.
    #[cfg(feature = "open")]
    pub fn open_with_default<P: AsRef<Path>>(path: P) -> Result<(), FileHandleError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(FileHandleError::NotFound(path.to_owned()));
        }

        Self::dispatch_open(path)
    }

    #[cfg(feature = "open")]
    pub fn open_all(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> BatchOutcome {
        Self::batch(paths, |path| Self::open_with_default(path))
    }

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

    #[cfg(feature = "show")]
    pub fn show_all(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> BatchOutcome {
        Self::batch(paths, |path| Self::show(path))
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
        let path = path.as_ref();

        std::fs::symlink_metadata(path).map_err(|_| FileHandleError::NotFound(path.to_owned()))?;

        trash_lib::delete(path).map_err(|e| FileHandleError::Trash(e.to_string()))
    }

    #[cfg(feature = "trash")]
    pub fn trash_all(paths: impl IntoIterator<Item = impl AsRef<Path>>) -> BatchOutcome {
        Self::batch(paths, |path| Self::trash(path))
    }

    #[cfg(any(feature = "open", feature = "show", feature = "trash"))]
    fn batch(
        paths: impl IntoIterator<Item = impl AsRef<Path>>,
        operation: impl Fn(&Path) -> Result<(), FileHandleError>,
    ) -> BatchOutcome {
        let mut outcome = BatchOutcome::default();

        for path in paths {
            let path = path.as_ref().to_path_buf();
            match operation(&path) {
                Ok(()) => outcome.succeeded.push(path),
                Err(error) => outcome.failed.push((path, error)),
            }
        }

        outcome
    }
}

#[cfg(test)]
mod tests;
