//! Build-script helpers for `include_packed`.
use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    string::FromUtf8Error,
};

use object::{
    Architecture, BinaryFormat, Endianness, SymbolFlags, SymbolKind, SymbolScope,
    write::{Object, StandardSection, Symbol, SymbolSection},
};

//
// ==================== PUBLIC BUILDER API ====================
//

/// A builder for configuring the asset packing process.
///
/// This provides a clean, high-level API for use in `build.rs` scripts.
///
/// # Example
/// ```no_run
/// // in build.rs
/// include_packed::Config::new("assets")
///   .level(5)
///   .build()
///   .expect("Failed to pack assets");
/// ```
#[derive(Debug)]
pub struct Config {
    path: PathBuf,
    level: i32,
}

impl Config {
    /// Creates a new configuration for a given asset path.
    ///
    /// The path can be a single file or a directory and should be relative to
    /// the crate root (`CARGO_MANIFEST_DIR`).
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            level: DEFAULT_COMPRESSION_LEVEL,
        }
    }

    /// Sets the zstd compression level (1-21).
    ///
    /// Higher levels provide better compression at the cost of slower build times.
    /// If not set, a default level of `3` is used.
    #[must_use]
    pub const fn level(mut self, level: i32) -> Self {
        self.level = level;
        self
    }

    /// Runs the asset packing process with the specified configuration.
    ///
    /// This is the final method that should be called in the builder chain.
    /// It handles all platform-specific logic internally, preparing assets for
    /// the [`include_packed!`](`crate::include_packed`) macro.
    ///
    /// # Errors
    /// Returns an [`Error`] if any part of the build process fails, such as file I/O
    /// or object file creation.
    pub fn build(self) -> Result<()> {
        // Get the target architecture from the environment variable Cargo provides.
        let target_arch =
            env::var("CARGO_CFG_TARGET_ARCH").map_err(|_| Error::Var("CARGO_CFG_TARGET_ARCH"))?;

        // Set an environment variable for the procedural macro to read. This is the
        // primary communication channel to determine the build strategy (native vs. wasm).
        println!("cargo:rustc-env=INCLUDE_PACKED_TARGET_ARCH={target_arch}");

        // Run the native asset packer ONLY if the target is not wasm32.
        if target_arch != "wasm32" {
            make_includable_with_level(&self.path, self.level)?;
        }
        Ok(())
    }
}

/// The default compression level used by [`make_includable`].
pub const DEFAULT_COMPRESSION_LEVEL: i32 = 6;

/// A specialized `Result` type for build script operations.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur during the asset packing process in a build script.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Object writing error")]
    Object(#[from] object::write::Error),
    #[error("Environment variable '{0}' not set by Cargo")]
    Var(&'static str),
    #[error("Path '{0}' not found (current directory is '{1}')")]
    PathNotFound(String, String),
    #[error("Path '{0}' has unsupported file type")]
    UnsupportedFileType(String),
    #[error("Could not convert object file name to UTF-8")]
    FromUtf8(#[from] FromUtf8Error),
    #[error("A generic build error occurred: {0}")]
    Generic(String),
}

/// Internal implementation that prepares a path for inclusion on native targets.
fn make_includable_with_level<P: AsRef<Path>>(path: P, level: i32) -> Result<()> {
    make_includable_impl(path.as_ref(), level)
}

/// Recursively processes files and directories.
fn make_includable_impl(path: &Path, level: i32) -> Result<()> {
    let canonical_path = path.canonicalize().map_err(|_| {
        Error::PathNotFound(
            path.display().to_string(),
            std::env::current_dir().map_or_else(|_| "unknown".into(), |p| p.display().to_string()),
        )
    })?;
    println!("cargo:rerun-if-changed={}", canonical_path.display());

    let metadata = fs::metadata(&canonical_path)?;
    if metadata.is_dir() {
        for entry in fs::read_dir(&canonical_path)? {
            make_includable_impl(&entry?.path(), level)?;
        }
        Ok(())
    } else if metadata.is_file() {
        process_file(&canonical_path, &metadata, level)
    } else {
        Err(Error::UnsupportedFileType(path.display().to_string()))
    }
}

/// Internal implementation that compresses and packs a single file into an object file.
fn process_file(path: &Path, metadata: &fs::Metadata, level: i32) -> Result<()> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map_err(|_| Error::Var("CARGO_MANIFEST_DIR"))?;

    let path_for_hashing = path
        .strip_prefix(&manifest_dir)
        .unwrap_or(path)
        .to_path_buf();

    let mut hasher = DefaultHasher::new();
    path_for_hashing.hash(&mut hasher);
    metadata.modified()?.hash(&mut hasher);
    let unique_name = format!("include_packed_{:016x}", hasher.finish());

    let content = fs::read(path)?;
    let compressed_content = zstd::encode_all(&*content, level)?;

    // Create the object file
    let info = TargetInfo::from_build_script_vars();
    let mut object = Object::new(info.binfmt, info.arch, info.endian);
    let section = object.add_subsection(StandardSection::ReadOnlyData, unique_name.as_bytes());

    let sym = object.add_symbol(Symbol {
        name: unique_name.as_bytes().to_vec(),
        value: 0,
        size: compressed_content.len() as u64,
        kind: SymbolKind::Data,
        scope: SymbolScope::Dynamic,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
    object.add_symbol_data(sym, section, &compressed_content, 1);
    let obj_buf = object.write()?;

    // Write the object file and instruct Cargo to link it
    let out_dir = env::var("OUT_DIR").map_err(|_| Error::Var("OUT_DIR"))?;

    let object_file_name = format!("{unique_name}.o");
    let object_path = PathBuf::from(&out_dir).join(object_file_name);
    fs::write(&object_path, obj_buf)?;

    let len_file_path = format!("{out_dir}/{unique_name}.len");
    fs::write(len_file_path, compressed_content.len().to_string())?;

    println!("cargo:rustc-link-arg={}", object_path.display());

    Ok(())
}

/// Internal helper to get target-specific information for object file creation.
struct TargetInfo {
    binfmt: BinaryFormat,
    arch: Architecture,
    endian: Endianness,
}

impl TargetInfo {
    fn from_build_script_vars() -> Self {
        let binfmt = match env::var("CARGO_CFG_TARGET_OS")
            .expect("CARGO_CFG_TARGET_OS not set")
            .as_str()
        {
            "macos" | "ios" => BinaryFormat::MachO,
            "windows" => BinaryFormat::Coff,
            "linux" | "android" | "freebsd" | "netbsd" | "openbsd" | "dragonfly" | "solaris"
            | "illumos" => BinaryFormat::Elf,
            unk => panic!("unhandled operating system '{unk}' for include-packed"),
        };
        let arch = match env::var("CARGO_CFG_TARGET_ARCH")
            .expect("CARGO_CFG_TARGET_ARCH not set")
            .as_str()
        {
            "x86" => Architecture::I386,
            "x86_64" => Architecture::X86_64,
            "arm" => Architecture::Arm,
            "aarch64" => Architecture::Aarch64,
            "riscv32" => Architecture::Riscv32,
            "riscv64" => Architecture::Riscv64,
            "mips" => Architecture::Mips,
            "mips64" => Architecture::Mips64,
            "powerpc" => Architecture::PowerPc,
            "powerpc64" => Architecture::PowerPc64,
            unk => panic!("unhandled architecture '{unk}' for include-packed"),
        };
        let endian = match env::var("CARGO_CFG_TARGET_ENDIAN")
            .expect("CARGO_CFG_TARGET_ENDIAN not set")
            .as_str()
        {
            "little" => Endianness::Little,
            "big" => Endianness::Big,
            unk => unreachable!("unhandled endianness '{unk}'"),
        };

        Self {
            binfmt,
            arch,
            endian,
        }
    }
}
