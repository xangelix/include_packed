//! Procedural macro implementation for the `include_packed` crate. Do not use directly.
use std::{env, fs, path::PathBuf};

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{LitStr, parse_macro_input};

/// Includes a large, compressed binary file without high compile-time costs.
///
/// This macro takes a single string literal which must be a path to an asset
/// relative to the crate root (`CARGO_MANIFEST_DIR`).
///
/// It expands to an expression of type `Vec<u8>`, containing the
/// decompressed contents of the file.
///
/// # Build Dependencies
///
/// This macro requires a `build.rs` script to be configured for the consuming crate,
/// which must use the [`include_packed::Config`](https://docs.rs/include_packed/0.1.0/include_packed/build/struct.Config.html)
/// builder to prepare assets.
///
/// # Platform Specifics
///
/// - **Native (e.g., Linux, Windows, macOS):** The macro links to an object file
///   created by the build script, keeping `rustc`'s memory usage and compile times low.
/// - **Wasm (`wasm32`):** The macro reads the asset file at compile time, compresses it,
///   and embeds the bytes directly into the `.wasm` binary. This avoids the native
///   linking process but may result in higher compiler memory usage for the Wasm target.
///
/// # Panics
///
/// This macro will cause a compilation failure if:
/// - The build script has not been run correctly.
/// - The specified file path does not exist.
/// - Any of the intermediate files created by the build script are missing or corrupt.
#[proc_macro]
pub fn include_packed(input: TokenStream) -> TokenStream {
    let lit_str = parse_macro_input!(input as LitStr);

    // Read the environment variable set by the build script to determine the target.
    env::var("INCLUDE_PACKED_TARGET_ARCH").map_or_else(|_| syn::Error::new(
                lit_str.span(),
                "include_packed: build script has not run. This is expected during analysis (e.g., by rust-analyzer).",
            )
            .to_compile_error()
            .into(), |target_arch| if target_arch == "wasm32" {
                // We are building for Wasm.
                get_tokens_wasm(&lit_str).into()
            } else {
                // We are building for a native target.
                get_tokens_native(&lit_str).into()
            })
}

/// Wasm implementation: Reads, compresses, and embeds the file inside the macro itself.
fn get_tokens_wasm(lit_str: &LitStr) -> TokenStream2 {
    use proc_macro_crate::{FoundCrate, crate_name};
    use proc_macro2::Span;
    use syn::Ident;

    let path_str = lit_str.value();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is not set; this macro must be run by Cargo.");
    let path = PathBuf::from(manifest_dir).join(&path_str);

    let content = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) => {
            let msg = format!(
                "include_packed: could not read file '{}' for wasm target: {err}",
                path.display()
            );
            return syn::Error::new(lit_str.span(), msg).to_compile_error();
        }
    };

    let compressed_content = zstd::encode_all(&*content, zstd::DEFAULT_COMPRESSION_LEVEL)
        .expect("zstd compression failed in proc-macro");
    let compressed_len = compressed_content.len();

    let crate_name = match crate_name("include_packed") {
        Ok(FoundCrate::Name(name)) => Ident::new(&name, Span::call_site()),
        Ok(FoundCrate::Itself) => Ident::new("crate", Span::call_site()),
        Err(_) => Ident::new("include_packed", Span::call_site()), // Fallback
    };

    quote! {
        {
            const COMPRESSED_DATA: [u8; #compressed_len] = [#(#compressed_content),*];
            #crate_name::decompress(&COMPRESSED_DATA)
        }
    }
}

/// Native implementation: Uses build script artifacts (.len file and linked .o file).
fn get_tokens_native(lit_str: &LitStr) -> TokenStream2 {
    use proc_macro_crate::{FoundCrate, crate_name};
    use proc_macro2::Span;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use syn::Ident;

    let out_dir =
        env::var("OUT_DIR").expect("OUT_DIR is not set; this macro must be run by Cargo.");

    let path_str = lit_str.value();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is not set; this macro must be run by Cargo.");
    let mut path = PathBuf::from(&manifest_dir);
    path.push(&path_str);

    let canonical_path = path
        .canonicalize()
        .unwrap_or_else(|e| panic!("Could not find file '{}': {e}", path.display()));
    let path_for_hashing = PathBuf::from(&path_str);
    let metadata = fs::metadata(&canonical_path).unwrap_or_else(|e| {
        panic!(
            "Could not read metadata for '{}': {e}",
            canonical_path.display()
        )
    });
    let modified_time = metadata.modified().unwrap_or_else(|e| {
        panic!(
            "Could not read modification time for '{}': {e}",
            canonical_path.display()
        )
    });

    let mut hasher = DefaultHasher::new();
    path_for_hashing.hash(&mut hasher);
    modified_time.hash(&mut hasher);
    let unique_name = format!("include_packed_{:016x}", hasher.finish());

    let len_path = PathBuf::from(&out_dir).join(format!("{unique_name}.len"));
    let Ok(len_str) = fs::read_to_string(&len_path) else {
        let msg = format!(
            "include_packed: failed to read .len file for asset at '{path_str}'\nexpected at: {}",
            len_path.display()
        );
        return syn::Error::new(lit_str.span(), msg).to_compile_error();
    };

    let compressed_len: usize = len_str.parse().unwrap_or_else(|_| {
        panic!(
            "include_packed: corrupt .len file at '{}'",
            len_path.display()
        )
    });

    let crate_name = match crate_name("include_packed") {
        Ok(FoundCrate::Name(name)) => Ident::new(&name, Span::call_site()),
        Ok(FoundCrate::Itself) => Ident::new("crate", Span::call_site()),
        Err(_) => Ident::new("include_packed", Span::call_site()), // Fallback
    };

    quote! {
        {
            unsafe extern "C" {
                #[link_name = #unique_name]
                static STATIC: [u8; #compressed_len];
            }
            #crate_name::decompress(unsafe { &STATIC })
        }
    }
}
