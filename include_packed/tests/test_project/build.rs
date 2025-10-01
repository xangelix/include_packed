fn main() {
    include_packed::Config::new("blobs")
        .level(5)
        .build()
        .expect("Failed to pack assets");
}
