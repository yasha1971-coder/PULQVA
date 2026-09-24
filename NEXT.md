# NEXT

## Current verified state

T058 is complete and merged on main at
`0ca4264e47cb9d8c5a464cc93e2b43561e602710`.

Verified T058 closeout head:
`d02793b5d9d2e11fb9c627b88f69dacdce4b85f5`.

All 11 required workflows passed for that exact closeout head.

## Active atomic task

**T059 — Wire verified Arti materialization into backend prelaunch preparation**

Phase: IMPLEMENT. T059 is NOT DONE.

The real completed-file backend now preserves the existing T056 yt-dlp materialization step and adds
T058 Arti materialization before the existing pipeline can prepare or launch Tor.

Implemented ordering:

1. frontend query/locator are validated into backend-owned candidate/pipeline inputs;
2. Tauri AppHandle resolves the package resource plan;
3. blocking backend validates packaged sidecar sources;
4. existing T056 prelaunch materializes yt-dlp and binds its path to the runtime layout and backend
   completed-file input;
5. exactly one verified Arti artifact is selected;
6. app-owned runtime directories are prepared/reused;
7. T058 atomically materializes Arti;
8. the materialized Arti path must equal both `runtime/bin/arti(.exe)` and the backend-derived
   completed-file Arti input path;
9. only then is the existing completed-file pipeline invoked; Tor runtime preparation/launch stays
   downstream of this boundary.

The orchestration helper is covered without launching processes. Tests prove:

- yt-dlp materialization happens before Arti materialization;
- Arti materialization happens before the pipeline;
- Arti materialization failure prevents the pipeline closure from running;
- missing/duplicate verified Arti artifacts fail before the Arti materializer callback;
- runtime directories exist before the Arti materializer callback;
- a non-layout Arti materialized path fails closed.

No frontend filesystem path is accepted. No FFmpeg extraction/materialization, new network operation,
shell command, direct-network fallback, bundle activation, installer, external provider, or AI
provider is added.

Branch:
`task/T059-wire-arti-materialization-into-backend-prelaunch`

## Immediate next action

Inspect the exact T059 implementation head and all triggered CI. If any check fails, diagnose only
that concrete failure. If all required Windows/Linux/privacy checks are green, close T059 in a later
bounded phase.

Do not start T060.


## Recovery after first T059 implementation CI

Implementation head `f7dad20ceaff5100e2ce2858d1e161099ed17a70` produced 10/11 green workflows.
Only `desktop-shell-check` failed, and both Windows/Linux jobs failed at test-module compilation
before desktop boundary tests.

Exact error: `DownloadActionError` was referenced by the new Arti-failure test closure but omitted
from the test module import list. Production behavior was not implicated.

This recovery changes only that test import. No production path, ordering, Tor behavior, privacy
boundary, or materialization logic is changed.


## Recovery: remaining Windows drift isolated to the MSVC linker

Two captured Windows `/Brepro` executables were compared directly:

- prior candidate SHA:
  `14af3c8e0d9ea0a9983f80592aaec1ca00068d3655fa8a78aa326f4a262f8a5e`;
- later fresh-run SHA:
  `e86747f49a70c13fd494326b74e70dd8c7b23947f23c81d6162752fff9af96ab`;
- both are exactly 19,760,128 bytes.

PE comparison shows:

- `.text` is byte-identical;
- `.data`, `.pdata`, and `.reloc` are byte-identical;
- only `.rdata` and PE/linker metadata differ;
- the Rich header's relevant tool build changed from `36256` to `36257`;
- the PE reproducible-build hash and CodeView GUID changed accordingly.

Therefore the remaining drift is tied to the floating MSVC linker/toolset on
`windows-latest`, not Arti source code or the Rust-generated executable code.

This recovery removes that floating linker from the Windows Arti identity recipe. The workflow now
uses `rust-lld.exe` shipped by the already pinned Rust 1.91.0 toolchain, with
`linker-flavor=lld-link` and `/Brepro`. Linux is unchanged. The existing committed Windows digest
is intentionally left unchanged so the first rust-lld candidate must fail closed and be captured;
a later phase must prove two unchanged rust-lld runs match before changing SHA256SUMS.

## Active recovery: suppress Windows PDB identity (supersedes earlier recovery notes)

PR #61 remains draft; T059 is ACTIVE / NOT DONE.
Parent head: `5c4c02475b40e1f581cbc3466703496ca3ce9d4c`.
Run `35981665243` attempts 1 and 2 failed only at Windows whole-file identity.
Artifacts `10800577059` and `10820967928` were independently hashed and compared:
both 19,624,960 bytes, but SHA-256 `16cc66dd363a357eddb9129e5b62cc000c94b1d43b9bfc80a7b9b56fcf9ed6ac`
versus `bb908c07a1c0b60e19b050bf667a7c44379a3a33228b8c83d799de3b1a8e1d2e`.
Exactly 20 bytes differ: PE/debug timestamps and the CodeView PDB GUID.
All code sections and all other bytes match. See PR checkpoint 5818360924.

This Windows-only recipe change adds `/DEBUG:NONE` to the pinned rust-lld flags,
retaining `/Brepro`. An executable acceptance check requires no CodeView debug
record and a reproducible-build marker in the actual PE output. The identity receipt
records the linker SHA-256 and effective Rust flags. Linux and product code are unchanged.

The old committed executable digest is deliberately retained. A fresh successful
compile/CLI/PE check is still expected to fail at the old-digest comparison and upload
the candidate. Do not call that mismatch a new compilation failure.

Verification: local YAML/shell/Python syntax and rejection of both old CodeView-bearing
artifacts; Windows build and reproducibility remain pending. Exact new head and CI run
IDs are recorded in the PR checkpoint after publishing this commit.

ONE next action: inspect that new head's Windows compile, CLI, PE metadata check and
captured identity. If those succeed, rerun the same Windows job once unchanged in a later
phase and compare complete executable hashes. Only matching independent builds permit
a later digest promotion. Do not merge PR #61 or start T060.
