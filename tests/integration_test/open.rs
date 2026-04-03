use file_handle::{FileHandle, FileHandleError};
use std::fs::File;
use std::path::Path;
use tempfile::tempdir;

/// 存在しないパスに対して NotFound エラーが返ることを確認
#[test]
fn test_not_found_error() {
    let non_existent_path = Path::new("this_file_really_should_not_exist_12345.txt");
    let result = FileHandle::open_with_default(non_existent_path);

    match result {
        Err(FileHandleError::NotFound(_)) => (),
        _ => panic!("Expected NotFound error, got {:?}", result),
    }
}

/// ファイルをデフォルトアプリで開く統合テスト
#[test]
fn test_open_file_integration() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_open.txt");
    File::create(&file_path).unwrap();

    let result = FileHandle::open_with_default(&file_path);

    // CI 環境（Headless Linux 等）では関連アプリが存在しない場合を許容する
    if let Err(e) = result {
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            let err_str = e.to_string();
            if err_str.contains("No suitable launcher found") {
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

/// ディレクトリをデフォルトアプリ（ファイルマネージャ等）で開く統合テスト
#[test]
fn test_open_directory_integration() {
    let dir = tempdir().unwrap();

    let result = FileHandle::open_with_default(dir.path());

    // CI 環境（Headless Linux 等）では関連アプリが存在しない場合を許容する
    if let Err(e) = result {
        #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
        {
            let err_str = e.to_string();
            if err_str.contains("No suitable launcher found") {
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
