"""Fail closed when the independently reviewed Arti source or feature boundary changes."""
import argparse
import json
from pathlib import Path

def verify(target, registry_receipt, qualification):
    records = json.loads(registry_receipt.read_text(encoding="utf-8-sig"))
    records_by_kind = {r["kind"]: r for r in records}
    expected = {
        "src/*/arti-2.6.0/Cargo.lock": qualification["cargo_lock_sha256"],
        "cache/*/arti-2.6.0.crate": qualification["source_crate_sha256"],
    }
    if len(records) != 2 or set(records_by_kind) != set(expected):
        raise ValueError("Missing or duplicate reviewed source/lock receipt")
    for kind, digest in expected.items():
        if records_by_kind[kind]["sha256"] != digest:
            raise ValueError("Reviewed source/lock drift: " + kind)
    banned = {"legacy", "experimental", "seekable", "zstdmt", "zdict_builder"}
    observed = {}
    for crate in ("zstd", "zstd_safe", "zstd_sys"):
        files = sorted((target / "release" / ".fingerprint").glob("*/lib-" + crate + ".json"))
        if not files:
            raise ValueError("Missing compiled feature fingerprint: " + crate)
        features = set()
        for path in files:
            value = json.loads(path.read_text())["features"]
            features.update(json.loads(value) if isinstance(value, str) else value)
        if features & banned:
            raise ValueError("Unreviewed security feature: " + crate + " " + repr(sorted(features & banned)))
        observed[crate] = sorted(features)
    if qualification["security_result"] != "PASS_SCOPED_KNOWN_ADVISORY_APPLICABILITY":
        raise ValueError("Candidate lacks scoped security review")
    return {"status": "PASS_SCOPED", "observed_compiled_features": observed,
            "source_crate_sha256": qualification["source_crate_sha256"],
            "cargo_lock_sha256": qualification["cargo_lock_sha256"],
            "scope": "Exact reviewed source and Windows feature exclusions; not a general vulnerability scan"}

def main():
    p = argparse.ArgumentParser()
    for name in ("target", "registry-receipt", "qualification", "output"):
        p.add_argument("--" + name, required=True, type=Path)
    a = p.parse_args()
    result = verify(a.target, a.registry_receipt, json.loads(a.qualification.read_text()))
    with a.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2)
    print("PULQVA_ARTI_SECURITY_QUALIFICATION_PASS", json.dumps(result, sort_keys=True))

if __name__ == "__main__":
    main()
