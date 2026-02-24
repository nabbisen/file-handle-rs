use file_handle::{FileHandle, FileHandleError};
use std::fs::File;
use std::path::Path;
use tempfile::tempdir;

/// 共通設定: 存在しないパスに対するエラーハンドリングのテスト
#[test]
fn test_not_found_error() {
    let non_existent_path = Path::new("this_file_really_should_not_exist_12345.txt");
    let result = FileHandle::show(non_existent_path);

    match result {
        Err(FileHandleError::NotFound(_)) => (),
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}

/// feature = "show" のテスト
#[test]
fn test_show_integration() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_file.txt");
    File::create(&file_path).unwrap();

    let result = FileHandle::show(&file_path);

    // CI 環境（特に Linux）では、サービスが見つからないエラーを許容する
    if let Err(e) = result {
        #[cfg(target_os = "linux")]
        {
            let err_str = e.to_string();
            if err_str.contains("ServiceUnknown") || err_str.contains("D-Bus error") {
                eprintln!("Skipping test: File manager service not available in this environment.");
                return;
            }
        }
        panic!("FileHandle::show failed with unexpected error: {:?}", e);
    }
}
