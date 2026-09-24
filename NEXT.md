# NEXT

## Current verified state

T059 implementation is complete and verified at
`c7b81f00fe23803bfc79c61a48f14afddef055e3`.
All 11 required workflows passed for that exact head, including Windows/Linux Arti identity
(`36037909043`), desktop shell, Rust, continuity and privacy/media checks.

The completed-file backend now materializes yt-dlp first, then authenticated Arti,
binds the resulting Arti path to the backend runtime layout, and only then enters the
pipeline that can prepare/launch Tor. Missing, duplicate, failed or mismatched Arti
materialization blocks that pipeline. No new network path was introduced.

Windows Arti reproducibility recovery is verified. Pinned Rust 1.91.0 rust-lld with
`/Brepro /DEBUG:NONE` produced two byte-identical executables; the promoted SHA-256 is
`9245c7b5f71391238539bf2d78a492453cae66f978cf8e479039a26f1667f3df`.
The subsequent promotion-head CI passed, including full executable SHA verification.
Evidence/provenance: `sidecars/arti/README.md` and PR #61 checkpoints.

## Closeout pending

Branch: `task/T059-wire-arti-materialization-into-backend-prelaunch`
Draft PR: #61

This metadata-only closeout must pass its own CI before PR #61 is made ready and merged.
The exact closeout head and new workflow IDs are recorded in the PR checkpoint.
Do not merge based only on the previous green implementation head.

ONE next action: observe the exact closeout head's required CI. If green, finish PR #61;
if any failure occurs, diagnose it before unrelated work. Do not start T060 before merge.

## Next atomic task after T059 merge

T060 — Safely extract FFmpeg from the verified packaged archive.

Implement the backend-only bounded extraction-to-owned-staging boundary described in
`tasks/READY/T060-verified-ffmpeg-archive-extraction.md`. FFmpeg is an authenticated archive,
not a direct-binary artifact. Runtime publication and prelaunch wiring remain later tasks.
T060 is specified only; no implementation or new build for it has started.

## Retained limits

T059 is not release packaging, bundle-resource or end-to-end natural-language download proof.
Existing filesystem/concurrency trust limits from T058 remain. Kernel/privacy contracts unchanged.
