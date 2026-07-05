use file_handle::{FileHandle, FileHandleError};
use std::fs::File;
use std::path::Path;
use tempfile::tempdir;

/// Verifies that a missing path returns NotFound.
#[test]
fn test_not_found_error() {
    let non_existent_path = Path::new("this_file_really_should_not_exist_12345.txt");
    let result = FileHandle::open_with_default(non_existent_path);

    match result {
        Err(FileHandleError::NotFound(_)) => (),
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}

/// Integration test for opening a file with its default application.
#[test]
#[ignore = "opens the system default application"]
fn test_open_file_integration() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_open.txt");
    File::create(&file_path).unwrap();

    let result = FileHandle::open_with_default(&file_path);

    // Headless CI environments may not have a suitable associated app.
    if let Err(e) = result {
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            if matches!(e, FileHandleError::NoHandlerAvailable { .. }) {
                eprintln!("Skipping test: No launcher available in this environment.");
                return;
            }
        }
        panic!(
            "FileHandle::open_with_default failed with unexpected error: {:?}",
            e
        );
    }
}

/// Integration test for opening a directory with its default app.
#[test]
#[ignore = "opens the system default application"]
fn test_open_directory_integration() {
    let dir = tempdir().unwrap();

    let result = FileHandle::open_with_default(dir.path());

    // Headless CI environments may not have a suitable associated app.
    if let Err(e) = result {
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            if matches!(e, FileHandleError::NoHandlerAvailable { .. }) {
                eprintln!("Skipping test: No launcher available in this environment.");
                return;
            }
        }
        panic!(
            "FileHandle::open_with_default (dir) failed with unexpected error: {:?}",
            e
        );
    }
}
