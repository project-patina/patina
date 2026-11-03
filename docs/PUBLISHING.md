# Publishing the first Patina repository

The intended repository name for these prepared files is `project-patina/patina`.
No repository is created and no source is uploaded by the preparation scripts.
If you choose a different name, update repository metadata and links first.

## Contents and licenses

- Original launcher code and Patina modifications: root `LICENSE`, MIT.
- Client in `engine/`: unchanged `engine/LICENSE`, with its original copyright.
- Patina changes to the engine: `engine/LICENSE-PATINA`, MIT.
- Fonts, icons and vendored components: both `THIRD_PARTY_NOTICES.md` files and
  their referenced full license texts. Preserve them all.

The source snapshot's upstream commit is unknown. Do not invent a base revision
or claim the Rust client was independently authored by Patina. AI-assisted work
is disclosed in the README. This release does not include Minecraft game files.

## Prepare and inspect

```sh
python3 scripts/prepare-source-release.py --check
python3 scripts/prepare-source-release.py --output ../patina-opensource
```

The output directory contains public source and `SOURCE_MANIFEST.json` with a
SHA-256 for every selected source file. The adjacent `.tar.gz` and `.sha256`
contain the same source tree. No existing output is overwritten. Verify with:

```sh
cd ..
sha256sum -c patina-opensource.tar.gz.sha256
```

The selection uses Git's tracked and unignored files, then applies additional
checks. It rejects credentials, known token patterns, local home paths, private
work logs, game archives, build output and unreviewed original demo PNGs. It does
not claim to detect every possible secret or copyrighted asset. Review the actual
file list, manifest, screenshots and notices before making the repository public.

Local backups and investigation notes stay in the working checkout and are not
part of the export. They are not removed by the preparation process.

## Publish only after reviewing the output

These are manual publication commands, not steps run by the exporter. Configure
your Git author identity first; use your GitHub-provided no-reply email if desired.
The final command makes the source public, so inspect the staged files first.

```sh
cd ../patina-opensource
git init -b main
git add .
git diff --cached --stat
git status --short
git commit -m "Prepare Patina launcher and client for open source"
gh repo create project-patina/patina --public --source=. --remote=origin --push
```

Suggested description:

> Open-source, vibecoded Minecraft launcher in Svelte and Tauri, with an optional Rust client. Minimal UI, themes and instance management. MIT licensed.

Suggested topics: `minecraft`, `launcher`, `rust`, `tauri`, `svelte`, `typescript`,
`open-source`, `vibecoding`.

## Patina mods

Rust instances list mods from `project-patina/patina-mods`. Its `main` branch holds
`index.json` and `index.json.sig`, a `tauri signer sign` signature made with the update
key. Packages are `.pmod` files in that repository's releases, at most 64 MiB.

```json
{
  "schema": 1,
  "mods": [{
    "id": "minimap", "name": "Minimap", "version": "1.0.0", "mcVersion": "26.3",
    "summary": { "pl": "Mapa okolicy", "en": "A map of the area" },
    "icon": "https://...",
    "url": "https://github.com/project-patina/patina-mods/releases/download/minimap-1.0.0/minimap.pmod",
    "sha256": "...", "size": 123456
  }]
}
```

`id` must equal the id in the mod's `patina-mod.toml`, because the launcher turns the
mod on under that name. `icon` is optional. One invalid entry rejects the whole index.

After publication, enable private vulnerability reporting and review repository
settings. CI definitions are included, but their first GitHub-hosted run is not
verified until they run in the new repository.

## Binary releases

Source exports contain no executables or prepared game resources. Build binaries
separately. `scripts/collect-licenses.py` copies the root notices and vendored/font
license texts into a distribution folder. Review transitive dependency license
obligations for the selected build as well. Do not bundle local accounts, worlds,
Minecraft jars, extracted assets or private skins/capes.
