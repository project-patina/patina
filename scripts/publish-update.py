#!/usr/bin/env python3
"""Publish packages from scripts/package-update.sh in project-patina/patina-updates.

Writes latest.json, signs it, creates the GitHub release with the zips and only then
pushes the manifest, so launchers never see a version whose files are not there yet.
--dry-run stops before anything leaves this computer.
"""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = "project-patina/patina-updates"
PLATFORMS = ["linux-x86_64", "windows-x86_64"]
# Limits of crates/core/src/update.rs.
MAX_PACKAGE = 400 * 1024 * 1024
MAX_MANIFEST = 64 * 1024


def package_name(version, platform):
    return f"patina-{version}-{platform}.zip"


def package_url(version, platform):
    return f"https://github.com/{REPOSITORY}/releases/download/v{version}/{package_name(version, platform)}"


def sha256_of(path):
    digest = hashlib.sha256()
    with open(path, "rb") as file:
        for chunk in iter(lambda: file.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def build_manifest(version, notes_pl, notes_en, critical, date, dist):
    if not re.fullmatch(r"\d+\.\d+\.\d+", version):
        raise ValueError(f"Not a release version: {version}")
    if not notes_pl.strip() or not notes_en.strip():
        raise ValueError("Both notes are required")
    platforms = {}
    for platform in PLATFORMS:
        path = dist / package_name(version, platform)
        if not path.is_file():
            raise ValueError(f"Missing package: {path}")
        size = path.stat().st_size
        if not 0 < size <= MAX_PACKAGE:
            raise ValueError(f"Launchers refuse a package of this size: {path} ({size} bytes)")
        platforms[platform] = {"url": package_url(version, platform), "sha256": sha256_of(path), "size": size}
    manifest = {"version": version, "critical": critical, "date": date,
                "notes": {"pl": notes_pl, "en": notes_en}, "platforms": platforms}
    data = (json.dumps(manifest, indent=2, ensure_ascii=False) + "\n").encode()
    if len(data) > MAX_MANIFEST:
        raise ValueError("The manifest is over the limit launchers accept")
    return data


def sign(manifest, key):
    """Leaves latest.json.sig beside the manifest: base64 of a minisign signature."""
    subprocess.run(["npx", "tauri", "signer", "sign", "-f", str(key), "-p", "", str(manifest)],
                   cwd=ROOT, check=True, stdout=subprocess.DEVNULL)
    signature = manifest.with_name(manifest.name + ".sig")
    if not signature.is_file():
        raise ValueError("The signer wrote no signature")
    return signature


def publish(version, notes, manifest, signature, dist):
    packages = [str(dist / package_name(version, platform)) for platform in PLATFORMS]
    subprocess.run(["gh", "release", "create", f"v{version}", *packages, "--repo", REPOSITORY,
                    "--title", f"Patina {version}", "--notes", notes], check=True)
    with tempfile.TemporaryDirectory(prefix="patina-updates-") as temp:
        checkout = Path(temp) / "repository"
        subprocess.run(["git", "clone", "--depth", "1", f"https://github.com/{REPOSITORY}.git", str(checkout)], check=True)
        shutil.copy(manifest, checkout / "latest.json")
        shutil.copy(signature, checkout / "latest.json.sig")
        subprocess.run(["git", "add", "latest.json", "latest.json.sig"], cwd=checkout, check=True)
        subprocess.run(["git", "commit", "-m", f"Release {version}"], cwd=checkout, check=True)
        subprocess.run(["git", "push", "origin", "HEAD:main"], cwd=checkout, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("version")
    parser.add_argument("--notes-pl", required=True)
    parser.add_argument("--notes-en", required=True)
    parser.add_argument("--critical", action="store_true", help="launchers install this update without asking")
    parser.add_argument("--dry-run", action="store_true", help="stop before any network call")
    parser.add_argument("--key", type=Path, default=Path.home() / ".config/patina/update-signing.key")
    parser.add_argument("--dist", type=Path, default=ROOT / "dist-updates")
    args = parser.parse_args()
    try:
        date = datetime.now(timezone.utc).strftime("%Y-%m-%d")
        data = build_manifest(args.version, args.notes_pl, args.notes_en, args.critical, date, args.dist)
    except ValueError as error:
        parser.exit(1, f"{error}\n")
    manifest = args.dist / "latest.json"
    manifest.write_bytes(data)
    signature = sign(manifest, args.key)
    print(data.decode())
    if args.dry_run:
        print(f"Dry run: nothing was published. Signed manifest in {manifest}")
        return
    publish(args.version, args.notes_en, manifest, signature, args.dist)
    print(f"Published {args.version} in {REPOSITORY}")


if __name__ == "__main__":
    main()
