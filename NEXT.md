# NEXT

## Current verified state

T029 is complete.

PULQVA now has a controlled FFmpeg process boundary:

- launcher accepts only `FfmpegRemuxPlan`;
- executable and argv come only from the typed local-only plan;
- FFmpeg is spawned directly with no shell;
- `RunningFfmpeg` owns the child handle;
- deterministic `try_wait` and stop/wait cleanup exist;
- Windows/Linux tests use a local fixture that records exact argv;
- the fixture performs no network and creates no media output.

Verified PR head:
`0e11a4612a334b777ee7161db4bea010f043185f`

## Active atomic task

**T030 — Prove one real local FFmpeg remux**

T030 adds a dedicated ignored proof test plus Windows/Linux CI:

- exact pinned FFmpeg/ffprobe archives are SHA-256 verified before execution;
- the immutable T024 MP4 fixture is verified by Git blob identity and exact byte size;
- test-only crate-internal setup converts that local fixture into the same validated
  `CompletedDownloadResult` type without widening any production constructor;
- real execution starts only from `FfmpegRemuxPlan`;
- FFmpeg receives `-protocol_whitelist file` and stream-copy `-c copy`;
- execution is bounded;
- output must be a distinct, non-empty regular Matroska file;
- real pinned ffprobe validates the local remuxed output.

## Queued next task

**T031 — Bootstrap the Tauri 2 desktop shell**

Create the first Windows/Linux desktop shell and a minimal typed Rust command boundary while keeping
the frontend unable to own external network access.

## Do not do yet

- no transcoding/re-encoding;
- no arbitrary user URL execution from the UI;
- no AI provider;
- no frontend Internet access;
- no direct-network fallback.

## Success

The actual pinned FFmpeg sidecar performs one bounded local stream-copy remux on a validated local
artifact and the actual pinned ffprobe accepts the result, with no FFmpeg network input.
