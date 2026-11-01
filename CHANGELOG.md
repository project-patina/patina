# Changelog

## 0.2.0 — 2026-11-01

### Added

- Six color palettes with dark, light and system modes.
- Offline NEWS with release notes bundled with the launcher.
- MIT licensing for Patina work, preserved Horizon attribution and third-party notices.
- Public-source documentation, contribution templates, CI and a validated source exporter.
- Original SVG demo textures and a synthetic preview account.

### Changed

- Simplified the home screen, recent instances and appearance settings.
- Removed repeated shortcuts and statistic cards.
- Kept detailed appearance controls in expandable sections.

### Fixed

- Ordered settings persistence, retry after errors and saving before closing/launching.
- Tauri close handling and the main window's required destroy permission.
- Instance version validation and preparation/running reservations.
- Download size verification, unique temporary files and cancellation handling.
- Thumbnail lifecycle errors and cleanup of game-event listeners.
- Failed system-trash operations preserve instance files instead of deleting them permanently.

The Rust engine's upstream base commit was not recorded in the imported snapshot.
This changelog describes Patina release preparation; it is not a reconstructed
history of all changes inherited from Horizon.
