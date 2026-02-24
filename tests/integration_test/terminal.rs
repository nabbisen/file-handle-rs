use file_handle::FileHandle;
use tempfile::tempdir;

#[test]
fn test_terminal_integration() {
    let dir = tempdir().unwrap();

    // ディレクトリでターミナルを開く試行
    // assert はしない
    // 理由 = CI 環境（Headless Linux 等）では標準ターミナルが見つからずエラーになる可能性がある
    //        結果の Ok / Err が環境に依存する
    let result = FileHandle::open_terminal(dir.path());
    println!("Terminal open result: {:?}", result);
}
