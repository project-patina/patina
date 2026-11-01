# Third-party notices

Patina's MIT license covers original Patina work and Patina modifications.
It does not replace the component licenses or third-party copyrights below.

## Horizon — Rust client and tools

`engine/` is a modified derivative of
[horizon-rust/horizon](https://github.com/horizon-rust/horizon), originally by
**Ludovic Marie and the Horizon contributors**.

- Original MIT license: [engine/LICENSE](engine/LICENSE), retained verbatim.
- Patina modifications: [engine/LICENSE-PATINA](engine/LICENSE-PATINA), MIT.
- Provenance: [engine/UPSTREAM.md](engine/UPSTREAM.md).
- Vendored engine code and artwork: [engine/THIRD_PARTY_NOTICES.md](engine/THIRD_PARTY_NOTICES.md).

The upstream base commit for this already-modified snapshot was not recorded.
This is not a claim to the latest upstream source or an endorsement by Horizon.

## Frontend fonts and libraries

| Component | License | Full notice |
| --- | --- | --- |
| Inter, via Fontsource | SIL OFL 1.1 | [Inter](LICENSES/fonts/Inter-OFL-1.1.txt) |
| Outfit, via Fontsource | SIL OFL 1.1 | [Outfit](LICENSES/fonts/Outfit-OFL-1.1.txt) |
| JetBrains Mono, via Fontsource | SIL OFL 1.1 | [JetBrains Mono](LICENSES/fonts/JetBrainsMono-OFL-1.1.txt) |
| Lucide Svelte 1.58.0, including Feather-derived icons | ISC and retained Feather MIT | [Lucide / Feather](LICENSES/lucide/ISC-and-Feather-MIT.txt) |
| skinview3d 3.4.2 | MIT | [skinview3d](LICENSES/skinview3d/MIT.txt) |
| Three.js 0.156.1 | MIT | [Three.js](LICENSES/three/MIT.txt) |

Other npm and Rust dependencies are resolved from their lockfiles and retain
their package licenses. This notice highlights bundled artwork and vendored code;
it is not a complete dependency bill of materials. Binary-release maintainers
must review licenses for the dependencies included in their selected build.

## Demonstration graphics and game content

`src/lib/demo/*.svg` are original geometric Patina fixtures under the root MIT
license. They are not extracted game textures. Demo accounts are synthetic;
the screenshots use those fixtures.

Minecraft binaries, extracted textures, audio, user account skins/capes and
downloaded packs are not covered by Patina's MIT license. They are excluded from
the source export. Runtime access to an image or Modrinth project does not grant
permission to relicense it.
