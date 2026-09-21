# T029 — Add a controlled FFmpeg child-process launcher

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Introduce the first FFmpeg process side effect behind `FfmpegRemuxPlan`, while keeping the proof
local and media-transformation-free.

## Acceptance criteria

- launcher accepts only `FfmpegRemuxPlan`;
- executable and argv come only from the typed plan;
- process is spawned directly, never through a shell;
- typed running-process capability owns the child handle;
- deterministic stop/wait cleanup exists;
- Windows and Linux tests use a local fixture executable that records argv;
- fixture proves local-only/remux argv is passed unchanged;
- fixture performs no network and no media transformation;
- no direct-network fallback exists;
- all existing checks remain green.

## Out of scope

- real FFmpeg remux/transcode;
- desktop UI;
- AI provider;
- arbitrary user URL execution.
