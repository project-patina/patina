#!/usr/bin/env python3
"""Validate and export public source. Never uploads or changes Git history."""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import tarfile
import tempfile


ROOT = Path(__file__).resolve().parents[1]
HORIZON_LICENSE_SHA256 = "b4b374ae9d0f2d8f7be35c6783e41327431da3b5662605da9ba31493630621df"
BLOCKED_DIRS = {".git", ".codex", ".claude", ".local", ".venv", "__pycache__",
                "node_modules", "target", "dist", "dist-windows", "exports",
                "backups", "instances", "saves", "crash-reports"}
BLOCKED_SUFFIXES = {".exe", ".dll", ".so", ".dylib", ".jar", ".class", ".jfr",
                    ".hprof", ".pem", ".key", ".p12", ".pfx", ".jks", ".keystore",
                    ".zip", ".tar", ".gz", ".tgz", ".7z", ".pyc"}
BLOCKED_NAMES = {"accounts.json", "settings.json", "hosts.yml", "dev-skin.png",
                 "dev-cape.png", "dev-cape2.png", "BACKUP.md", "REVIEW-0.2.0.md", "SOURCE_MANIFEST.json"}
SECRETS = re.compile(
    r"gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}"
    r"|sk-[A-Za-z0-9_-]{32,}|AKIA[A-Z0-9]{16}"
    r"|(?m:^-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----)"
)
REQUIRED = {"LICENSE", "THIRD_PARTY_NOTICES.md", "README.md", "CONTRIBUTING.md",
            "SECURITY.md", "Cargo.lock", "package-lock.json", "resources/news.json",
            "engine/LICENSE", "engine/LICENSE-PATINA", "engine/UPSTREAM.md",
            "engine/THIRD_PARTY_NOTICES.md", "engine/Cargo.lock", ".github/workflows/ci.yml",
            "LICENSES/fonts/Inter-OFL-1.1.txt", "LICENSES/fonts/Outfit-OFL-1.1.txt",
            "LICENSES/fonts/JetBrainsMono-OFL-1.1.txt", "LICENSES/lucide/ISC-and-Feather-MIT.txt",
            "LICENSES/skinview3d/MIT.txt", "LICENSES/three/MIT.txt"}


def digest(data):
    return hashlib.sha256(data).hexdigest()


def validate_file(relative, data):
    path = PurePosixPath(relative)
    if path.is_absolute() or ".." in path.parts or set(path.parts) & BLOCKED_DIRS:
        raise ValueError(f"Private/generated directory in export: {relative}")
    if path.name in BLOCKED_NAMES or path.suffix.lower() in BLOCKED_SUFFIXES:
        raise ValueError(f"Private/generated file in export: {relative}")
    if path.name == ".env" or (path.name.startswith(".env.") and path.name != ".env.example"):
        raise ValueError(f"Environment file in export: {relative}")
    if len(data) > 5 * 1024 * 1024:
        raise ValueError(f"File exceeds the 5 MiB source limit: {relative}")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        return
    if SECRETS.search(text):
        raise ValueError(f"Credential-like content in {relative}; review locally, never print it")
    if str(Path.home()) in text:
        raise ValueError(f"Personal home path in public file: {relative}")


def source_files(root=ROOT):
    result = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
                            cwd=root, capture_output=True, check=True)
    paths = sorted(set(result.stdout.decode().split("\0")) - {""})
    missing = REQUIRED - set(paths)
    if missing:
        raise ValueError("Required source/notices missing from Git selection: " + ", ".join(sorted(missing)))
    files = {}
    for relative in paths:
        path = root / relative
        if path.is_symlink() or not path.is_file():
            raise ValueError(f"Only regular source files may be exported: {relative}")
        data = path.read_bytes()
        validate_file(relative, data)
        files[relative] = data
    if digest(files["engine/LICENSE"]) != HORIZON_LICENSE_SHA256:
        raise ValueError("The original Horizon MIT license changed; review attribution before export")
    original = files["engine/LICENSE"].decode()
    if "Copyright (c) 2026 Ludovic Marie and the Horizon contributors" not in original:
        raise ValueError("The original Horizon copyright must remain intact")
    if files["engine/LICENSE-PATINA"] != files["LICENSE"]:
        raise ValueError("Patina's engine contribution notice must match the root MIT license")
    return files


def export(files, output):
    output = output.resolve()
    archive = Path(str(output) + ".tar.gz")
    checksum = Path(str(archive) + ".sha256")
    if output.is_relative_to(ROOT):
        raise ValueError("Choose an output outside the working repository")
    if any(p.exists() or p.is_symlink() for p in (output, archive, checksum)):
        raise ValueError("Output already exists; choose a new directory to preserve previous exports")
    output.parent.mkdir(parents=True, exist_ok=True)
    stage = Path(tempfile.mkdtemp(prefix=".patina-source-", dir=output.parent))
    try:
        for relative, data in files.items():
            target = stage / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            target.chmod(0o755 if relative.endswith(".sh") else 0o644)
        version = json.loads(files["package.json"])["version"]
        manifest = {"project": "Patina", "version": version, "format": 1,
                    "upstream": "https://github.com/horizon-rust/horizon",
                    "files": {p: digest(data) for p, data in files.items()}}
        (stage / "SOURCE_MANIFEST.json").write_text(json.dumps(manifest, indent=2) + "\n")
        stage.rename(output)
    finally:
        if stage.exists():
            shutil.rmtree(stage)
    with tarfile.open(archive, "w:gz") as tar:
        tar.add(output, arcname=output.name)
    checksum.write_text(f"{digest(archive.read_bytes())}  {archive.name}\n")
    return archive, checksum


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true")
    group.add_argument("--output", type=Path)
    args = parser.parse_args()
    try:
        files = source_files()
        print(f"Source selection OK: {len(files)} files, {sum(map(len, files.values())) / 1024 / 1024:.1f} MiB")
        print("Horizon MIT SHA-256: " + digest(files["engine/LICENSE"]))
        if args.output:
            archive, checksum = export(files, args.output)
            print(f"Source directory: {args.output.resolve()}\nArchive: {archive}\nChecksum: {checksum}")
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"Source release rejected: {error}\n")


if __name__ == "__main__":
    main()
