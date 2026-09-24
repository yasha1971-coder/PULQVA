# T059 — Wire verified Arti materialization into backend prelaunch preparation

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Use the T058 verified Arti materializer in the real completed-file backend flow before Arti can
launch, without expanding scope to FFmpeg extraction or release packaging.

## Acceptance criteria

- consume the T057-verified packaged sidecar plan already resolved from the injected Tauri AppHandle;
- prepare app-owned runtime directories before publication;
- select exactly one verified Arti artifact from the validated plan;
- call the T058 Arti materializer before building or launching the Tor runtime;
- require the materialized result path to equal both the runtime layout's Arti executable path and
  the backend-derived completed-file Arti input path;
- preserve the existing T056 yt-dlp materialization ordering and path binding;
- missing/duplicate Arti artifacts fail closed before process launch;
- an invalid/mismatched Arti materialized path fails closed before Tor runtime preparation or launch;
- do not expose any filesystem path to frontend input;
- no FFmpeg extraction/materialization in this task;
- no new network operation, shell command, direct-network fallback, or privacy weakening;
- Windows/Linux tests prove Arti materialization happens before Arti launch and that failure blocks
  the pipeline;
- all existing privacy/media/desktop checks remain green.

## Out of scope

FFmpeg archive extraction/materialization, Tauri bundle activation, installer/release packaging,
external providers, AI providers, and direct-network fallback.


## Current phase: IMPLEMENT

T059 preserves the established T056 ordering and adds T058 Arti materialization before the existing
completed-file pipeline can prepare or launch Tor:

1. validate all packaged sidecar sources;
2. materialize verified yt-dlp using the existing T056 boundary and bind its path;
3. select exactly one verified Arti artifact;
4. prepare/reuse app-owned runtime directories;
5. materialize Arti using the T058 boundary and bind its path to both the runtime layout and the
   backend-derived completed-file Arti input;
6. only then enter the existing completed-file pipeline, whose first Tor-related step is building
   the download preflight and preparing the Arti runtime.

A new orchestration helper makes this ordering testable without launching any process. Tests prove
yt-dlp -> Arti -> pipeline ordering and prove that an Arti materialization error prevents the
pipeline closure from running. Separate Arti prelaunch tests cover missing/duplicate verified
artifacts, runtime-directory preparation before materialization, and materialized-path mismatch.

No FFmpeg extraction, bundle activation, installer, external provider, AI provider, or direct-network
fallback is introduced.


## Recovery: missing test import

Head `f7dad20ceaff5100e2ce2858d1e161099ed17a70` passed every workflow except
`desktop-shell-check`. Both OS jobs failed only because the test module referenced
`DownloadActionError` without importing it. This recovery adds that import only.


## Recovery: pin the Windows linker

Direct comparison of the two 19,760,128-byte `/Brepro` Windows artifacts showed that executable
code/data are stable and drift is confined to PE/linker metadata. Their Rich headers identify a
floating MSVC tool build change from `36256` to `36257`; CodeView/Repro identity changes with it.

The Windows identity recipe now uses the `rust-lld.exe` shipped by pinned Rust 1.91.0 instead of
the mutable Visual Studio linker supplied by `windows-latest`. It keeps `/Brepro` and the
fail-closed SHA comparison. The committed digest is not changed in this phase.
