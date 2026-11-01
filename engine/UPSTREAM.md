# Upstream and local modifications

This tree is a **modified derivative of Horizon**, originally created by
**Ludovic Marie and the Horizon contributors**.

- Canonical repository: <https://github.com/horizon-rust/horizon>
- Upstream project: <https://playhorizon.dev/>
- Original license: MIT, preserved verbatim in [LICENSE](LICENSE).
- Original copyright: `Copyright (c) 2026 Ludovic Marie and the Horizon contributors`.
- Patina contributions: MIT, [LICENSE-PATINA](LICENSE-PATINA), also covered by the
  [root license](../LICENSE).

The source was already locally modified when this repository was prepared.
Crates, commands and namespaces use `patina-*`; the separate launcher integration
is in `../crates/core/` and `../src-tauri/`. Renaming does not transfer authorship
of inherited work to Patina.

The exact upstream base commit is **not recorded** in this snapshot. No commit,
release tag, unmodified-source claim or complete inventory of pre-existing local
changes is implied. Future upstream imports should record a commit and preserve
an inspectable diff. The original MIT notice was checked against upstream on
2026-11-01.

Keep both MIT notices and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) when
redistributing this tree or substantial parts of it. Font, icon and vendored
library licenses remain applicable. The [root notices](../THIRD_PARTY_NOTICES.md)
describe the repository's overall license map.

Minecraft game files and generated resources are separate inputs, not Patina
source under MIT. Obtain the game legitimately and prepare resources locally.
Patina is independent; Horizon maintainers do not provide support for this fork.
