# T069 Recovery Handoff v2 — immutable lineage design

Status: PREPARED / NO CI ACCEPTANCE. Parent PR #92 at 0c0f32c934eb4a6184f460558b99832aa9fbc107. One bounded design phase. Do not interpret as authority to merge, release or dispatch.

## Failure evidence
- CI continuity-guard run 37854628179 job 113575727659: `scripts/recovery_guard.py` reports `sealed file changed: NEXT.md`.
- Historical `recovery/INDEX.json` blob dd3711d6512cec3907c3ac16708d80a18ccef401 records `NEXT.md` Git blob 001f3ffac0bb5c2f8b936f9099733429513573c1; current blob b0243192bf64c40be8150219ed3550364a614401.
- The same validator pins branch feat/T069-intent-contract, PR80, product trees, kernel tree, state contract, historical model archive, and ADR inventory. It is not a generic latest-head validator. Blindly rehashing NEXT.md will violate the original sealed-generation meaning.
- Separate desktop failure is a Tor-readiness error-text expectation. Out of scope for this handoff.

## Industrial review roles and hypotheses
Security: H1 historical index can be preserved byte-for-byte while a new signed/hashed handoff envelope records its identity and target state. Reject if any old evidence changes.
Recovery: H2 two independent validators (v1 original on its exact historical source, v2 handoff on new generation) can establish continuity without allowing arbitrary changes. Reject if v2 passes with a mutated ancestor.
Build: H3 target tree/commit and all required file identities can be bound before any CI or release attempt. Reject if foreign head or unstaged changes are accepted.
QA: H4 negative controls for wrong parent, changed NEXT, mismatched tree, duplicate JSON keys, symlink/path traversal, missing archive and old-index tamper must all fail closed.
Governance: H5 an append-only evidence record can distinguish accepted historical scope from pending product E2E. Reject if a historical green run becomes implied release acceptance.

## Proposed data contract (v2; NOT implemented)
A separate `recovery/handoffs/<generation>.json`, never overwriting `recovery/INDEX.json`:
- schema: 2, protocol: pulqva-recovery-handoff-v2
- parent: exact historical source commit/tree, original INDEX Git blob and SHA256 of bytes, expected v1 validator identity, verified parent outcome reference
- target: exact integration commit/tree, parent(s), declared branch/PR, manifest of Git blob identities for NEXT.md, PROJECT_STATE.json, kernel and relevant task/evidence files
- scope: explicitly accepted Linux fixed Commons request -> 10 candidates -> index1 -> verified file; explicitly NOT AI/human UI/Windows live/package
- evidence: run 37769355186 attempt1 source 3ca7a85ef5de3f9108cc3213da173a6f65356c87; artifact 11546608929 SHA256 b6dc1ca740ef4b00876d477308f033594ea0be5d27712a57df39c91fb8730b7f; selected.webm SHA256 0d77b81c7670ff7766240766a83a7fab4a3ad3aeb81d72f039113285b0acf423
- method: independent immutable generation, exact source and evaluator, no hot repairs, no old/new artifact mixing
- authority: explicit owner action where needed, never inferred from historical scheduler record; no automation reactivation

## Acceptance protocol (future implementation)
1. PREPARE: freeze exact parent and target; validate original v1 INDEX against the original historical commit in an isolated read-only checkout. Do not demand it match newer NEXT.md.
2. VALIDATE: independently read Git blobs and SHA256 for parent index, all target manifest files, and exact ancestry. Do not trust user-supplied path strings without canonical allowlist; reject duplicate keys and symlinks.
3. NEGATIVE MATRIX: wrong commit, wrong parent, wrong index digest, wrong NEXT digest, missing evidence, invalid scope, symlink escape, duplicate JSON key, tampered receipt. Each must fail closed; original failures retained.
4. ACCEPT: emit a bounded `matrix_receipt.json` with per-probe ID, hypothesis, invariant set, result and duration. Record actual checkout and CI job/run identity. No green status if any mandatory probe unexecuted.
5. INTEGRATE: update the continuity workflow to invoke v1 on historical source and v2 on integration source. Keep both validators and evidence; never skip/disable old validation merely to turn CI green.
6. RELEASE GATE: separate Windows/Linux fresh compilation, FFmpeg remux, Tor egress, real user-choice E2E and clean-machine ZIP; v2 recovery PASS alone never satisfies these.

## Explicit stop/go
DONE: identify precise v1/target mismatch and design v2 lineage boundary.
PENDING: implementation of validator + negative tests + CI wiring as a separate bounded task.
BLOCKED: PR #92 continuity acceptance until a verified handoff exists.
NOT TESTED: v2 runtime, its test matrix, combined release, real AI/human-choice E2E.

ONE NEXT ACTION: implement a read-only v2 validator and synthetic negative matrix in a separate source generation; do not modify historical INDEX or kernel, do not suppress v1 guard, and do not dispatch live media.
