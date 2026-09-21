# T026 — Add a completed download result for UI handoff

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Combine typed request identity and `CompletedMediaArtifactReceipt` into a stable completed-download
result for the future desktop UI boundary.

## Acceptance criteria

- typed `CompletedDownloadResult` exists;
- result is created only after successful child completion and validated artifact receipt creation;
- source URL is retained as `YtDlpMediaSourceUrl`;
- artifact canonical path and byte size come only from `CompletedMediaArtifactReceipt`;
- result construction cannot accept a raw/unvalidated artifact path;
- process handles and proxy/Tor internals are not exposed;
- deterministic display-facing fields expose source URL, canonical artifact path, and byte size;
- no network/process/filesystem side effect is added by result construction;
- tests prove successful completion produces the expected source/artifact fields;
- all existing checks remain green.

## Out of scope

- desktop UI implementation;
- FFmpeg;
- AI provider;
- arbitrary user URL execution.
