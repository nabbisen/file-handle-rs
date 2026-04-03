#[allow(unused_imports)]
use std::path::Path;
#[allow(unused_imports)]
use std::process::Command;

use super::FileHandle;
#[allow(unused_imports)]
use crate::FileHandleError;

impl FileHandle {
    #[cfg(feature = "open")]
    pub fn dispatch_open(path: &Path) -> Result<(), FileHandleError> {
        // macOS の `open` コマンドは拡張子 / UTI に基づいてデフォルトアプリを起動する。
        // `-W` フラグを付けることでアプリが終了するまで待機する。
        Command::new("open")
            .arg("-W")
            .arg(path)
            .spawn()?
            .wait()
            .map(|_| ())
            .map_err(|e| FileHandleError::OpFailed(e.to_string()))
    }

    #[cfg(feature = "show")]
    pub fn dispatch_show(path: &Path, is_dir: bool) -> Result<(), FileHandleError> {
        let mut cmd = Command::new("open");

        if !is_dir {
            cmd.arg("-R");
        }

        cmd.arg(path)
            .spawn()?
            .wait()
            .map(|_| ())
            .map_err(|e| FileHandleError::OpFailed(e.to_string()))
    }

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal(path: &Path) -> Result<(), FileHandleError> {
        Command::new("open")
            .args(["-a", "Terminal"])
            .arg(path)
            .spawn()?
            .wait()
            .map(|_| ())
            .map_err(|e| FileHandleError::OpFailed(e.to_string()))
    }
}
