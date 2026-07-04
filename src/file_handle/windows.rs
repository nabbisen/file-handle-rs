#[allow(unused_imports)]
use std::path::Path;
#[allow(unused_imports)]
use std::process::Command;

use super::FileHandle;
#[allow(unused_imports)]
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
        let mut child = Command::new("cmd")
            .args(["/C", "start", "cmd.exe"])
            .current_dir(path)
            .spawn()
            .map_err(|e| Self::map_spawn_error(Operation::Terminal, "cmd", e))?;

        Self::wait_for_command(Operation::Terminal, "cmd", &mut child)
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
}
