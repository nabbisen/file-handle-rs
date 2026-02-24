use super::FileHandle;

impl FileHandle {
    #[cfg(feature = "show")]
    pub fn dispatch_show(path: &Path, is_dir: bool) -> Result<(), FileHandleError> {
        let mut arg = std::ffi::OsString::from(if is_dir { "" } else { "/select," });
        arg.push(path);

        Command::new("explorer.exe")
            .arg(arg)
            .spawn()?
            .status()
            .map(|_| ())
            .map_err(|e| FileHandleError::OpFailed(e.to_string()))
    }

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal(path: &Path) -> Result<(), FileHandleError> {
        Command::new("cmd")
            .args(["/C", "start", "cmd.exe"])
            .current_dir(path)
            .spawn()?
            .status()
            .map(|_| ())
            .map_err(|e| FileHandleError::OpFailed(e.to_string()))
    }
}
