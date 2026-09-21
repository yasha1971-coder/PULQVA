# T030 — Prove one real local FFmpeg remux

Parent: MEDIA FOUNDATION  
Status: ACTIVE

## Goal

Run the actual pinned FFmpeg sidecar through the typed local process boundary on one validated local
media artifact and prove a bounded stream-copy remux.

## Acceptance criteria

- actual pinned FFmpeg/ffprobe snapshot is downloaded and SHA-256 verified;
- immutable media fixture is verified by Git blob SHA-1 and exact byte size;
- test-only setup creates a validated `CompletedDownloadResult` without widening production APIs;
- real FFmpeg execution starts only from `FfmpegRemuxPlan`;
- input is a local regular file;
- argv retains `-protocol_whitelist file`;
- remux retains `-c copy` and performs no re-encoding;
- execution is bounded by an explicit timeout;
- output is a non-empty regular file distinct from input;
- actual pinned ffprobe validates the local Matroska output;
- Windows and Linux run the same real-remux proof;
- no FFmpeg network input or direct-network fallback exists;
- all existing checks remain green.

## Out of scope

- transcoding/re-encoding;
- desktop UI;
- AI provider;
- arbitrary user URL execution.
