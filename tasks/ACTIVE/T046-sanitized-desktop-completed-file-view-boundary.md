# T046 — Add a sanitized desktop completed-file view boundary

Parent: DESKTOP FOUNDATION  
Status: ACTIVE

## Goal

Convert a verified T045 completed remux result into a deterministic data-only desktop success value
without exposing backend filesystem or runtime internals.

## Acceptance criteria

- input is a verified `CompletedFfmpegRemuxResult`;
- output includes only a safe display filename, byte size, and stable completion stage;
- display filename is derived from the validated output file name only;
- missing output file name fails closed;
- no absolute or parent filesystem path is serialized;
- no executable path, argv, source URL, proxy/SOCKS data, process identifier, or raw result internals are serialized;
- frontend behavior remains unchanged;
- no process is started;
- no external network access occurs;
- Windows and Linux desktop checks remain green;
- existing privacy/media/remux checks remain green.

## Out of scope

- live desktop command wiring for completed results;
- external candidate search;
- AI provider integration;
- packaging/release installers.
