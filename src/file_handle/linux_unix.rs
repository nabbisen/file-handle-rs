#[allow(unused_imports)]
use std::path::Path;

use super::FileHandle;
#[allow(unused_imports)]
use crate::FileHandleError;

impl FileHandle {
    #[cfg(feature = "open")]
    pub fn dispatch_open(path: &Path) -> Result<(), FileHandleError> {
        use std::process::Command;

        // xdg-open はデフォルトアプリに委譲する標準ツール。
        // xdg-open が見つからない環境向けに gio open / mimeopen をフォールバックとして試みる。
        for launcher in ["xdg-open", "gio open", "mimeopen"] {
            let mut parts = launcher.split_whitespace();
            let bin = parts.next().unwrap();
            let mut cmd = Command::new(bin);
            for arg in parts {
                cmd.arg(arg);
            }
            cmd.arg(path);

            match cmd.spawn() {
                Ok(mut child) => {
                    child.wait()?;
                    return Ok(());
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            }
        }

        Err(FileHandleError::OpFailed(
            "No suitable launcher found (tried: xdg-open, gio, mimeopen)".into(),
        ))
    }

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
        use std::process::Command;

        for term in ["xdg-terminal-exec", "gnome-terminal", "konsole", "xterm"] {
            if Command::new(term).current_dir(path).spawn().is_ok() {
                return Ok(());
            }
        }

        Err(FileHandleError::OpFailed("No terminal found".into()))
    }
}
