#!/usr/bin/env python3
"""Copy source notices into a binary distribution, preserving their paths."""
import argparse
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine-only", action="store_true")
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    destination = args.destination.resolve()
    if destination == ROOT or ROOT.is_relative_to(destination):
        parser.error("Choose a distribution directory, not the working repository or its parent")
    paths = [ROOT / "engine" / name for name in
             ["LICENSE", "LICENSE-PATINA", "UPSTREAM.md", "THIRD_PARTY_NOTICES.md"]]
    paths += list((ROOT / "engine/crates/patina-theme/fonts").glob("OFL-*.txt"))
    paths += [ROOT / "engine/crates/patina-theme/icons/pixel/LICENSE-pixelarticons.txt"]
    paths += [ROOT / "engine/third_party/wgpu-hal" / name for name in ["LICENSE.MIT", "LICENSE.APACHE"]]
    if not args.engine_only:
        paths += [ROOT / "LICENSE", ROOT / "THIRD_PARTY_NOTICES.md"]
        paths += [p for p in (ROOT / "LICENSES").rglob("*") if p.is_file()]
    for path in paths:
        base = ROOT / "engine" if args.engine_only else ROOT
        target = destination / path.relative_to(base)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)
    print(f"Copied {len(paths)} license/provenance files to {destination}")


if __name__ == "__main__":
    main()
