#[cfg(any(feature = "open", feature = "show", feature = "trash"))]
use std::path::PathBuf;

use super::BatchOutcome;
#[cfg(any(
    feature = "open",
    feature = "show",
    feature = "trash",
    feature = "terminal"
))]
use super::FileHandle;
use crate::FileHandleError;
use crate::{Availability, Operation};

#[test]
fn batch_outcome_helpers_for_empty_outcome() {
    let outcome = BatchOutcome::default();

    assert!(outcome.all_ok());
    assert!(!outcome.any_failed());
    assert!(outcome.succeeded.is_empty());
    assert!(outcome.failed.is_empty());
}

#[test]
fn operation_display_uses_public_operation_names() {
    assert_eq!(Operation::Open.to_string(), "open");
    assert_eq!(Operation::Show.to_string(), "show");
    assert_eq!(Operation::Terminal.to_string(), "terminal");
}

#[test]
fn no_handler_available_display_is_stable() {
    let error = FileHandleError::NoHandlerAvailable {
        operation: Operation::Open,
        tried: vec!["missing-launcher".to_owned()],
    };

    assert_eq!(
        error.to_string(),
        r#"no OS handler available for open (tried: ["missing-launcher"])"#
    );
}

#[test]
fn availability_values_are_plain_public_values() {
    use std::collections::HashSet;

    let values = [
        Availability::Available,
        Availability::Unavailable,
        Availability::Unknown,
    ];
    let unique = values.into_iter().collect::<HashSet<_>>();

    assert_eq!(unique.len(), 3);
    assert_eq!(format!("{:?}", Availability::Available), "Available");
}

#[cfg(feature = "open")]
#[test]
fn open_all_reports_missing_paths() {
    let paths = missing_paths();

    let outcome = FileHandle::open_all(paths.iter());

    assert!(outcome.succeeded.is_empty());
    assert_eq!(outcome.failed.len(), 2);
    assert_not_found_failures(&outcome);
}

#[cfg(feature = "show")]
#[test]
fn show_all_reports_missing_paths() {
    let paths = missing_paths();

    let outcome = FileHandle::show_all(paths.iter());

    assert!(outcome.succeeded.is_empty());
    assert_eq!(outcome.failed.len(), 2);
    assert_not_found_failures(&outcome);
}

#[cfg(feature = "trash")]
#[test]
fn trash_reports_missing_path_before_delegating() {
    let path = missing_paths().remove(0);

    let result = FileHandle::trash(&path);

    assert!(matches!(result, Err(FileHandleError::NotFound(failed)) if failed == path));
}

#[cfg(feature = "trash")]
#[test]
fn trash_all_reports_missing_paths() {
    let paths = missing_paths();

    let outcome = FileHandle::trash_all(paths.iter());

    assert!(outcome.succeeded.is_empty());
    assert_eq!(outcome.failed.len(), 2);
    assert_not_found_failures(&outcome);
}

#[cfg(all(unix, feature = "trash"))]
#[test]
fn trash_does_not_reject_dangling_symlink_as_not_found() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("missing-target");
    let link = dir.path().join("dangling-link");
    symlink(&target, &link).unwrap();

    let result = FileHandle::trash(&link);

    assert!(
        !matches!(result, Err(FileHandleError::NotFound(_))),
        "dangling symlink was rejected before trash delegation: {result:?}"
    );
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows"), feature = "open"))]
#[test]
fn linux_open_reports_no_handler_when_all_candidates_are_missing() {
    use crate::Operation;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("file.txt");
    std::fs::write(&path, "test").unwrap();

    let result = FileHandle::dispatch_open_with(&path, ["__file_handle_missing_open__"]);

    assert!(matches!(
        result,
        Err(FileHandleError::NoHandlerAvailable {
            operation: Operation::Open,
            tried
        }) if tried == vec!["__file_handle_missing_open__".to_owned()]
    ));
}

#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    feature = "terminal"
))]
#[test]
fn linux_terminal_reports_no_handler_when_all_candidates_are_missing() {
    let dir = tempfile::tempdir().unwrap();

    let result = FileHandle::dispatch_terminal_with(dir.path(), ["__file_handle_missing_term__"]);

    assert!(matches!(
        result,
        Err(FileHandleError::NoHandlerAvailable {
            operation: Operation::Terminal,
            tried
        }) if tried == vec!["__file_handle_missing_term__".to_owned()]
    ));
}

#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    feature = "terminal"
))]
#[test]
fn linux_terminal_availability_reports_available_for_executable_on_synthetic_path() {
    let dir = tempfile::tempdir().unwrap();
    make_executable_file(&dir.path().join("test-terminal"));
    let search_path = std::env::join_paths([dir.path()]).unwrap();

    let availability = FileHandle::terminal_availability_with(
        ["test-terminal"],
        Some(&search_path),
        Some(std::ffi::OsStr::new(":0")),
        None,
    );

    assert_eq!(availability, Availability::Available);
}

#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    feature = "terminal"
))]
#[test]
fn linux_terminal_availability_reports_unavailable_for_missing_command() {
    let dir = tempfile::tempdir().unwrap();
    let search_path = std::env::join_paths([dir.path()]).unwrap();

    let availability = FileHandle::terminal_availability_with(
        ["missing-terminal"],
        Some(&search_path),
        Some(std::ffi::OsStr::new(":0")),
        None,
    );

    assert_eq!(availability, Availability::Unavailable);
}

#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    feature = "terminal"
))]
#[test]
fn linux_terminal_availability_rejects_directories_and_non_executable_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("directory-terminal")).unwrap();
    std::fs::write(dir.path().join("plain-terminal"), "not executable").unwrap();
    let search_path = std::env::join_paths([dir.path()]).unwrap();

    for command in ["directory-terminal", "plain-terminal"] {
        let availability = FileHandle::terminal_availability_with(
            [command],
            Some(&search_path),
            Some(std::ffi::OsStr::new(":0")),
            None,
        );

        assert_eq!(availability, Availability::Unavailable);
    }
}

#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    feature = "terminal"
))]
#[test]
fn linux_terminal_availability_reports_unavailable_for_headless_environment() {
    let dir = tempfile::tempdir().unwrap();
    make_executable_file(&dir.path().join("test-terminal"));
    let search_path = std::env::join_paths([dir.path()]).unwrap();

    let availability =
        FileHandle::terminal_availability_with(["test-terminal"], Some(&search_path), None, None);

    assert_eq!(availability, Availability::Unavailable);
}

#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "windows"),
    feature = "terminal"
))]
#[test]
fn linux_terminal_candidate_order_is_shared_with_launch_logic() {
    assert_eq!(
        super::linux_unix::TERMINAL_CANDIDATES,
        ["xdg-terminal-exec", "gnome-terminal", "konsole", "xterm"]
    );
}

#[cfg(all(
    unix,
    not(target_os = "macos"),
    not(target_os = "windows"),
    feature = "terminal"
))]
fn make_executable_file(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, "#!/bin/sh\n").unwrap();
    let mut permissions = std::fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).unwrap();
}

#[cfg(all(target_os = "windows", feature = "terminal"))]
#[test]
fn windows_trusted_cmd_resolution_accepts_absolute_comspec_file() {
    let dir = tempfile::tempdir().unwrap();
    let cmd = dir.path().join("cmd.exe");
    std::fs::write(&cmd, "").unwrap();

    let resolved = FileHandle::trusted_cmd_path_from(Some(cmd.as_os_str().to_os_string()), None);

    assert_eq!(resolved, Some(cmd));
}

#[cfg(all(target_os = "windows", feature = "terminal"))]
#[test]
fn windows_trusted_cmd_resolution_rejects_relative_comspec() {
    let resolved =
        FileHandle::trusted_cmd_path_from(Some(std::ffi::OsString::from("cmd.exe")), None);

    assert_eq!(resolved, None);
}

#[cfg(all(target_os = "windows", feature = "terminal"))]
#[test]
fn windows_trusted_cmd_resolution_rejects_directory_comspec() {
    let dir = tempfile::tempdir().unwrap();

    let resolved =
        FileHandle::trusted_cmd_path_from(Some(dir.path().as_os_str().to_os_string()), None);

    assert_eq!(resolved, None);
}

#[cfg(all(target_os = "windows", feature = "terminal"))]
#[test]
fn windows_trusted_cmd_resolution_falls_back_to_system_root() {
    let root = tempfile::tempdir().unwrap();
    let system32 = root.path().join("System32");
    std::fs::create_dir(&system32).unwrap();
    let cmd = system32.join("cmd.exe");
    std::fs::write(&cmd, "").unwrap();

    let resolved = FileHandle::trusted_cmd_path_from(
        Some(std::ffi::OsString::from("cmd.exe")),
        Some(root.path().as_os_str().to_os_string()),
    );

    assert_eq!(resolved, Some(cmd));
}

#[cfg(all(target_os = "windows", feature = "terminal"))]
#[test]
fn windows_trusted_cmd_resolution_rejects_missing_or_non_file_fallback() {
    let root = tempfile::tempdir().unwrap();
    let system32 = root.path().join("System32");
    std::fs::create_dir(&system32).unwrap();

    let missing =
        FileHandle::trusted_cmd_path_from(None, Some(root.path().as_os_str().to_os_string()));
    assert_eq!(missing, None);

    std::fs::create_dir(system32.join("cmd.exe")).unwrap();
    let directory =
        FileHandle::trusted_cmd_path_from(None, Some(root.path().as_os_str().to_os_string()));
    assert_eq!(directory, None);
}

#[cfg(all(target_os = "windows", feature = "terminal"))]
#[test]
fn windows_trusted_cmd_resolution_does_not_use_arbitrary_path_lookup() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("cmd.exe"), "").unwrap();

    let resolved = FileHandle::trusted_cmd_path_from(None, None);

    assert_eq!(resolved, None);
}

#[cfg(any(feature = "open", feature = "show", feature = "trash"))]
fn missing_paths() -> Vec<PathBuf> {
    let base = std::env::temp_dir().join(format!("file-handle-missing-{}", std::process::id()));

    vec![base.with_extension("one"), base.with_extension("two")]
}

#[cfg(any(feature = "open", feature = "show", feature = "trash"))]
fn assert_not_found_failures(outcome: &BatchOutcome) {
    for (path, error) in &outcome.failed {
        assert!(matches!(error, FileHandleError::NotFound(failed) if failed == path));
    }
}
