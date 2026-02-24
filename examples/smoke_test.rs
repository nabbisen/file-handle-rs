// 開発者が目視で確認するための簡易ツール
// ```sh
// cargo run --example smoke_test --features show
// ```
fn main() {
    #[cfg(feature = "show")]
    {
        use file_handle::FileHandle;
        FileHandle::show(".").expect("Failed to open current dir");
        println!("Did the file manager open ?");
    }
    #[cfg(not(feature = "show"))]
    println!("show feature Should be activated.");
}
