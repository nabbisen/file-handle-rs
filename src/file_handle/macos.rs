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
        let mut child = Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| Self::map_spawn_error(Operation::Open, "open", e))?;

        Self::wait_for_command(Operation::Open, "open", &mut child)
    }

    #[cfg(feature = "show")]
    pub fn dispatch_show(path: &Path, is_dir: bool) -> Result<(), FileHandleError> {
        let mut cmd = Command::new("open");

        if !is_dir {
            cmd.arg("-R");
        }

        let mut child = cmd
            .arg(path)
            .spawn()
            .map_err(|e| Self::map_spawn_error(Operation::Show, "open", e))?;

        Self::wait_for_command(Operation::Show, "open", &mut child)
    }

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal(path: &Path) -> Result<(), FileHandleError> {
        let mut child = Command::new("open")
            .args(["-a", "Terminal"])
            .arg(path)
            .spawn()
            .map_err(|e| Self::map_spawn_error(Operation::Terminal, "open", e))?;

        Self::wait_for_command(Operation::Terminal, "open", &mut child)
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
