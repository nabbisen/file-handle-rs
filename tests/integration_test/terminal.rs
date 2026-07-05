use file_handle::FileHandle;
use tempfile::tempdir;

#[test]
#[ignore = "opens the system terminal application"]
fn test_terminal_integration() {
    let dir = tempdir().unwrap();

    // Try opening a terminal in a directory without asserting the result.
    // Headless CI environments may not have a standard terminal available, so
    // Ok / Err is environment-dependent.
    let result = FileHandle::open_terminal(dir.path());
    println!("Terminal open result: {:?}", result);
}
