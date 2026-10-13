#!/usr/bin/env python3
"""Fail-closed preflight for the frozen v0.1 live acceptance manifest.

This is a manifest contract test, not live Tor or desktop E2E evidence.
"""
import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "apps/pulqva-desktop/PROTOTYPE_E2E_ADMISSION.json"
SHA40 = re.compile(r"[0-9a-f]{40}\Z")
EXPECTED = {"E2E-01", "E2E-02", "E2E-03", "E2E-04", "E2E-05"}


def no_duplicates(pairs):
    out = {}
    for key, value in pairs:
        if key in out:
            raise ValueError(f"duplicate key: {key}")
        out[key] = value
    return out


def verify_manifest(data):
    if data.get("protocol") != "pulqva-prototype-e2e-admission-v1":
        raise ValueError("unknown protocol")
    if data.get("status") != "PREPARED_NOT_EXECUTED":
        raise ValueError("preflight cannot certify a live result")
    if not isinstance(data.get("source_sha"), str) or not SHA40.fullmatch(data["source_sha"]):
        raise ValueError("invalid exact source SHA")
    fixed = data.get("fixed_inputs")
    if not isinstance(fixed, dict):
        raise ValueError("missing fixed inputs")
    if fixed.get("source") != "exact Git SHA above":
        raise ValueError("unexpected source policy")
    if fixed.get("provider") != "Wikimedia Commons" or fixed.get("query") != "countdown":
        raise ValueError("changed fixture")
    if fixed.get("selection_index") != 1 or fixed.get("max_candidates") != 10:
        raise ValueError("changed selection geometry")
    if fixed.get("max_media_bytes") != 8 * 1024 * 1024:
        raise ValueError("changed media limit")
    if fixed.get("network") != "Tor-only SOCKS5H":
        raise ValueError("changed privacy transport")
    if fixed.get("desktop_os") != ["windows-latest", "ubuntu-24.04"]:
        raise ValueError("missing operating system")
    probes = data.get("probes")
    if not isinstance(probes, list) or len(probes) != len(EXPECTED):
        raise ValueError("incomplete probe inventory")
    if {p.get("id") for p in probes if isinstance(p, dict)} != EXPECTED:
        raise ValueError("probe ID mismatch")
    if any(p.get("status") != "NOT_TESTED" or not isinstance(p.get("expect"), str)
           or not p["expect"].strip() for p in probes):
        raise ValueError("preflight cannot pre-approve probes")
    acceptance = data.get("acceptance")
    if not isinstance(acceptance, dict):
        raise ValueError("missing acceptance")
    for key in ("require_all_pass", "require_per_probe_raw_evidence",
                "require_matrix_receipt", "require_exact_source",
                "require_independent_sha256", "require_windows_and_linux"):
        if acceptance.get(key) is not True:
            raise ValueError(f"required gate disabled: {key}")
    for key in ("allow_mocked_results", "allow_direct_network_fallback",
                "allow_release_from_ci_green_alone"):
        if acceptance.get(key) is not False:
            raise ValueError(f"forbidden fallback enabled: {key}")
    return True


class ManifestPreflight(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data = json.loads(MANIFEST.read_text(encoding="utf-8"),
                              object_pairs_hook=no_duplicates)

    def test_frozen_preflight(self):
        self.assertTrue(verify_manifest(self.data))

    def test_missing_probe_fails(self):
        from copy import deepcopy
        data = deepcopy(self.data)
        data["probes"].pop()
        with self.assertRaises(ValueError):
            verify_manifest(data)

    def test_privacy_bypass_fails(self):
        from copy import deepcopy
        data = deepcopy(self.data)
        data["acceptance"]["allow_direct_network_fallback"] = True
        with self.assertRaises(ValueError):
            verify_manifest(data)

    def test_fake_green_fails(self):
        from copy import deepcopy
        data = deepcopy(self.data)
        data["probes"][0]["status"] = "PASS"
        with self.assertRaises(ValueError):
            verify_manifest(data)

    def test_duplicate_json_fails(self):
        with self.assertRaises(ValueError):
            json.loads('{"protocol":"x","protocol":"y"}', object_pairs_hook=no_duplicates)


if __name__ == "__main__":
    unittest.main()
