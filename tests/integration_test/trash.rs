use file_handle::FileHandle;
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_trash_integration() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("to_be_trashed.txt");
    File::create(&file_path).unwrap();

    // ゴミ箱移動を実行
    assert!(file_path.exists());
    assert!(FileHandle::trash(&file_path).is_ok());

    // 移動後は元のパスに存在しないはず
    assert!(!file_path.exists());
}
