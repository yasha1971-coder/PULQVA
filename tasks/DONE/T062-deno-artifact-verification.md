# T062 — Verify candidate Deno artifacts on Windows/Linux

Status: DONE within artifact verification scope; closeout CI/merge pending
Outcome: reproducible CI evidence for the two candidate archives and executable
identities, without enabling Deno in the application.

Use sidecars/deno/CANDIDATE.json and ADR-0006. Download only exact official assets
in CI setup; recompute size/SHA-256 before reading the ZIP. Extract only the
expected deno/deno.exe into an owned directory with traversal/type/size checks.
Record archive and executable hashes/sizes and local --version on each OS.
Use a controlled environment and disable update checks before any launch.
No install script, host runtime discovery, runtime downloads or self-updates.

Acceptance:
- Both pinned archive identities and local runtime version verified in CI.
- Receipt contains exact executable hash and extracted byte count per OS.
- Wrong hash and unsafe/missing archive entries fail closed.
- Identify observed platform/ABI limits; do not claim all Linux/Windows supported.

Scope excludes application launch wiring, permission isolation certification,
YouTube retrieval and full distribution licensing clearance. Split those later.

Evidence: 7798c67a9a8109a1fec2115d063382f4cafb56a0 passed all 11 workflows.
Run 36126900433, jobs 108044998361 (Linux), 108044998705 (Windows):
archive SHA-256/size, bounded extraction, local Deno 2.9.7 --version and five
fixture tests passed on both. Receipts are in sidecars/deno/ARTIFACT_VERIFICATION.json.
Observed Linux glibc 2.39 and Windows Server 2025 build 26100 only.
Executable sizes: Linux 95,830,104 bytes; Windows 97,462,048 bytes.
Neither a minimum OS/ABI guarantee nor proof of no network traffic is established
by a successful --version invocation. No challenge script or application launched.
