# T043 — Add the backend-only FFmpeg remux planning boundary

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Convert a verified T042 completed download result into a deterministic typed local FFmpeg remux plan
without starting FFmpeg or exposing filesystem/runtime details to frontend code.

## Acceptance criteria

- input is a verified `CompletedDownloadResult`;
- FFmpeg executable path is explicit backend input;
- remux output path is derived by backend code from the validated completed artifact path;
- the typed plan reuses the existing `FfmpegRemuxPlan` API;
- default container is an explicit supported typed container;
- invalid executable path fails closed;
- invalid/missing/traversing/equal-to-input output fails closed through the existing typed plan;
- plan arguments remain local-file-only and contain the existing `file` protocol whitelist;
- frontend receives no executable path, input/output filesystem path, argv, source URL, or process identifier;
- frontend behavior remains unchanged;
- no FFmpeg process is spawned;
- no external network access occurs;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- FFmpeg process execution;
- completed result surfacing to frontend;
- external candidate search;
- AI provider integration;
- packaging/release installers.
