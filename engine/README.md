# Patina Client — Horizon-derived Rust engine

The optional Rust client, server, command-line tools and resource preparer live
here. They derive from [Horizon](https://github.com/horizon-rust/horizon), by
**Ludovic Marie and the Horizon contributors**, under the original MIT notice in
[LICENSE](LICENSE). Read [UPSTREAM.md](UPSTREAM.md) before redistributing them.

This snapshot targets **Minecraft: Java Edition 26.3**. It is a separate program,
not a Fabric/Quilt runtime. Gameplay, graphics and world compatibility need testing
for your particular build. Back up worlds before trying a different client.

## Build and install

Use current stable Rust; some crates require Rust 1.90+. Linux default client
features need ALSA and udev development libraries (`libasound2-dev` / `libudev-dev`
on Debian-derived systems, `alsa-lib` / `systemd-libs` on Arch). Running the client
also needs working GPU drivers. Resource preparation needs the supported game
installation and an appropriate Java/JDK runtime.

From this directory:

```sh
cargo check --locked -p patina-client -p patina-prepare
cargo build --locked --release -p patina-client --bin patina-client --bin patina-ctl -p patina-prepare --bin patina-prepare
```

Or from the repository root:

```sh
sh scripts/install-engine.sh
```

`AMETHYST_ENGINE_DIR` overrides the installation directory. Create a Rust instance
in the launcher to prepare resources locally. Generated resources and game files
must stay out of Git and source archives.

The root Cargo workspace excludes this engine workspace. Root launcher tests do
not test the client. Many engine parity/exporter tests need the pinned game and
generated resources; a source-only CI check is not the full parity suite.

## Layout

- `crates/`: client, server, renderer, world and support libraries.
- `mods/patina/src/test/java/`: reference/exporter helper sources.
- `research-lock.json`: preparation versions and checksums.
- `third_party/wgpu-hal/`: vendored graphics code, MIT OR Apache-2.0.
- `crates/patina-theme/fonts/`: bundled fonts with individual OFL notices.

Review inherited native-mod functionality when distributing a build. The
[root README](../README.md) covers launcher development.

## Client changes on top of the inherited engine

- Start-up shows `crates/patina-client-render/src/boot_splash.rs`: a black screen
  with Patina's animation (`crates/patina-theme/splash/`, made by
  `scripts/render-splash.mjs`) until the title screen is ready. No Mojang logo is
  drawn. `PATINA_SPLASH_SHOTS=<folder>` saves its frames, for testing.
- The title screen's buttons are drawn by code (`menu_button` in
  `crates/patina-client/src/title.rs`), and it names the client's version: the
  launcher's workspace version (`crates/patina-core/build.rs`), or `PATINA_VERSION`.
- Options is vanilla's. Far terrain, resolution scale, anti-aliasing, HDR, battery mode, the
  graphics API and the FPS overlay are rows in Video Settings; controllers hang off Controls.
  `.pmod` mods load from `patina-mods/` as `patina-mods.txt` lists them, with no screen.
- No friends sharing, relay, Discord presence, mod registry, or "I'm playing" ping.
  `patina-telemetry` stays for local crash and performance summaries, and sends
  nothing unless `PATINA_TELEMETRY_URL` names a server. `patina-server` has no update
  source and `crates/patina-release-sign/release-keys.txt` lists no key until Patina
  publishes server builds.

## Mods

The client loads content mods from `patina-mods/*.pmod` in the game folder (`*.pmod`
zip files, or unpacked folders with their manifest), off unless `patina-mods.txt` turns
them on; the client has no screen for them. Modpacks are `.ppack` files
and shader packages `.pshader`.

## License

Inherited Horizon work: [MIT, original authors](LICENSE). Patina modifications:
[MIT, Patina contributors](LICENSE-PATINA). Other components:
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
