# T029 — Add a controlled FFmpeg child-process launcher

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Introduce the first FFmpeg process side effect behind `FfmpegRemuxPlan`, while keeping the proof
local and media-transformation-free.

## Acceptance criteria

- launcher accepts only `FfmpegRemuxPlan`;
- executable and argv come only from the typed plan;
- process is spawned directly, never through a shell;
- typed `RunningFfmpeg` owns the child handle;
- deterministic `try_wait` and stop/wait cleanup exist;
- Windows and Linux package tests use a local fixture executable;
- fixture records and proves exact local-only/remux argv;
- fixture performs no network;
- fixture creates no remux/media output;
- no direct-network fallback exists;
- all existing checks remain green.

## Out of scope

- real FFmpeg remux/transcode;
- desktop UI;
- AI provider;
- arbitrary user URL execution.
