#[cfg(feature = "terminal")]
use std::ffi::OsStr;
#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
use std::path::Path;

use super::FileHandle;
#[cfg(feature = "terminal")]
use crate::Availability;
#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
use crate::FileHandleError;
#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
use crate::Operation;

#[cfg(feature = "terminal")]
pub(crate) const TERMINAL_CANDIDATES: &[&str] =
    &["xdg-terminal-exec", "gnome-terminal", "konsole", "xterm"];

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
        Self::dispatch_terminal_with(path, TERMINAL_CANDIDATES.iter().copied())
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

    #[cfg(feature = "terminal")]
    pub fn dispatch_terminal_availability() -> Availability {
        let path = std::env::var_os("PATH");
        let display = std::env::var_os("DISPLAY");
        let wayland_display = std::env::var_os("WAYLAND_DISPLAY");

        Self::terminal_availability_with(
            TERMINAL_CANDIDATES.iter().copied(),
            path.as_deref(),
            display.as_deref(),
            wayland_display.as_deref(),
        )
    }

    #[cfg(feature = "terminal")]
    pub(crate) fn terminal_availability_with<'a>(
        terminals: impl IntoIterator<Item = &'a str>,
        search_path: Option<&OsStr>,
        display: Option<&OsStr>,
        wayland_display: Option<&OsStr>,
    ) -> Availability {
        if Self::is_headless_display(display, wayland_display) {
            return Availability::Unavailable;
        }

        if terminals
            .into_iter()
            .any(|terminal| Self::command_available_on_path(terminal, search_path))
        {
            Availability::Available
        } else {
            Availability::Unavailable
        }
    }

    #[cfg(feature = "terminal")]
    pub(crate) fn command_available_on_path(command: &str, search_path: Option<&OsStr>) -> bool {
        let Some(search_path) = search_path else {
            return false;
        };

        std::env::split_paths(search_path).any(|dir| {
            if dir.as_os_str().is_empty() || !dir.is_dir() {
                return false;
            }

            Self::is_executable_file(&dir.join(command))
        })
    }

    #[cfg(feature = "terminal")]
    fn is_headless_display(display: Option<&OsStr>, wayland_display: Option<&OsStr>) -> bool {
        fn missing(value: Option<&OsStr>) -> bool {
            value.is_none_or(OsStr::is_empty)
        }

        missing(display) && missing(wayland_display)
    }

    #[cfg(all(unix, feature = "terminal"))]
    fn is_executable_file(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;

        let Ok(metadata) = path.metadata() else {
            return false;
        };

        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(all(not(unix), feature = "terminal"))]
    fn is_executable_file(path: &Path) -> bool {
        path.is_file()
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
