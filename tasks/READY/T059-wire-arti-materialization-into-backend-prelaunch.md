# T059 — Wire verified Arti materialization into backend prelaunch preparation

Parent: DESKTOP FOUNDATION  
Status: READY

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
