//! Include large, compressed files in your binary without the high compile time cost.
//!
//! This crate provides the [`include_packed!`] macro as an efficient alternative to
//! `std::include_bytes!`. It combines the fast compile-time approach of `include-blob`
//! with the binary size reduction of `zstd` compression.
//!
//! ## How It Works
//!
//! Instead of embedding file contents directly into your source code, this crate
//! processes files in a build script.
//!
//! 1.  **Build Script:** You use the [`build::Config`] builder in your `build.rs` script. For native
//!     targets, it reads your asset files, compresses them with `zstd`, and creates
//!     linkable object files.
//! 2.  **Macro Expansion:** The `include_packed!` macro in your code expands to an
//!     expression that links to the compressed data (on native) or embeds the compressed
//!     data directly (on Wasm).
//! 3.  **Runtime:** At runtime, the expression decompresses the data and returns it
//!     as a `Vec<u8>`. Decompression is performed on each call.
//!
//! This method significantly reduces compile times for projects with large binary assets
//! and keeps the final executable size smaller.
//!
//! ## Usage
//!
//! 1. Add `include_packed` to your `Cargo.toml`. The `build` feature is required for
//!    build-dependencies.
//!
//! ```toml
//! [dependencies]
//! include_packed = "0.1.0"
//!
//! [build-dependencies]
//! include_packed = { version = "0.1.0", features = ["build"] }
//! ```
//!
//! 2. Create a `build.rs` file in your project root to prepare the assets.
//!
//! ```no_run
//! // build.rs
//! // This handles all platform-specific logic automatically.
//! include_packed::Config::new("assets")
//!   .level(10) // Set a custom zstd compression level (optional)
//!   .build()
//!   .expect("Failed to pack assets");
//! ```
//!
//! 3. Use the macro in your code to include an asset.
//!
//! ```no_run
//! // src/main.rs
//! use include_packed::include_packed;
//!
//! // This returns a Vec<u8> with the decompressed file content.
//! let data: Vec<u8> = include_packed!("assets/my_file.txt");
//! println!("Decompressed data is {} bytes long.", data.len());
//!
//! ```

#![doc(html_root_url = "https://docs.rs/include_packed/0.1.0")]

// Re-export the procedural macro.
pub use include_packed_macros::include_packed;

//
// ===== RUNTIME CODE =====
//

/// Decompresses data that was compressed at compile time.
///
/// This function is an implementation detail of the [`include_packed!`] macro and is not
/// intended to be called directly by user code. Its signature is not guaranteed to be stable.
///
/// # Panics
///
/// Panics if the provided data is not valid zstd-compressed data. This indicates a bug in
/// `include_packed` itself, as the data should always be valid if generated correctly.
#[doc(hidden)]
#[must_use]
pub fn decompress(compressed_data: &'static [u8]) -> Vec<u8> {
    zstd::decode_all(compressed_data).expect(
        "BUG: include_packed: failed to decompress compile-time data. This indicates a bug in the crate.",
    )
}

//
// ===== BUILD-TIME CODE =====
//

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "build")]
pub use build::Config;
