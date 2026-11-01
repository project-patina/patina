#!/bin/sh
# Builds engine/ and installs it where the launcher looks for it.
# Game resources are prepared later by the launcher, at the first start of a Rust instance.
set -eu

repo=$(cd "$(dirname "$0")/.." && pwd)
dest=${AMETHYST_ENGINE_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/amethyst/engine}

cd "$repo/engine"
cargo build -j 6 --release --locked -p patina-client --bin patina-client --bin patina-ctl -p patina-prepare --bin patina-prepare

mkdir -p "$dest/mods/patina/src/test"
cp -f target/release/patina-client target/release/patina-ctl target/release/patina-prepare research-lock.json LICENSE THIRD_PARTY_NOTICES.md "$dest/"
# The exporters patina-prepare compiles against the player's own copy of the game.
rm -rf "$dest/mods/patina/src/test/java"
cp -r mods/patina/src/test/java "$dest/mods/patina/src/test/java"
python3 "$repo/scripts/collect-licenses.py" --engine-only "$dest"

echo "Engine installed in $dest"
