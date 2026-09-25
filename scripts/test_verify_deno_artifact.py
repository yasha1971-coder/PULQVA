import hashlib
import io
from pathlib import Path
import stat
import warnings
import tempfile
import unittest
import zipfile

from verify_deno_artifact import unpack_verified, clean_environment, MAX_EXECUTABLE


class ArtifactTests(unittest.TestCase):
    def make_zip(self, names, symlink=False):
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, "w") as z:
            for name in names:
                info = zipfile.ZipInfo(name)
                info.create_system = 3
                info.external_attr = ((stat.S_IFLNK if symlink else stat.S_IFREG) | 0o755) << 16
                with warnings.catch_warnings():
                    warnings.simplefilter("ignore", UserWarning)
                    z.writestr(info, b"fixture executable")
        raw = stream.getvalue()
        return raw, {"bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}

    def test_valid_identity(self):
        raw, asset = self.make_zip(["deno"])
        with tempfile.TemporaryDirectory() as d:
            p, receipt = unpack_verified(raw, asset, "deno", Path(d))
            self.assertEqual(p.read_bytes(), b"fixture executable")
            self.assertEqual(receipt["sha256"], hashlib.sha256(p.read_bytes()).hexdigest())

    def test_wrong_hash_before_write(self):
        raw, asset = self.make_zip(["deno"])
        asset["sha256"] = "0" * 64
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(ValueError):
                unpack_verified(raw, asset, "deno", Path(d))
            self.assertEqual(list(Path(d).iterdir()), [])

    def test_unsafe_missing_duplicate_entries(self):
        for names, symlink in [(["../deno"], False), (["/deno"], False),
                               (["deno.exe"], False), ([], False),
                               (["deno", "other"], False), (["deno", "deno"], False),
                               (["deno"], True)]:
            with self.subTest(names=names, symlink=symlink):
                raw, asset = self.make_zip(names, symlink)
                with tempfile.TemporaryDirectory() as d:
                    with self.assertRaises(ValueError):
                        unpack_verified(raw, asset, "deno", Path(d))
                    self.assertEqual(list(Path(d).iterdir()), [])

    def test_expanded_size_rejected_before_write(self):
        from unittest.mock import patch
        raw, asset = self.make_zip(["deno"])
        with tempfile.TemporaryDirectory() as d, patch("verify_deno_artifact.MAX_EXECUTABLE", 1):
            with self.assertRaises(ValueError):
                unpack_verified(raw, asset, "deno", Path(d))
            self.assertEqual(list(Path(d).iterdir()), [])

    def test_environment_is_allowlisted(self):
        from unittest.mock import patch
        with patch.dict("os.environ", {"DENO_V8_FLAGS": "--bad", "HTTP_PROXY": "bad",
                                     "PATH": "bad", "SECRET": "bad"}, clear=True):
            env = clean_environment(Path("/owned"))
        self.assertFalse({"DENO_V8_FLAGS", "HTTP_PROXY", "PATH", "SECRET"} & env.keys())
        self.assertEqual(env["DENO_NO_UPDATE_CHECK"], "1")


if __name__ == "__main__":
    unittest.main()
