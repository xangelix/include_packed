use include_packed::include_packed;

fn main() {
    let original_content = "Contents of file.txt\n";

    // Macro returns a Vec<u8> with the decompressed data.
    let data_vec: Vec<u8> = include_packed!("blobs/file.txt");

    let s = std::str::from_utf8(&data_vec).expect("data is not valid UTF-8");
    println!("{s}");

    assert_eq!(s, original_content);
    assert_eq!(data_vec.as_slice(), original_content.as_bytes());

    println!("Decompressed data matches original.");
}
