"""Offline regression gate for candidate preparation, before any Cargo build."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import unittest

SCRIPT = Path(__file__).with_name("t068_production_graph.py").resolve()


def git_blob(data):
    return hashlib.sha1(b"blob " + str(len(data)).encode() + b"\0" + data).hexdigest()


class PrepareTests(unittest.TestCase):
    def assert_statement_boundary(self, data):
        # Python accepts LF and CRLF; Git may translate a checkout on Windows.
        # Match the physical separator as stored, without altering source bytes.
        markers = [b"exist_ok=True)" + ending + b" shutil.copy2"
                   for ending in (b"\n", b"\r\n")]
        found = [marker for marker in markers if marker in data]
        self.assertEqual(sum(data.count(marker) for marker in markers), 1)
        return found[0]

    def test_source_compiles_and_statements_are_separate(self):
        data = SCRIPT.read_bytes()
        compile(data, str(SCRIPT), "exec")
        self.assert_statement_boundary(data)

    def test_previous_literal_newline_bug_is_detected(self):
        data = SCRIPT.read_bytes()
        correct = self.assert_statement_boundary(data)
        broken = b"exist_ok=True)" + bytes([92, 110]) + b" shutil.copy2"
        self.assertEqual(data.count(correct), 1)
        with self.assertRaises(SyntaxError):
            compile(data.replace(correct, broken), "historical-defect.py", "exec")

    def test_lf_and_crlf_checkouts_keep_the_corruption_guard(self):
        # Synthesized checkouts exercise both styles on EVERY native runner.
        # Conversion is restricted to test copies, never byte-identity checks.
        source = SCRIPT.read_bytes().replace(b"\r\n", b"\n")
        for ending in (b"\n", b"\r\n"):
            with self.subTest(ending=ending):
                data = source.replace(b"\n", ending)
                compile(data, "checkout.py", "exec")
                correct = self.assert_statement_boundary(data)
                broken = b"exist_ok=True)" + bytes([92, 110]) + b" shutil.copy2"
                mutated = data.replace(correct, broken, 1)
                self.assertNotEqual(mutated, data)
                with self.assertRaises(SyntaxError):
                    compile(mutated, "historical-defect.py", "exec")
                restored = json.loads(json.dumps({"content": data.decode("utf-8")}))["content"].encode("utf-8")
                self.assertEqual(restored, data)
                self.assertEqual(git_blob(restored), git_blob(data))

    def test_json_round_trip_preserves_exact_source_bytes(self):
        data = SCRIPT.read_bytes()
        restored = json.loads(json.dumps({"content": data.decode("utf-8")}))["content"].encode("utf-8")
        self.assertEqual(restored, data)
        self.assertEqual(git_blob(restored), git_blob(data))
        compile(restored, str(SCRIPT), "exec")

    def test_prepare_on_clean_checkout_with_spaces(self):
        with tempfile.TemporaryDirectory(prefix="pulqva prepare test ") as owned:
            base = Path(owned)
            root = base / "source tree"
            scratch = base / "scratch space"
            scratch.mkdir()
            manifest = root / "crates/pulqva-discovery/Cargo.toml"
            manifest.parent.mkdir(parents=True)
            original = b'[package]\nname="fixture"\nversion="0.0.0"\n[dependencies]\n'
            manifest.write_bytes(original)
            (root / "Cargo.toml").write_text('[workspace]\nmembers=["crates/pulqva-discovery"]\n', encoding="utf-8")
            lock = b"version = 4\n"
            (root / "Cargo.lock").write_bytes(lock)
            source = root / "tools/t068_https_executor.rs"
            source.parent.mkdir()
            source.write_bytes(b"fn main() {}\n")
            self.assertFalse((manifest.parent / "examples").exists())
            env_file = base / "github env"
            env_file.write_bytes(b"")
            env = dict(os.environ, GITHUB_ENV=str(env_file), TMPDIR=str(scratch), TMP=str(scratch), TEMP=str(scratch))
            result = subprocess.run([sys.executable, str(SCRIPT), "prepare"], cwd=root,
                                    env=env, capture_output=True, text=True, timeout=15)
            self.assertEqual(result.returncode, 0, result.stderr)
            name, value = env_file.read_text().strip().split("=", 1)
            self.assertEqual(name, "T068_PROD")
            candidate = Path(value).resolve()
            self.assertTrue(candidate.is_relative_to(scratch.resolve()))
            self.assertEqual((candidate / "Cargo.lock").read_bytes(), lock)
            self.assertEqual((candidate / "crates/pulqva-discovery/examples/t068_https_executor.rs").read_bytes(), source.read_bytes())
            deps = tomllib.loads((candidate / "crates/pulqva-discovery/Cargo.toml").read_text())["dependencies"]
            self.assertEqual(deps["reqwest"]["version"], "=0.13.5")
            self.assertEqual(deps["reqwest"]["features"], ["rustls-no-provider", "socks"])
            self.assertFalse(deps["reqwest"]["default-features"])
            self.assertEqual(deps["rustls"]["version"], "=0.23.43")
            self.assertEqual(deps["tokio"]["version"], "=1.53.1")
            self.assertEqual(deps["webpki-roots"], "=1.0.9")
            self.assertEqual(manifest.read_bytes(), original)
            self.assertEqual((root / "Cargo.lock").read_bytes(), lock)
            self.assertFalse((manifest.parent / "examples").exists())


if __name__ == "__main__":
    unittest.main()
