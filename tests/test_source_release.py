import importlib.util
from pathlib import Path
import tempfile
import unittest


spec = importlib.util.spec_from_file_location("source_release", Path(__file__).resolve().parents[1] / "scripts/prepare-source-release.py")
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class SourceReleaseSafety(unittest.TestCase):
    def test_rejects_private_paths_even_if_force_added_to_git(self):
        for name in ["accounts.json", "folder/settings.json", ".env", ".env.production",
                     "engine/.local/game.json", "engine/target/client", "public/dev-skin.png",
                     "runtime/game.jar", "dist-updates/latest.json", "backups/source.tar.gz", "../outside", "/absolute"]:
            with self.subTest(name=name), self.assertRaises(ValueError):
                release.validate_file(name, b"{}")

    def test_only_the_pinned_gradle_wrapper_may_be_a_jar(self):
        wrapper = "java-mod/gradle/wrapper/gradle-wrapper.jar"
        release.validate_file(wrapper, (release.ROOT / wrapper).read_bytes())
        with self.assertRaises(ValueError):
            release.validate_file(wrapper, b"another jar")

    def test_rejects_tokens_without_disclosing_their_value(self):
        token = "ghp_" + "a" * 36
        with self.assertRaises(ValueError) as error:
            release.validate_file("src/example.ts", token.encode())
        self.assertNotIn(token, str(error.exception))

    def test_rejects_actual_private_key_blocks(self):
        text = "-----BEGIN " + "RSA PRIVATE KEY-----\nexample\n"
        with self.assertRaises(ValueError):
            release.validate_file("notes.txt", text.encode())

    def test_allows_source_notices_and_explicit_env_example(self):
        for name in [".env.example", "LICENSE", "engine/LICENSE", "engine/crates/example/src/main.rs"]:
            release.validate_file(name, b"public source")
        release.validate_file("tests/fixture.rs", b'let key = "-----BEGIN RSA PRIVATE KEY-----\\nAAA=\\n";')

    def test_source_selection_handles_removed_tracked_files(self):
        import subprocess
        from unittest.mock import patch
        root = Path(__file__).resolve().parents[1]
        with tempfile.TemporaryDirectory() as temp:
            checkout = Path(temp)
            (checkout / "engine").mkdir()
            for name in ["LICENSE", "engine/LICENSE", "engine/LICENSE-PATINA"]:
                (checkout / name).write_bytes((root / name).read_bytes())
            (checkout / "removed.txt").write_text("old source")
            subprocess.run(["git", "init", "-q", temp], check=True)
            subprocess.run(["git", "add", "."], cwd=checkout, check=True)
            (checkout / "removed.txt").unlink()
            (checkout / "replacement.md").write_text("replacement")
            with patch.object(release, "REQUIRED", {"LICENSE", "engine/LICENSE", "engine/LICENSE-PATINA"}):
                files = release.source_files(checkout)
            self.assertNotIn("removed.txt", files)
            self.assertEqual(files["replacement.md"], b"replacement")

    def test_export_refuses_to_overwrite_an_existing_directory(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "public-source"
            output.mkdir()
            marker = output / "keep.txt"
            marker.write_text("keep")
            with self.assertRaises(ValueError):
                release.export({}, output)
            self.assertEqual(marker.read_text(), "keep")

    def test_archive_manifest_and_checksum_cover_the_export(self):
        import hashlib
        import json
        import tarfile
        files = {"package.json": b'{"version":"0.2.0"}', "LICENSE": b"original notice"}
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "source"
            archive, checksum = release.export(files, output)
            manifest = json.loads((output / "SOURCE_MANIFEST.json").read_text())
            self.assertEqual(manifest["files"]["LICENSE"], hashlib.sha256(files["LICENSE"]).hexdigest())
            self.assertTrue(checksum.read_text().startswith(hashlib.sha256(archive.read_bytes()).hexdigest()))
            with tarfile.open(archive) as tar:
                self.assertEqual(tar.extractfile("source/LICENSE").read(), files["LICENSE"])


if __name__ == "__main__":
    unittest.main()
