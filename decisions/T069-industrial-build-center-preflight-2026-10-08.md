# PULQVA — Industrial Build Center / PREPARE (2026-10-08)

Status: PREPARED, NOT BUILT. One bounded task: inspect and freeze integration/build inventory before any combined build. No human engineers hired or autonomous workers running.

## Role review gates
- Chief systems engineer: enforce CORE_CONTRACT, PRIVACY_INVARIANTS, UX_CONTRACT and source provenance.
- Rust/runtime engineer: reproducible Cargo builds, fail-closed Tor, process lifetime and cancellation.
- Security engineer: Tor DNS/egress and dependency trust; no direct fallback.
- AI/discovery engineer: typed intent, choice provenance, held-out semantics.
- Windows engineer: native toolchain, signed/hashed sidecars, clean Windows host.
- Linux engineer: packaged runtime, dynamic dependency inspection, clean Linux host.
- Release engineer: deterministic package inventory, SBOM, SHA256, provenance/attestation.
- Independent QA: test original artifacts, reject unmatched test conditions, confirm actual request -> human choice -> saved file.

These are review roles, not a claim that external experts have joined.

## Frozen source inventory (observed GitHub PR heads)
- PR80 feat/T069-intent-contract a3a934c6c04167ecb5b7874345bb590c4deb433b (draft/open)
- PR85 fix/T069-ffmpeg-monthly-source b81115a0e60bd4f0b934a42ec2a6693d6c6d6c25 (draft/open; sibling of PR87)
- PR87 feat/T069-parallel-method df5ac954485920c49e422fdb1eb5e9d3dd43c648 (draft/open)
- PR88 feat/T069-retained-profile c4df7aa2d025fe46559626a267b263c15803fdb6 (draft/open; descends from PR87)
- This preparation branch is based on PR88 head. It is not a merged integration candidate; PR85 is a separate sibling and MUST NOT be silently assumed included.
- Linux retained-commons owner dispatch 37769355186 at source 3ca7a85ef5de3f9108cc3213da173a6f65356c87 succeeded; scope is fixed selection + retained file, not AI/human UI or packaged release. Original failed run 37642119152 stays FAILED.

## Existing workflow inventory (not all build artifacts)
arti-config-contract.yml; arti-lifecycle-check.yml; arti-materialization-check.yml; arti-sidecar-check.yml; commons-tor-discovery-check.yml; continuity-guard.yml; deno-artifact-check.yml; desktop-shell-check.yml; discovery-contract-check.yml; ffmpeg-real-remux-check.yml; ffmpeg-sidecar-check.yml; intent-model-qwen08-smoke.yml; intent-runner-contract.yml; llama-sidecar-build-contract.yml; llama-sidecar-materialization.yml; rust-check.yml; t069e-matched-cd.yml; t069f-llama-server-smoke.yml; tor-readiness-check.yml; windows-existing-tor-harness.yml; ytdlp-sidecar-check.yml; ytdlp-tor-media-check.yml; ytdlp-tor-metadata-check.yml.

Workflow presence is not proof of completed or passing packaging. No release workflow or clean-machine ZIP acceptance was established by this inventory.

## Acceptance matrix before launching builds
1. Source: explicit integration parent/head/tree, exact PR85+PR88 delta reconciliation; no cherry-pick by guess.
2. Method: predeclare platform/toolchain/version, fixed fixtures, hash identities and acceptance gates before first expensive run.
3. Supply chain: exact sidecar versions/SHA256, FFmpeg monthly snapshot compatibility, Arti 2.6 baseline vs 2.7 candidate isolated.
4. Privacy: Tor bootstrap, remote DNS, fail-closed and descendant egress; any regression blocks release.
5. Semantics: real natural-language input -> typed intent -> actual candidate selection provenance -> saved file; fixed index1 fixture is insufficient.
6. Packaging: one self-contained ZIP per OS, no Python/Rust/Node/Tor/FFmpeg/model setup; verify extracted executable and clean-machine launch.
7. Provenance: SHA256 of ZIP and binaries, SBOM, source SHA, reproducible dependency lock and optional GitHub artifact attestation.
8. Independent QA: fresh Windows and Linux clean-machine evidence, rollback and negative-path checks; never accept stale/foreign evidence.

## Stop/go
GO only for a future single bounded integration PREPARE with exact conflict/diff inspection. NO-GO for mass builds, release, merging sibling branches, replacing sidecars or declaring complete E2E until gates pass.

## One next action
Inspect PR85 vs PR88 changed-file overlap and compatibility at their exact heads, then create a conflict-free integration plan and test admission criteria; one integration generation only after preflight. Keep kernel, historical recovery/INDEX sealed hashes and global last_verified_commit unchanged. No build, workflow dispatch, merge, release or scheduler activation in this preparation task.

## Agent-department integration review — PREPARE 2 (2026-10-08)

Independent role-based reviews, not actual parallel hired staff:
- Integration lead: PR85 head b81115a0e60bd4f0b934a42ec2a6693d6c6d6c25, base a3a934c6c04167ecb5b7874345bb590c4deb433b. PR88 head c4df7aa2d025fe46559626a267b263c15803fdb6, base df5ac954485920c49e422fdb1eb5e9d3dd43c648. Both remain draft/open. Their direct changed-file lists were inspected through GitHub PR files API.
- Merge-conflict reviewer: exact overlap in PR85/PR88 changed-file lists: NEXT.md and recovery/INDEX.json. No direct overlap in the product code or workflow files in these two PR diffs; this does NOT prove their complete dependency graphs are compatible.
- Build reviewer: PR85 touches apps/pulqva-desktop/src-tauri/src/main.rs, both FFmpeg workflows, sidecars/ffmpeg/SHA256SUMS, SOURCE_PROOF.json and VERSION. PR88 touches media workflow, evidence and readiness adapters. PR85 also touches PROJECT_STATE.json; PR88 does not.
- QA reviewer: PR88's live PASS predates its later documentation commits, so a new combined commit cannot inherit that exact-head acceptance. Need fresh deterministic checks on the integrated head and one separately admitted scoped media generation only if required.
- Release/security reviewer: never replace sealed recovery/INDEX.json wholesale; migrate/extend it with a versioned snapshot and independently verify SHA map. Keep Tor fail-closed and no-API-key kernel invariants. Do not claim a packaged release or Windows live E2E.

Integration method decision: choose PR88 head as candidate parent, port only PR85's FFmpeg functional/pin changes after a file-by-file review. Manually reconcile NEXT.md and recovery/INDEX.json; reconcile PROJECT_STATE.json without erasing independent lane A evidence. Before any commit, verify source heads unchanged and inspect complete patches for overlapping semantic dependencies. Reject mismatched test conditions rather than explaining away mismatches. First combined build wave only after a deterministic preflight and one frozen integration generation.

DONE: exact-head PR file inventory and overlap classification. PENDING: inspect PR85 functional patch and PR88 base inheritance, prepare one conflict-free integration generation. NOT TESTED: combined build, remux, Tor/media E2E, clean-machine ZIP. No code merge, dispatch, build, release or scheduler activation in this task.

ONE NEXT ACTION: run source-level FFmpeg patch/consumer compatibility review against PR88's exact head; record each changed consumer and pin, then authorize or reject a single integration commit.
