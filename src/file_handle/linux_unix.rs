#[cfg(feature = "show")]
use std::path::Path;

#[cfg(feature = "show")]
use crate::FileHandleError;

use super::FileHandle;

impl FileHandle {
    #[cfg(feature = "show")]
    pub fn dispatch_show(path: &Path, is_dir: bool) -> Result<(), FileHandleError> {
        use zbus::blocking::Connection;

        let conn = Connection::session()?;
        let abs_path = std::fs::canonicalize(path)?;
        let url = url::Url::from_file_path(&abs_path)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid Path"))?;

        let method = if is_dir { "ShowFolders" } else { "ShowItems" };
        let uri = url.as_str();

        // D-Bus の引数 "(ass)" を構築
        // 第1引数: URIs (array of strings -> Vec<&str>)
        // 第2引数: startup_id (string -> &str)
        let uris: Vec<&str> = vec![uri];
        let startup_id: &str = "";

        conn.call_method(
            Some("org.freedesktop.FileManager1"),
            "/org/freedesktop/FileManager1",
            Some("org.freedesktop.FileManager1"),
            method,
            &(uris, startup_id), // Vec<&str> は zbus によって自動的に 'as' としてシグナリングされます
        )
        .map(|_| ())
        .map_err(|e| FileHandleError::OpFailed(e.to_string()))
    }

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal(path: &Path) -> Result<(), FileHandleError> {
        for term in ["xdg-terminal-exec", "gnome-terminal", "konsole", "xterm"] {
            use std::process::Command;

            if Command::new(term).current_dir(path).spawn().is_ok() {
                return Ok(());
            }
        }
        Err(FileHandleError::OpFailed("No terminal found".into()))
    }
}
