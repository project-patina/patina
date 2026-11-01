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

Review inherited telemetry, multiplayer and native-mod functionality when
distributing a build. The [root README](../README.md) covers launcher development.

## License

Inherited Horizon work: [MIT, original authors](LICENSE). Patina modifications:
[MIT, Patina contributors](LICENSE-PATINA). Other components:
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
