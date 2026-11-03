#!/bin/sh
# Builds and installs Patina Dev: a second app beside the stable Patina, for looking at a
# build before it is published. It has its own data folder, identifier and desktop entry,
# and never checks for updates. Stable's program, data and desktop entry are not touched.
set -eu

repo=$(cd "$(dirname "$0")/.." && pwd)
data=${XDG_DATA_HOME:-$HOME/.local/share}
opt="$HOME/.local/opt/patina-dev"
root="$data/patina-dev"
entry="$data/applications/patina-dev.desktop"
# Its own target folder: stable's target/release stays what it was.
target="$repo/target/patina-dev"
web="$target/web"

if [ ! -f "$repo/resources/patina-mod/patina-client-26.3.jar" ]; then
    echo "The integrated mod is missing: run scripts/build-java-mod.sh" >&2
    exit 1
fi

# Merged over tauri.conf.json. Lists are replaced whole, so the window is written out.
config=$(python3 - "$repo/src-tauri/tauri.conf.json" <<'PY'
import json, sys

conf = json.load(open(sys.argv[1]))
window = dict(conf["app"]["windows"][0], title="Patina Dev")
print(json.dumps({
    "productName": "Patina Dev",
    "identifier": "dev.amethyst.launcher.dev",
    "build": {"frontendDist": "../target/patina-dev/web"},
    "app": {"windows": [window]},
}))
PY
)

cd "$repo"
VITE_PATINA_DEV=1 npx vite build --outDir "$web" --emptyOutDir
# Cargo cannot see the frontend folder: this is what makes it embed the new files.
touch src-tauri/src/lib.rs
PATINA_DEV=1 TAURI_CONFIG=$config CARGO_TARGET_DIR="$target" cargo build -j 6 --release --locked -p amethyst-launcher --features custom-protocol

mkdir -p "$opt" "$root" "$(dirname "$entry")"
# A rename, so a running Patina Dev keeps its file.
cp "$target/release/amethyst-launcher" "$opt/patina-dev.new"
mv -f "$opt/patina-dev.new" "$opt/patina-dev"
cp "$repo/src-tauri/icons/icon.png" "$opt/patina-dev.png"
AMETHYST_ENGINE_DIR="$root/engine" sh "$repo/scripts/install-engine.sh"

cat > "$entry.new" <<DESKTOP
[Desktop Entry]
Type=Application
Name=Patina Dev
GenericName=Game launcher
Comment=Launcher Minecraft z silnikiem Rust, wersja przed publikacją
Exec=$opt/patina-dev
TryExec=$opt/patina-dev
Icon=$opt/patina-dev.png
Terminal=false
Categories=Game;
StartupWMClass=patina-dev
DESKTOP
mv -f "$entry.new" "$entry"

echo "Patina Dev installed in $opt, data in $root"
