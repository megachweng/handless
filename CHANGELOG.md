# Changelog

All notable changes to this Windows-only internal branch are documented here.

## [Unreleased]

### Changed

- Removed updater integration, update checks, updater settings, updater tray items, and updater signing artifacts.
- Disabled installer bundling for production builds; release builds now produce the main executable without NSIS or MSI packaging.
- Cleaned non-Windows platform-specific configuration, code paths, and user-facing documentation for this branch.
- Removed source-code and attribution rows from the settings UI.

### Fixed

- Production builds now use the real MinGW binutils path on Windows to avoid `dlltool.exe` shim failures.
