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
#[cfg(any(
    feature = "open",
    feature = "show",
    feature = "trash",
    feature = "terminal"
))]
use crate::FileHandleError;

#[test]
fn batch_outcome_helpers_for_empty_outcome() {
    let outcome = BatchOutcome::default();

    assert!(outcome.all_ok());
    assert!(!outcome.any_failed());
    assert!(outcome.succeeded.is_empty());
    assert!(outcome.failed.is_empty());
}

#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
#[test]
fn operation_display_uses_public_operation_names() {
    use crate::Operation;

    assert_eq!(Operation::Open.to_string(), "open");
    assert_eq!(Operation::Show.to_string(), "show");
    assert_eq!(Operation::Terminal.to_string(), "terminal");
}

#[cfg(any(feature = "open", feature = "show", feature = "terminal"))]
#[test]
fn no_handler_available_display_is_stable() {
    use crate::Operation;

    let error = FileHandleError::NoHandlerAvailable {
        operation: Operation::Open,
        tried: vec!["missing-launcher".to_owned()],
    };

    assert_eq!(
        error.to_string(),
        r#"no OS handler available for open (tried: ["missing-launcher"])"#
    );
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
    use crate::Operation;

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
