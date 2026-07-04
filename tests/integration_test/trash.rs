use file_handle::FileHandle;
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_trash_integration() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("to_be_trashed.txt");
    File::create(&file_path).unwrap();

    // Move the file to the trash.
    assert!(file_path.exists());
    assert!(FileHandle::trash(&file_path).is_ok());

    // The original path should be gone after trashing.
    assert!(!file_path.exists());
}
