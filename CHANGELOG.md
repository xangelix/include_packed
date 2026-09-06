# Changelog

## v0.1.7

This release upgrades internal dependencies across the workspace, loosens build dependencies, improves documentation and test reliability, and adds CI and funding configurations.

### Added

- **CI/CD:** Added a GitHub Actions workflow checking formatting and running tests.
- **Funding:** Added GitHub Sponsors funding configuration.
- **API:** Made `include_packed::build` a public module to expose build-script helper types (`Config`, `Error`, `Result`, and `DEFAULT_COMPRESSION_LEVEL`).

### Fixed

- **Documentation & Tests:** Annotated crate-level downstream examples as `ignore` so `cargo test` and doctests succeed out of the box without requiring external build setup.
- **Documentation:** Fixed broken intra-doc links in `build.rs`.

### Changed

- **Dependencies:** Upgraded `zstd` to `v0.14` and `object` to `v0.40` in `include_packed`.
- **Dependencies:** Upgraded `syn` to `v3` and `zstd` to `v0.14` in `include_packed_macros`.
- **Dependencies:** Loosened the `cc` build dependency requirement to `"1"`.

## v0.1.6

This release resolves a critical linking issue for downstream binaries, updates internal build dependencies, and introduces repository-wide formatting rules.

### Added

- **Tooling:** Added an `.editorconfig` to the repository root to standardize whitespace and file formatting rules across different IDEs and contributors.

### Fixed

- **Downstream Linking:** Replaced the raw `rustc-link-arg` directive with the `cc` crate to bundle generated object files into static library archives (`.a` / `.lib`). This resolves `undefined symbol` errors by ensuring Cargo correctly propagates the packed data transitively to downstream binaries.

### Changed

- **Dependencies:** Upgraded the `object` crate build dependency to `v0.38`.
