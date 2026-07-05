use file_handle::{FileHandle, FileHandleError};
use std::fs::File;
use std::path::Path;
use tempfile::tempdir;

/// Verifies missing-path error handling.
#[test]
fn test_not_found_error() {
    let non_existent_path = Path::new("this_file_really_should_not_exist_12345.txt");
    let result = FileHandle::show(non_existent_path);

    match result {
        Err(FileHandleError::NotFound(_)) => (),
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}

/// Integration test for feature = "show".
#[test]
#[ignore = "opens the system file manager"]
fn test_show_integration() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_file.txt");
    File::create(&file_path).unwrap();

    let result = FileHandle::show(&file_path);

    // Linux CI often lacks a file-manager D-Bus service.
    if let Err(e) = result {
        #[cfg(target_os = "linux")]
        {
            let err_str = e.to_string();
            if matches!(e, FileHandleError::NoHandlerAvailable { .. })
                || err_str.contains("ServiceUnknown")
                || err_str.contains("D-Bus error")
            {
                eprintln!("Skipping test: File manager service not available in this environment.");
                return;
            }
        }
        panic!("FileHandle::show failed with unexpected error: {:?}", e);
    }
}
