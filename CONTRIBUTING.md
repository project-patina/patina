# Contributing to Patina

Thanks for helping with Patina. Small, focused changes are easiest to review.
Read the [README](README.md), [security guidance](SECURITY.md) and
[upstream attribution](engine/UPSTREAM.md) first.

## Before a change

Open an issue for a substantial change and explain the user problem. Keep the
launcher minimal: avoid adding repeated buttons, settings or background services
without a clear reason. Report the component (launcher or Rust client), OS, game
version, loader and reproduction steps when filing a bug. Sanitize logs.

## Development and validation

The launcher and engine are separate Cargo workspaces. Launcher checks:

```sh
npm ci
npm run check
npm test
npm run build
python3 -m unittest discover -s tests -p 'test_*.py'
python3 scripts/prepare-source-release.py --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

For engine changes, run the relevant engine checks and tests separately and say
which need local game resources. Do not claim that launcher tests validate engine
parity. Add meaningful regression tests for behavior changes; explain any check
you could not run. Include before/after screenshots for visible UI changes.

## AI-assisted contributions

AI-assisted and hand-written contributions are welcome. Patina is openly
vibecoded. Review what you submit, disclose substantial AI assistance in the PR,
explain the resulting behavior and include validation evidence. Generating code
is not a substitute for understanding its source or license.

## Licensing and provenance

Contributions of your original code and artwork are offered under this project's
MIT license unless explicitly agreed otherwise. Only submit work you have the
right to contribute. Preserve existing copyrights and third-party notices.
Code inherited in `engine/` keeps its original MIT attribution; Patina's added notice
covers Patina contributions. Record the upstream repository and revision when
importing future changes. Do not submit game files, decompiled game source,
extracted textures, private skins/capes or downloaded modpacks.

There is no CLA requirement in this repository. Please follow
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
