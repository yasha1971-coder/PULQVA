# NEXT

## CURRENT: T060-C tar.xz stream adapter, CI pending

Supersedes prior current sections below. T060-B head
`ecbe966eb8c3f5f5f9aa63dbf425ba5a147981e9` passed all 11 workflows.
PR #62 / branch `task/T060-verified-ffmpeg-archive-extraction` remains draft.

Added immutable tar.xz-byte adapter: source SHA before parsing, 256 MiB source/decoder-memory
limits, 2 GiB total decompressed-byte budget including nonselected members/padding, raw TAR
entry policy, per-member actual-size checks, extracted-byte hash receipt and full XZ footer
consumption. Nonzero trailing TAR data and bytes beyond the single XZ stream fail closed.
GNU long-name/PAX/sparse extensions and links/special entries are unsupported and rejected.

Four grouped Rust tests cover valid extraction, links/duplicates/wrong SHA/truncated XZ,
exact-limit versus over-limit reading, and trailing XZ data. tar 0.4.44 and xz2 0.1.7
are pinned; xz2 uses static liblzma. Rust/Cargo unavailable locally: CI compile/tests pending.
Local continuity/diff checks precede publication. New head/run IDs are checkpointed in PR #62.

NOT DONE: parent T060. Both adapters still write to an internal writer, not an owned stage;
partial data on failure MUST be discarded by future owned-stage integration. Actual pinned
BtbN archive compatibility remains NOT TESTED, including TAR extension requirements.
ONE next action: inspect exact-head CI; diagnose any failure first. If green, inspect real
archive compatibility and implement authenticated source snapshot + owned staging/cleanup
in a later bounded phase. Do not wire runtime publication or merge incomplete T060.

## CURRENT: T060-B ZIP stream adapter, CI pending

Supersedes the T060-A current section below. All 11 workflows passed for T060-A head
`b6b0dbd8d9b6c0191a659e693b7c0db11952b11f`, including Windows/Linux desktop tests.
PR #62 / branch `task/T060-verified-ffmpeg-archive-extraction` remains draft and ACTIVE.

Added an immutable ZIP-byte adapter: authenticate the entire bounded source against a
backend-supplied pinned digest before parsing; reject unsupported ZIP layouts, metadata
count ambiguity, unsafe paths/types/encryption; select one exact FFmpeg; stream stored/deflate
data into an internal writer with actual byte limits, CRC checking and output digest receipt.
ZIP64/multipart/prefixed/trailing layouts are deliberately unsupported in this phase.

The writer is NOT yet an owned filesystem stage. On failure it may hold partial data and
the later staging caller MUST discard it. No production caller, filesystem publication,
runtime integration, or tar.xz adapter is implemented. Parent T060 is NOT DONE.
zip 2.4.2 is pinned with default features disabled and deflate enabled; its upstream API
source was read. Real pinned BtbN archive compatibility is NOT TESTED yet.

Local continuity/diff checks run before publication. Rust/Cargo remain unavailable locally;
new Rust compilation/tests await CI. Exact new head/run IDs are recorded in PR #62.
ONE next action: inspect this exact head's CI; diagnose failures first. If green, continue
bounded extraction work with tar.xz and then owned staging/source-file integration, including
real pinned-archive compatibility. No merge until the complete T060 contract is verified.

## CURRENT: T060-A metadata policy, CI pending

This section supersedes the historical T059 closeout notes below.
T059 merged in PR #61; main `d14b742e9662b1ccb68f64d28500e97efdff5454`
passed all 11 post-merge workflows. Branch: `task/T060-verified-ffmpeg-archive-extraction`.

T060-A adds a backend-only pure archive metadata policy and five grouped Rust tests.
It rejects unsafe paths/types, duplicate executable members and declared-size/count overflows.
Any rejected entry poisons the scan. No file extraction, authentication or staging is implemented.
Header limits alone do NOT enforce actual decompressed-byte bounds. No production caller is wired.

Rust/Cargo is unavailable locally: compilation and Rust tests await Windows/Linux desktop CI.
Local continuity and diff checks run before publication. Exact head/PR/runs are checkpointed
in the new PR. Parent T060 remains ACTIVE / NOT DONE.

ONE next action: observe exact T060-A head CI; diagnose any failure first. If green, a later
bounded phase may implement T060-B (authenticated bounded ZIP adapter). Do not merge or call
T060 complete on the strength of metadata-only checks.

## Historical T059 closeout snapshot (superseded above)

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
