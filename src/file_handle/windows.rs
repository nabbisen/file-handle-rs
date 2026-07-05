#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
use std::path::Path;
#[cfg(feature = "terminal")]
use std::path::PathBuf;
#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
use std::process::Command;

use super::FileHandle;
#[cfg(feature = "terminal")]
use crate::Availability;
#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
use crate::FileHandleError;
#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
use crate::Operation;

impl FileHandle {
    #[cfg(feature = "open")]
    pub fn dispatch_open(path: &Path) -> Result<(), FileHandleError> {
        let mut child = Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
            .map_err(|e| Self::map_spawn_error(Operation::Open, "cmd", e))?;

        Self::wait_for_command(Operation::Open, "cmd", &mut child)
    }

    #[cfg(feature = "show")]
    pub fn dispatch_show(path: &Path, is_dir: bool) -> Result<(), FileHandleError> {
        let mut arg = std::ffi::OsString::from(if is_dir { "" } else { "/select," });
        arg.push(path);

        let mut child = Command::new("explorer.exe")
            .arg(arg)
            .spawn()
            .map_err(|e| Self::map_spawn_error(Operation::Show, "explorer.exe", e))?;

        Self::wait_for_command(Operation::Show, "explorer.exe", &mut child)
    }

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal(path: &Path) -> Result<(), FileHandleError> {
        let Some(cmd_path) = Self::trusted_cmd_path() else {
            return Err(Self::no_handler(
                Operation::Terminal,
                ["%ComSpec%", "%SystemRoot%\\System32\\cmd.exe"],
            ));
        };
        let command_name = cmd_path.display().to_string();

        // This resolves the outer cmd.exe through trusted locations. The
        // `cmd /C start` launch strategy itself remains Windows shell debt.
        let mut child = Command::new(&cmd_path)
            .args(["/C", "start", "cmd.exe"])
            .current_dir(path)
            .spawn()
            .map_err(|e| Self::map_spawn_error(Operation::Terminal, &command_name, e))?;

        Self::wait_for_command(Operation::Terminal, &command_name, &mut child)
    }

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal_availability() -> Availability {
        if Self::trusted_cmd_path().is_some() {
            Availability::Available
        } else {
            Availability::Unavailable
        }
    }

    #[cfg(feature = "terminal")]
    fn trusted_cmd_path() -> Option<PathBuf> {
        Self::trusted_cmd_path_from(std::env::var_os("ComSpec"), std::env::var_os("SystemRoot"))
    }

    #[cfg(feature = "terminal")]
    pub(crate) fn trusted_cmd_path_from(
        comspec: Option<std::ffi::OsString>,
        system_root: Option<std::ffi::OsString>,
    ) -> Option<PathBuf> {
        comspec
            .map(PathBuf::from)
            .filter(|path| path.is_absolute() && path.is_file())
            .or_else(|| {
                let mut path = PathBuf::from(system_root?);
                path.push("System32");
                path.push("cmd.exe");

                if path.is_file() { Some(path) } else { None }
            })
    }

    #[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
    fn wait_for_command(
        operation: Operation,
        command: &str,
        child: &mut std::process::Child,
    ) -> Result<(), FileHandleError> {
        let status = child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(FileHandleError::OpFailed(format!(
                "{operation}: `{command}` exited with {status}"
            )))
        }
    }

    #[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
    fn map_spawn_error(
        operation: Operation,
        command: &str,
        error: std::io::Error,
    ) -> FileHandleError {
        if error.kind() == std::io::ErrorKind::NotFound {
            FileHandleError::NoHandlerAvailable {
                operation,
                tried: vec![command.to_owned()],
            }
        } else {
            error.into()
        }
    }

    #[cfg(feature = "terminal")]
    fn no_handler<'a>(
        operation: Operation,
        tried: impl IntoIterator<Item = &'a str>,
    ) -> FileHandleError {
        FileHandleError::NoHandlerAvailable {
            operation,
            tried: tried.into_iter().map(str::to_owned).collect(),
        }
    }
}
