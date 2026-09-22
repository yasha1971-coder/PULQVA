# T045 — Add the backend-only completed FFmpeg remux result boundary

Parent: DESKTOP FOUNDATION  
Status: READY

## Goal

Complete a running T044 FFmpeg remux into a validated typed backend-only result.

## Acceptance criteria

- input is a T044 `RunningFfmpegRemuxRuntime`;
- FFmpeg completion waits for the owned process and fails closed on non-zero exit;
- expected output path is retained from the verified remux plan, not accepted from frontend input;
- successful completion validates that the output exists;
- output must be a regular file and not a symlink;
- output must be non-empty;
- output must remain distinct from the validated input artifact;
- successful completion returns a typed backend-only remux result;
- process/completion/artifact failures remain distinguishable;
- no shell command string is constructed;
- no URL, proxy, SOCKS endpoint, or external-network field is introduced;
- frontend receives no executable path, input/output filesystem path, argv, source URL, process identifier, or raw completion internals;
- frontend behavior remains unchanged;
- Windows and Linux desktop checks remain green;
- all existing privacy/media checks remain green.

## Out of scope

- completed remux result surfacing to frontend;
- external candidate search;
- AI provider integration;
- packaging/release installers.
