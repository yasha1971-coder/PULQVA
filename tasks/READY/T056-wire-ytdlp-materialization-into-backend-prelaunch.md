# T056 — Wire verified yt-dlp materialization into backend prelaunch preparation

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Use the T055 verified yt-dlp materializer in the real completed-file backend flow before any yt-dlp
process can launch, without expanding scope to Arti or FFmpeg materialization.

## Acceptance criteria

- consume the T054 verified sidecar plan already resolved from the injected Tauri AppHandle;
- prepare app-owned runtime directories before publication;
- select exactly the verified yt-dlp artifact from the validated plan;
- call the T055 backend materializer before building/running the completed-file pipeline;
- require the materialized result path to equal the runtime layout's yt-dlp executable path;
- do not expose any filesystem path to frontend input;
- do not materialize Arti or extract FFmpeg in this task;
- no new network operation, shell command, direct-network fallback, or privacy weakening;
- invalid/missing/duplicate yt-dlp artifacts fail closed before process launch;
- Windows and Linux desktop tests prove ordering and fail-closed behavior;
- all existing privacy/media/remux checks remain green.

## Out of scope

Arti executable authentication/materialization, FFmpeg archive extraction, Tauri bundle activation,
installer/release packaging, external candidate search, AI providers, and direct-network fallback.
