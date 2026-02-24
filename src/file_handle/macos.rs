use super::FileHandle;
#[allow(unused_imports)]
use crate::FileHandleError;

impl FileHandle {
    #[cfg(feature = "show")]
    pub fn dispatch_show(path: &Path, is_dir: bool) -> Result<(), FileHandleError> {
        let mut cmd = Command::new("open");

        if !is_dir {
            cmd.arg("-R");
        }

        cmd.arg(path)
            .spawn()?
            .status()
            .map(|_| ())
            .map_err(|e| FileHandleError::OpFailed(e.to_string()))
    }

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal(path: &Path) -> Result<(), FileHandleError> {
        Command::new("open")
            .args(["-a", "Terminal"])
            .arg(path)
            .spawn()?
            .status()
            .map(|_| ())
            .map_err(|e| FileHandleError::OpFailed(e.to_string()))
    }
}
