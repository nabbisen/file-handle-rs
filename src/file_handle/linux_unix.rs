#[allow(unused_imports)]
use std::path::Path;

use super::FileHandle;
#[allow(unused_imports)]
use crate::FileHandleError;
#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
use crate::Operation;

impl FileHandle {
    #[cfg(feature = "open")]
    pub fn dispatch_open(path: &Path) -> Result<(), FileHandleError> {
        Self::dispatch_open_with(path, ["xdg-open", "gio open", "mimeopen"])
    }

    #[cfg(feature = "open")]
    pub(crate) fn dispatch_open_with<'a>(
        path: &Path,
        launchers: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), FileHandleError> {
        use std::process::Command;

        let launchers: Vec<&str> = launchers.into_iter().collect();

        for launcher in &launchers {
            let mut parts = launcher.split_whitespace();
            let bin = parts.next().unwrap();
            let mut cmd = Command::new(bin);
            for arg in parts {
                cmd.arg(arg);
            }
            cmd.arg(path);

            match cmd.spawn() {
                Ok(mut child) => {
                    return Self::wait_for_command(Operation::Open, launcher, &mut child);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            }
        }

        Err(Self::no_handler(Operation::Open, launchers))
    }

    #[cfg(feature = "show")]
    pub fn dispatch_show(path: &Path, is_dir: bool) -> Result<(), FileHandleError> {
        use zbus::blocking::Connection;

        let conn = Connection::session().map_err(|_| {
            Self::no_handler(
                Operation::Show,
                ["session bus", "org.freedesktop.FileManager1"],
            )
        })?;
        let abs_path = std::fs::canonicalize(path)?;
        let url = url::Url::from_file_path(&abs_path)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid Path"))?;

        let method = if is_dir { "ShowFolders" } else { "ShowItems" };
        let uri = url.as_str();

        let uris: Vec<&str> = vec![uri];
        let startup_id: &str = "";

        conn.call_method(
            Some("org.freedesktop.FileManager1"),
            "/org/freedesktop/FileManager1",
            Some("org.freedesktop.FileManager1"),
            method,
            &(uris, startup_id),
        )
        .map(|_| ())
        .map_err(|e| {
            if Self::is_file_manager_unavailable(&e) {
                Self::no_handler(Operation::Show, ["org.freedesktop.FileManager1"])
            } else {
                FileHandleError::OpFailed(e.to_string())
            }
        })
    }

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal(path: &Path) -> Result<(), FileHandleError> {
        Self::dispatch_terminal_with(
            path,
            ["xdg-terminal-exec", "gnome-terminal", "konsole", "xterm"],
        )
    }

    #[cfg(feature = "terminal")]
    pub(crate) fn dispatch_terminal_with<'a>(
        path: &Path,
        terminals: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), FileHandleError> {
        use std::process::Command;

        let terminals: Vec<&str> = terminals.into_iter().collect();

        for term in &terminals {
            match Command::new(term).current_dir(path).spawn() {
                Ok(mut child) => {
                    return Self::wait_for_command(Operation::Terminal, term, &mut child);
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e.into()),
            }
        }

        Err(Self::no_handler(Operation::Terminal, terminals))
    }

    #[cfg(any(feature = "open", feature = "terminal"))]
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
    fn no_handler<'a>(
        operation: Operation,
        tried: impl IntoIterator<Item = &'a str>,
    ) -> FileHandleError {
        FileHandleError::NoHandlerAvailable {
            operation,
            tried: tried.into_iter().map(str::to_owned).collect(),
        }
    }

    #[cfg(feature = "show")]
    pub(crate) fn is_file_manager_unavailable(error: &zbus::Error) -> bool {
        const SERVICE_UNKNOWN: &str = "org.freedesktop.DBus.Error.ServiceUnknown";
        const NAME_HAS_NO_OWNER: &str = "org.freedesktop.DBus.Error.NameHasNoOwner";

        match error {
            zbus::Error::FDO(error) => matches!(
                error.as_ref(),
                zbus::fdo::Error::ServiceUnknown(_) | zbus::fdo::Error::NameHasNoOwner(_)
            ),
            zbus::Error::MethodError(name, _, _) => {
                let name = name.as_str();
                name == SERVICE_UNKNOWN || name == NAME_HAS_NO_OWNER
            }
            _ => false,
        }
    }
}
