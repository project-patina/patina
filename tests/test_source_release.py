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
                     "runtime/game.jar", "backups/source.tar.gz", "../outside", "/absolute"]:
            with self.subTest(name=name), self.assertRaises(ValueError):
                release.validate_file(name, b"{}")

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
