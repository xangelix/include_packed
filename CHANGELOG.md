# Changelog

## v0.1.6

This release resolves a critical linking issue for downstream binaries, updates internal build dependencies, and introduces repository-wide formatting rules.

### Added

- **Tooling:** Added an `.editorconfig` to the repository root to standardize whitespace and file formatting rules across different IDEs and contributors.

### Fixed

- **Downstream Linking:** Replaced the raw `rustc-link-arg` directive with the `cc` crate to bundle generated object files into static library archives (`.a` / `.lib`). This resolves `undefined symbol` errors by ensuring Cargo correctly propagates the packed data transitively to downstream binaries.

### Changed

- **Dependencies:** Upgraded the `object` crate build dependency to `v0.38`.
