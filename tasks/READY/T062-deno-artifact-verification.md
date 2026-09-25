# T062 — Verify candidate Deno artifacts on Windows/Linux

Status: READY — start only after PR #64 merges
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
