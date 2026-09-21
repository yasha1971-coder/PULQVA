# T026 — Add a completed download result for UI handoff

Parent: MEDIA FOUNDATION  
Status: READY

## Goal

Combine typed request identity and `CompletedMediaArtifactReceipt` into a stable completed-download
result for the future desktop UI boundary.

## Acceptance criteria

- typed completed-download result exists;
- result is constructible only from validated completion data;
- source URL is retained as typed source data;
- artifact canonical path and byte size come only from `CompletedMediaArtifactReceipt`;
- process handles and proxy internals are not exposed;
- no network/process/filesystem side effect is added by result construction;
- serialization/display-facing fields are deterministic;
- tests prove the result cannot substitute an unvalidated artifact path;
- all existing checks remain green.

## Out of scope

- desktop UI implementation;
- FFmpeg;
- AI provider;
- arbitrary user URL execution.
