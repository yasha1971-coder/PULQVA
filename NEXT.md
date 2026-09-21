# NEXT

## Current verified state

T028 is complete.

PULQVA now has a pure-data local-only `FfmpegRemuxPlan`:

- explicit FFmpeg executable path;
- input copied only from `CompletedDownloadResult`;
- typed MP4/Matroska output container;
- explicit output path distinct from the validated input;
- parent traversal is rejected;
- deterministic argv contains `-nostdin`, `-y`, `-protocol_whitelist file`, `-map 0`, and `-c copy`;
- no URL, proxy, shell, process spawn, or runtime network surface exists in the plan.

Verified PR head:
`33c5ef60429f88842967a1677256e2f44d5aaf96`

## Active atomic task

**T029 — Add a controlled FFmpeg child-process launcher**

The first FFmpeg process side effect is now behind `FfmpegRemuxPlan`:

- launcher accepts only the typed remux plan;
- executable and argv come only from that plan;
- child is spawned directly with no shell;
- `RunningFfmpeg` owns the child handle;
- deterministic stop/wait cleanup exists;
- a local fixture records exact argv;
- the fixture performs no network and creates no media output;
- Windows/Linux package tests exercise the fixture path.

## Queued next task

**T030 — Prove one real local FFmpeg remux**

Use the actual pinned FFmpeg sidecar on one validated local media artifact, perform a bounded
stream-copy remux with the typed process path, and verify the resulting file locally with ffprobe.

## Do not do yet

- no transcoding/re-encoding;
- no desktop UI implementation;
- no AI provider;
- no arbitrary user URL execution;
- no direct-network fallback.

## Success

A typed local-only remux plan can cross the process boundary with exact argv and deterministic child
ownership while the proof remains network-free and transformation-free.
