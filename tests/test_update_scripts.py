import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("publish_update", ROOT / "scripts/publish-update.py")
publish = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publish)


class UpdateManifest(unittest.TestCase):
    def packages(self, dist, version="0.3.0"):
        for platform in publish.PLATFORMS:
            (dist / publish.package_name(version, platform)).write_bytes(platform.encode())

    def test_manifest_matches_what_the_launcher_reads(self):
        with tempfile.TemporaryDirectory() as temp:
            dist = Path(temp)
            self.packages(dist)
            manifest = json.loads(publish.build_manifest("0.3.0", "Nowości", "News", True, "2026-11-01", dist))
        self.assertEqual(set(manifest), {"version", "critical", "date", "notes", "platforms"})
        self.assertTrue(manifest["critical"])
        self.assertEqual(manifest["notes"], {"pl": "Nowości", "en": "News"})
        linux = manifest["platforms"]["linux-x86_64"]
        self.assertEqual(linux["sha256"], hashlib.sha256(b"linux-x86_64").hexdigest())
        self.assertEqual(linux["size"], len("linux-x86_64"))
        self.assertTrue(linux["url"].startswith("https://github.com/project-patina/patina-updates/releases/download/v0.3.0/"))

    def test_refuses_what_launchers_would_refuse(self):
        with tempfile.TemporaryDirectory() as temp:
            dist = Path(temp)
            self.packages(dist)
            for version, pl, en in [("0.3", "a", "b"), ("v0.3.0", "a", "b"), ("0.3.0", " ", "b"), ("0.3.0", "a", ""), ("0.4.0", "a", "b")]:
                with self.subTest(version=version), self.assertRaises(ValueError):
                    publish.build_manifest(version, pl, en, False, "2026-11-01", dist)
            with self.assertRaises(ValueError):
                publish.build_manifest("0.3.0", "a" * publish.MAX_MANIFEST, "b", False, "2026-11-01", dist)


if __name__ == "__main__":
    unittest.main()
