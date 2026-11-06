# Changelog

## 0.4.6 — 2026-11-06

### Fixed

- Small fixes in the Patina Client: the shulker box preview is drawn over the screen, a key
  that cannot be stored no longer unbinds Zoom or Perspective, and while zoomed a number key
  no longer saves or loads a creative toolbar.
- The Fabric client asks for Fabric Loader 0.19.3, which its Fabric API needs.

## 0.4.5 — 2026-11-05

### Added

- Eleven new built-in mods in the Patina Client, on Fabric and in the Rust client, each
  with its own switch and settings: AppleSkin, Shulker Tooltip, Speedometer, Zoom (C),
  Ping, Perspective (V), Horse Stats, Status Effects, Weather Changer, Death Info and
  No Fog. Zoom and Perspective take any key and can be held or toggled; Death Info shows
  the dimension and time of death and copies the coordinates.
- The Patina Client screen is a catalogue of all sixteen mods with a search, a settings
  page per mod and a reset for each.
- The Fabric client carries AppleSkin 3.0.10 and Fabric API 0.165.2 from Modrinth as
  nested jars; `scripts/build-java-mod.sh` downloads them, pinned by SHA-256. The Rust
  client draws AppleSkin's overlays itself.

### Changed

- The clients' loading animation steps every 16 ms instead of 80 and shows the Patina
  wordmark under the finished cube. The launcher's update screen keeps its lighter sheet.

## 0.4.4 — 2026-11-05

### Changed

- New logo: the cube the loading animation builds, drawn from the same geometry and
  colours by `scripts/render-logo.mjs`. It replaces the old one in the launcher, its
  opening screen, the app icons on every platform, the Rust client's window, the Fabric
  mod and the site. On the opening screen the copper layer rises into place and the
  patina layer lands on it.
- This update is marked critical: launchers install it at start without asking.

## 0.4.3 — 2026-11-04

### Added

- Discord activity: Discord shows "Playing Patina" with what the player is doing. In the
  launcher, the page (home, servers, mods, skins, settings) or "Idle" after five minutes
  without input; in game, the Minecraft version and the main menu, a single player world
  or the server's name. Both clients report where the player is on their output, the same
  lines from Fabric and Rust, and only the launcher talks to the Discord app, through its
  local socket. Settings → General → Discord turns it off or hides the server; an IP
  address is never shown.

### Fixed

- Updating the launcher on Windows failed with "Disk error: Access is denied (os error 5)"
  and left the old version in place. Each copied file was flushed through a read-only
  handle, which Windows refuses; it is now flushed through a writable one. Renames and
  removals are tried again for a few seconds while an antivirus or the search indexer
  still holds a fresh file, a disk error names the file, and the running launcher is
  swapped like the other files, so a failure puts it back as well. Windows installs of
  0.4.2 and older need to download 0.4.3 by hand once.

## 0.4.2 — 2026-11-04

### Fixed

- The update screen's animation drew skewed squares and dark seams between the blocks
  (CSS 3D faces in WebKitGTK). It now plays the frames both game clients play while
  loading, from the same sprite sheet, which `scripts/render-splash.mjs` also writes for
  the launcher.

## 0.4.1 — 2026-11-04

### Fixed

- Rust client: preparing a fresh install failed with "is not a 26.3 version file Patina
  knows" since Mojang republished 26.3's version file with a new asset index. A version
  file of the pinned release (the same id, jars and Java, and only Mojang's own fields) is
  now accepted whenever Mojang republishes it; a rewrite by another launcher is still
  refused with what to do.

## 0.4.0 — 2026-11-03

### Added

- Servers page: well-known public servers with their region, icon, message, players and
  ping (the game's server list ping, with SRV records), a search that also checks a typed
  address, and Play, which starts the chosen instance and joins the server once. The
  region follows the system's time zone and can be changed. Partner servers come later.
- HUD editor in both clients: the game at its real size (the world itself, or a stand-in
  with the real HUD), where each built-in mod is dragged to its place and resized by its
  corner or the mouse wheel. A mod keeps to a corner, an edge's middle or the middle.
- The Rust client's main menu, theme and Patina Client screen match the Fabric mod's.
- Integrated Patina client mod for Fabric 26.3, installed before each launch and shown in
  the content list. It needs `scripts/build-java-mod.sh`; the release scripts refuse to
  package a launcher without it. The Rust client has the same screens built in.
- "Patina mods" tab for Rust instances, from a signed index in `project-patina/patina-mods`.
  The index is verified before parsing; packages are pinned by SHA-256 and size.
- `scripts/install-dev.sh`: Patina Dev, a second app beside the stable one, with its own
  data folder and no update checks.

### Changed

- The Patina Client screen is a small panel: each mod with its switch, the FPS counter's
  options and the HUD editor. The title background is the launcher's black Mono banner;
  the background choice is gone.
- New instances and the default loader offer Fabric and Rust. Existing Quilt and Vanilla
  instances keep working, and a saved default of either becomes Fabric.

### Fixed

- Rust client: downloaded skins vanished after a server's resource pack (the local player
  turned invisible in third person, the first-person arm showed Steve's).

## 0.3.0 — 2026-11-01

### Added

- Launcher language setting, Polish or English, for the interface and backend messages.
- Self-updater for the portable packages. `latest.json` in `project-patina/patina-updates`
  is verified against its minisign signature before parsing; packages are verified by
  SHA-256 and size, swapped with backups and rolled back on failure. Critical updates
  install without asking. Installing is refused while a game runs or prepares.
- `scripts/package-update.sh` and `scripts/publish-update.py` for building and publishing updates.
- Compact Home cards with an image and sanitized Markdown messages, replacing the NEWS page.
- Announcement feed from `project-patina/patina-news`, in both languages, with bounded
  downloads and an offline cache. A single bundled message is the fallback.

### Changed

- The engine's ready marker names the `research-lock.json` it was prepared for: a changed
  lock prepares resources again. Existing installs prepare once more after this update.
- Motion, dialogs and settings layout reworked.

### Removed

- Cinematic backdrop playback, avatar position and pose settings, wallpaper dimming
  and the blur setting. Old settings files keep loading; these fields are ignored.

### Fixed

- Standalone builds embed the UI through Tauri custom-protocol; release builds that would
  depend on a local Vite server no longer compile.
- A malformed percent escape in a `media:` URL could panic the protocol handler.

## 0.2.0 — 2026-11-01

### Added

- Six color palettes with dark, light and system modes.
- Offline NEWS with release notes bundled with the launcher.
- MIT licensing for Patina work, preserved upstream attribution and third-party notices.
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
history of all inherited changes.
