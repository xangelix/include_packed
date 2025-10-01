# `include-packed`

`include-packed` is a Rust crate that provides an efficient replacement for
`std::include_bytes!`. It is designed for projects that need to embed large
binary files without suffering from slow compile times and large executable sizes.

It combines the fast-compile approach of [`include-blob`] with `zstd` compression,
inspired by [`include-bytes-zstd`].

[![Crates.io](https://img.shields.io/crates/v/include_packed.svg)](https://crates.io/crates/include_packed)
[![Docs.rs](https://docs.rs/include_packed/badge.svg)](https://docs.rs/include_packed)

[`include-blob`]: https://crates.io/crates/include-blob
[`include-bytes-zstd`]: https://crates.io/crates/include-bytes-zstd

## How It Works

Instead of embedding file contents directly into your source code, this crate
processes files in a build script.

1.  **Build Script:** You use the `include_packed::Config` builder in your `build.rs`
    script. For native targets, it reads your asset files, compresses them with
    `zstd`, and creates linkable object files.
2.  **Macro Expansion:** The `include_packed!` macro in your code expands to an
    expression that links to the compressed data (on native) or embeds the
    compressed data directly (on Wasm).
3.  **Runtime:** At runtime, the expression decompresses the data and returns it
    as a `Vec<u8>`.

This method significantly reduces compile times for projects with large binary assets
and keeps the final executable size smaller.

## Usage

1.  Add `include_packed` to your `Cargo.toml`. The `build` feature is required for
    build-dependencies.

    ```toml
    [dependencies]
    include_packed = "0.1.0" # be sure to use the latest version

    [build-dependencies]
    include_packed = { version = "0.1.0", features = ["build"] } # be sure to use the latest version
    ```

2.  Create a `build.rs` file in your project root to prepare your assets.

    ```rust
    // build.rs
    fn main() {
        // This handles all platform-specific logic automatically.
        include_packed::Config::new("assets")
            .level(10) // Optional: set a zstd compression level (1-21)
            .build()
            .expect("Failed to pack assets");
    }
    ```

3.  Use the macro in your code to include an asset. The path must be relative
    to the crate root.

    ```rust
    // src/main.rs
    use include_packed::include_packed;

    fn main() {
        // This returns a Vec<u8> with the decompressed file content.
        let data: Vec<u8> = include_packed!("assets/my_file.txt");
        println!("Successfully included and decompressed {} bytes.", data.len());
    }
    ```

## License

This project is licensed under either of:

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgements

This crate is a combination of ideas from:
- [`include-blob`](https://github.com/SludgePhD/include-blob) by SludgePhD, licensed under 0BSD.
- [`include-bytes-zstd`](https://github.com/daac-tools/include-bytes-zstd) by Koichi Akabe, licensed under MIT/Apache-2.0.
